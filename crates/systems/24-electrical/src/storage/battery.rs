pub const MAX_CELLS: usize = 16;
pub const OCV_SOC_TABLE: [(f64, f64); 11] = [
    (0.0, 2.80),
    (0.1, 3.30),
    (0.2, 3.45),
    (0.3, 3.55),
    (0.4, 3.62),
    (0.5, 3.68),
    (0.6, 3.74),
    (0.7, 3.82),
    (0.8, 3.92),
    (0.9, 4.05),
    (1.0, 4.18),
];
pub const R0_CELL_25C_OHM: f64 = 0.0006;
pub const APPROX_R0_TEMP_COEF_PER_K: f64 = 0.03;
pub const APPROX_R0_LOW_SOC_FACTOR: f64 = 0.8;
pub const APPROX_R1_RATIO: f64 = 0.5;
pub const APPROX_RC_TAU_S: f64 = 30.0;
pub const PACK_THERMAL_MASS_J_PER_K: f64 = 25_000.0;
pub const APPROX_PACK_COOLING_W_PER_K: f64 = 2.0;
pub const APPROX_CAP_TOLERANCE: f64 = 0.02;
pub const APPROX_R_TOLERANCE: f64 = 0.05;
pub const APPROX_SOC_TOLERANCE: f64 = 0.01;
pub const APPROX_BLEED_CURRENT_A: f64 = 0.2;

pub fn ocv_from_soc(soc: f64) -> f64 {
    let s = soc.clamp(0.0, 1.0);
    for w in OCV_SOC_TABLE.windows(2) {
        let (s0, v0) = w[0];
        let (s1, v1) = w[1];
        if s <= s1 {
            return v0 + (v1 - v0) * (s - s0) / (s1 - s0);
        }
    }
    OCV_SOC_TABLE[OCV_SOC_TABLE.len() - 1].1
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryParams {
    pub cells: usize,
    pub capacity_ah: f64,
    pub r0_cell_25c_ohm: f64,
    pub rc_tau_s: f64,
    pub thermal_mass_j_per_k: f64,
    pub cooling_w_per_k: f64,
}

impl BatteryParams {
    pub fn new(cells: usize, capacity_ah: f64) -> Self {
        Self {
            cells: cells.clamp(1, MAX_CELLS),
            capacity_ah,
            r0_cell_25c_ohm: R0_CELL_25C_OHM * 75.0 / capacity_ah.max(1.0),
            rc_tau_s: APPROX_RC_TAU_S,
            thermal_mass_j_per_k: PACK_THERMAL_MASS_J_PER_K,
            cooling_w_per_k: APPROX_PACK_COOLING_W_PER_K,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CellState {
    pub soc: f64,
    pub v_rc: f64,
    pub cap_scale: f64,
    pub r_scale: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryOutput {
    pub terminal_v: f64,
    pub cell_v: [f64; MAX_CELLS],
    pub soc_mean: f64,
    pub temp_c: f64,
    pub heat_w: f64,
    pub current_a: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryPack {
    pub params: BatteryParams,
    pub cells: [CellState; MAX_CELLS],
    temp_c: f64,
}

impl BatteryPack {
    pub fn new(params: BatteryParams, serial_seed: u64, soc: f64, temp_c: f64) -> Self {
        let mut rng = SplitMix64(serial_seed);
        let mut cells = [CellState::default(); MAX_CELLS];
        for c in cells.iter_mut().take(params.cells) {
            c.cap_scale = 1.0 + APPROX_CAP_TOLERANCE * rng.unit();
            c.r_scale = 1.0 + APPROX_R_TOLERANCE * rng.unit();
            c.soc = (soc + APPROX_SOC_TOLERANCE * rng.unit()).clamp(0.0, 1.0);
        }
        Self {
            params,
            cells,
            temp_c,
        }
    }

    pub fn temp_c(&self) -> f64 {
        self.temp_c
    }

    pub fn cell_r0(&self, idx: usize) -> f64 {
        let c = &self.cells[idx];
        let temp_f = (APPROX_R0_TEMP_COEF_PER_K * (25.0 - self.temp_c)).exp();
        let soc_f = 1.0 + APPROX_R0_LOW_SOC_FACTOR * (1.0 - c.soc).powi(2);
        self.params.r0_cell_25c_ohm * c.r_scale * temp_f * soc_f
    }

    pub fn cell_ocv(&self, idx: usize) -> f64 {
        ocv_from_soc(self.cells[idx].soc)
    }

    pub fn open_circuit_v(&self) -> f64 {
        (0..self.params.cells).map(|k| self.cell_ocv(k)).sum()
    }

    pub fn soc_mean(&self) -> f64 {
        let n = self.params.cells;
        self.cells.iter().take(n).map(|c| c.soc).sum::<f64>() / n as f64
    }

    pub fn thevenin(&self) -> (f64, f64) {
        let n = self.params.cells;
        let emf: f64 = (0..n).map(|k| self.cell_ocv(k) - self.cells[k].v_rc).sum();
        let r: f64 = (0..n).map(|k| self.cell_r0(k)).sum();
        (emf, r)
    }

    pub fn terminal_v(&self, discharge_a: f64) -> f64 {
        let (emf, r) = self.thevenin();
        emf - discharge_a * r
    }

    pub fn step(
        &mut self,
        dt_us: u64,
        discharge_a: f64,
        ambient_c: f64,
        bleed: [bool; MAX_CELLS],
    ) -> BatteryOutput {
        let dt = dt_us as f64 * 1.0e-6;
        let n = self.params.cells;
        let tau = self.params.rc_tau_s;
        let decay = (-dt / tau).exp();
        let mut heat = 0.0;
        let mut cell_v = [0.0; MAX_CELLS];
        let mut v_sum = 0.0;
        for k in 0..n {
            let r0 = self.cell_r0(k);
            let r1 = r0 * APPROX_R1_RATIO;
            let c = &mut self.cells[k];
            let i_cell = discharge_a + if bleed[k] { APPROX_BLEED_CURRENT_A } else { 0.0 };
            c.soc = (c.soc - i_cell * dt / (self.params.capacity_ah * 3600.0 * c.cap_scale)).clamp(0.0, 1.0);
            c.v_rc = c.v_rc * decay + discharge_a * r1 * (1.0 - decay);
            let v = ocv_from_soc(c.soc) - discharge_a * r0 - c.v_rc;
            cell_v[k] = v;
            v_sum += v;
            heat += discharge_a * discharge_a * r0 + c.v_rc * c.v_rc / r1;
            if bleed[k] {
                heat += APPROX_BLEED_CURRENT_A * v;
            }
        }
        let cool = self.params.cooling_w_per_k * (self.temp_c - ambient_c);
        self.temp_c += (heat - cool) * dt / self.params.thermal_mass_j_per_k;
        BatteryOutput {
            terminal_v: v_sum,
            cell_v,
            soc_mean: self.soc_mean(),
            temp_c: self.temp_c,
            heat_w: heat,
            current_a: discharge_a,
        }
    }

    pub fn set_temp_c(&mut self, t: f64) {
        self.temp_c = t;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_BLEED: [bool; MAX_CELLS] = [false; MAX_CELLS];

    fn pack(soc: f64, temp: f64) -> BatteryPack {
        BatteryPack::new(BatteryParams::new(8, 75.0), 42, soc, temp)
    }

    #[test]
    fn ocv_curve_monotone_and_nominal() {
        let mut prev = 0.0;
        for k in 0..=100 {
            let v = ocv_from_soc(k as f64 / 100.0);
            assert!(v >= prev);
            prev = v;
        }
        let mid = ocv_from_soc(0.5) * 8.0;
        assert!((mid - 29.6).abs() < 0.8, "{mid}");
        assert_eq!(ocv_from_soc(-1.0), 2.8);
        assert_eq!(ocv_from_soc(2.0), 4.18);
    }

    #[test]
    fn discharge_curve_decreases_and_conserves_charge() {
        let mut b = pack(1.0, 25.0);
        let start_soc = b.soc_mean();
        let mut prev = b.terminal_v(20.0);
        let mut v_half = 0.0;
        for n in 0..3600 {
            let o = b.step(1_000_000, 20.0, 25.0, NO_BLEED);
            assert!(o.terminal_v <= prev + 0.05, "{n}");
            prev = o.terminal_v;
            if n == 1800 {
                v_half = o.terminal_v;
            }
        }
        let used = (start_soc - b.soc_mean()) * 75.0;
        assert!((used - 20.0).abs() < 1.0, "{used}");
        assert!(v_half > 28.0 && v_half < 33.0, "{v_half}");
        assert!(prev < v_half);
    }

    #[test]
    fn sag_grows_when_cold() {
        let warm = pack(0.8, 25.0);
        let cold = pack(0.8, -20.0);
        let sag_w = warm.terminal_v(0.0) - warm.terminal_v(150.0);
        let sag_c = cold.terminal_v(0.0) - cold.terminal_v(150.0);
        assert!(sag_c > 2.5 * sag_w, "{sag_w} {sag_c}");
        assert!(sag_w > 0.5 && sag_w < 6.0);
    }

    #[test]
    fn sag_grows_at_low_soc() {
        let hi = pack(0.9, 25.0);
        let lo = pack(0.1, 25.0);
        assert!(lo.terminal_v(0.0) - lo.terminal_v(100.0) > hi.terminal_v(0.0) - hi.terminal_v(100.0));
    }

    #[test]
    fn polarisation_relaxes() {
        let mut b = pack(0.7, 25.0);
        for _ in 0..300 {
            b.step(1_000_000, 100.0, 25.0, NO_BLEED);
        }
        let loaded = b.terminal_v(0.0);
        let ocv = b.open_circuit_v();
        assert!(loaded < ocv - 0.1);
        for _ in 0..300 {
            b.step(1_000_000, 0.0, 25.0, NO_BLEED);
        }
        assert!((b.terminal_v(0.0) - b.open_circuit_v()).abs() < 0.01);
    }

    #[test]
    fn heating_from_current() {
        let mut b = pack(0.8, 25.0);
        let mut heat = 0.0;
        for _ in 0..600 {
            heat = b.step(1_000_000, 150.0, 25.0, NO_BLEED).heat_w;
        }
        assert!(heat > 100.0);
        assert!(b.temp_c() > 27.0);
        for _ in 0..40_000 {
            b.step(1_000_000, 0.0, 25.0, NO_BLEED);
        }
        assert!((b.temp_c() - 25.0).abs() < 0.5);
    }

    #[test]
    fn seeded_imbalance_is_repeatable() {
        let a = BatteryPack::new(BatteryParams::new(8, 75.0), 7, 0.6, 25.0);
        let b = BatteryPack::new(BatteryParams::new(8, 75.0), 7, 0.6, 25.0);
        let c = BatteryPack::new(BatteryParams::new(8, 75.0), 8, 0.6, 25.0);
        assert_eq!(a, b);
        assert_ne!(a.cells[0], c.cells[0]);
        let spread = a.cells.iter().take(8).map(|x| x.soc).fold(0.0_f64, f64::max)
            - a.cells.iter().take(8).map(|x| x.soc).fold(1.0_f64, f64::min);
        assert!(spread > 0.0 && spread <= 0.02);
    }

    #[test]
    fn cell_count_and_capacity_are_parameters() {
        let p = BatteryParams::new(6, 50.0);
        assert_eq!(p.cells, 6);
        let b = BatteryPack::new(p, 1, 0.5, 25.0);
        assert!((b.open_circuit_v() - 6.0 * 3.68).abs() < 0.3);
        assert_eq!(BatteryParams::new(99, 65.0).cells, MAX_CELLS);
    }

    #[test]
    fn bleed_lowers_soc_of_bled_cell() {
        let mut b = pack(0.9, 25.0);
        let before = b.cells[2].soc;
        let mut m = NO_BLEED;
        m[2] = true;
        for _ in 0..3600 {
            b.step(1_000_000, 0.0, 25.0, m);
        }
        assert!(before - b.cells[2].soc > 0.001);
    }
}
