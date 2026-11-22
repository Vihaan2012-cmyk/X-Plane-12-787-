use super::battery::MAX_CELLS;

pub const CELL_OV_V: f64 = 4.2;
pub const APPROX_CELL_OV_DELAY_S: f64 = 0.5;
pub const CELL_UV_CUTOFF_V: f64 = 2.5;
pub const APPROX_CELL_UV_DELAY_S: f64 = 1.0;
pub const OVER_TEMP_DELAY_S: f64 = 1.0;
pub const OVER_TEMP_C: f64 = 70.0;
pub const APPROX_OVER_TEMP_RESET_C: f64 = 55.0;
pub const APPROX_IMBALANCE_FAULT_V: f64 = 0.15;
pub const APPROX_IMBALANCE_DELAY_S: f64 = 10.0;
pub const APPROX_BALANCE_START_V: f64 = 0.01;
pub const APPROX_BALANCE_MARGIN_V: f64 = 0.005;
pub const APPROX_BALANCE_MIN_CELL_V: f64 = 3.9;
pub const APPROX_BALANCE_MAX_CURRENT_A: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BmuInput {
    pub dt_us: u64,
    pub cells: usize,
    pub cell_v: [f64; MAX_CELLS],
    pub temp_c: f64,
    pub current_a: f64,
    pub reset: bool,
    pub bmu_internal_fail: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BmuOutput {
    pub balance: [bool; MAX_CELLS],
    pub disconnect: bool,
    pub over_voltage: bool,
    pub under_voltage: bool,
    pub over_temp: bool,
    pub imbalance_fault: bool,
    pub bmu_fail: bool,
    pub charge_inhibit: bool,
    pub min_cell_v: f64,
    pub max_cell_v: f64,
    pub spread_v: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BatteryMonitoringUnit {
    t_ov: u64,
    t_uv: u64,
    t_ot: u64,
    t_imb: u64,
    ov: bool,
    uv: bool,
    ot: bool,
    imb: bool,
}

fn us(s: f64) -> u64 {
    (s * 1.0e6).round() as u64
}

impl BatteryMonitoringUnit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step(&mut self, input: &BmuInput) -> BmuOutput {
        let n = input.cells.clamp(1, MAX_CELLS);
        let dt = input.dt_us;
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        for v in input.cell_v.iter().take(n) {
            lo = lo.min(*v);
            hi = hi.max(*v);
        }
        let spread = hi - lo;
        if hi > CELL_OV_V {
            self.t_ov = self.t_ov.saturating_add(dt);
        } else {
            self.t_ov = 0;
        }
        if lo < CELL_UV_CUTOFF_V {
            self.t_uv = self.t_uv.saturating_add(dt);
        } else {
            self.t_uv = 0;
        }
        if input.temp_c > OVER_TEMP_C {
            self.t_ot = self.t_ot.saturating_add(dt);
        } else if input.temp_c < APPROX_OVER_TEMP_RESET_C {
            self.t_ot = 0;
        }
        if spread > APPROX_IMBALANCE_FAULT_V {
            self.t_imb = self.t_imb.saturating_add(dt);
        } else {
            self.t_imb = 0;
        }
        if self.t_ov >= us(APPROX_CELL_OV_DELAY_S) {
            self.ov = true;
        }
        if self.t_uv >= us(APPROX_CELL_UV_DELAY_S) {
            self.uv = true;
        }
        if self.t_ot >= us(OVER_TEMP_DELAY_S) {
            self.ot = true;
        }
        if self.t_imb >= us(APPROX_IMBALANCE_DELAY_S) {
            self.imb = true;
        }
        if input.reset {
            if hi <= CELL_OV_V {
                self.ov = false;
            }
            if lo >= CELL_UV_CUTOFF_V {
                self.uv = false;
            }
            if input.temp_c < APPROX_OVER_TEMP_RESET_C {
                self.ot = false;
            }
            self.imb = false;
        }
        let mut balance = [false; MAX_CELLS];
        let mean = input.cell_v.iter().take(n).sum::<f64>() / n as f64;
        let can_balance = !self.ov
            && !self.ot
            && mean > APPROX_BALANCE_MIN_CELL_V - 0.3
            && input.current_a.abs() < APPROX_BALANCE_MAX_CURRENT_A
            && spread > APPROX_BALANCE_START_V;
        if can_balance {
            for (k, b) in balance.iter_mut().enumerate().take(n) {
                *b = input.cell_v[k] > lo + APPROX_BALANCE_MARGIN_V;
            }
        }
        let disconnect = self.ov || self.uv || self.ot;
        BmuOutput {
            balance,
            disconnect,
            over_voltage: self.ov,
            under_voltage: self.uv,
            over_temp: self.ot,
            imbalance_fault: self.imb,
            bmu_fail: input.bmu_internal_fail,
            charge_inhibit: self.ov || self.ot || input.bmu_internal_fail,
            min_cell_v: lo,
            max_cell_v: hi,
            spread_v: spread,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::battery::*;

    fn input(cell_v: [f64; MAX_CELLS], temp: f64) -> BmuInput {
        BmuInput {
            dt_us: 100_000,
            cells: 8,
            cell_v,
            temp_c: temp,
            current_a: 0.0,
            reset: false,
            bmu_internal_fail: false,
        }
    }

    fn flat(v: f64) -> [f64; MAX_CELLS] {
        [v; MAX_CELLS]
    }

    #[test]
    fn under_voltage_cutoff_after_delay_and_latch() {
        let mut m = BatteryMonitoringUnit::new();
        let mut v = flat(3.3);
        v[5] = 2.4;
        for _ in 0..9 {
            assert!(!m.step(&input(v, 25.0)).disconnect);
        }
        let o = m.step(&input(v, 25.0));
        assert!(o.disconnect && o.under_voltage);
        let o = m.step(&input(flat(3.3), 25.0));
        assert!(o.disconnect);
        let mut r = input(flat(3.3), 25.0);
        r.reset = true;
        assert!(!m.step(&r).disconnect);
    }

    #[test]
    fn under_voltage_reset_refused_while_low() {
        let mut m = BatteryMonitoringUnit::new();
        let mut i = input(flat(2.4), 25.0);
        for _ in 0..10 {
            m.step(&i);
        }
        i.reset = true;
        assert!(m.step(&i).disconnect);
    }

    #[test]
    fn over_voltage_cutoff_and_charge_inhibit() {
        let mut m = BatteryMonitoringUnit::new();
        let mut v = flat(4.0);
        v[0] = 4.3;
        for _ in 0..4 {
            assert!(!m.step(&input(v, 25.0)).over_voltage);
        }
        let o = m.step(&input(v, 25.0));
        assert!(o.over_voltage && o.charge_inhibit && o.disconnect);
    }

    #[test]
    fn over_temp_after_one_second() {
        let mut m = BatteryMonitoringUnit::new();
        for _ in 0..9 {
            assert!(!m.step(&input(flat(3.7), 71.0)).over_temp);
        }
        assert!(m.step(&input(flat(3.7), 71.0)).over_temp);
        let mut m3 = BatteryMonitoringUnit::new();
        for _ in 0..8 {
            m3.step(&input(flat(3.7), 71.0));
        }
        m3.step(&input(flat(3.7), 50.0));
        for _ in 0..8 {
            assert!(!m3.step(&input(flat(3.7), 71.0)).over_temp);
        }
    }

    #[test]
    fn imbalance_fault_and_bmu_fail() {
        let mut m = BatteryMonitoringUnit::new();
        let mut v = flat(3.7);
        v[3] = 3.4;
        for _ in 0..99 {
            assert!(!m.step(&input(v, 25.0)).imbalance_fault);
        }
        let o = m.step(&input(v, 25.0));
        assert!(o.imbalance_fault && !o.disconnect);
        let mut i = input(flat(3.7), 25.0);
        i.bmu_internal_fail = true;
        assert!(m.step(&i).bmu_fail);
    }

    #[test]
    fn balancing_converges_cells() {
        let mut b = BatteryPack::new(BatteryParams::new(8, 75.0), 11, 0.95, 25.0);
        b.cells[1].soc += 0.012;
        b.cells[6].soc += 0.015;
        let mut m = BatteryMonitoringUnit::new();
        let start = {
            let v: Vec<f64> = (0..8).map(|k| b.cell_ocv(k)).collect();
            v.iter().cloned().fold(0.0_f64, f64::max) - v.iter().cloned().fold(9.0_f64, f64::min)
        };
        let mut bal = [false; MAX_CELLS];
        let mut any = false;
        for _ in 0..5000 {
            let o = b.step(10_000_000, 0.0, 25.0, bal);
            let r = m.step(&BmuInput {
                dt_us: 10_000_000,
                cells: 8,
                cell_v: o.cell_v,
                temp_c: o.temp_c,
                current_a: 0.0,
                reset: false,
                bmu_internal_fail: false,
            });
            bal = r.balance;
            any |= bal.iter().any(|x| *x);
        }
        let v: Vec<f64> = (0..8).map(|k| b.cell_ocv(k)).collect();
        let end = v.iter().cloned().fold(0.0_f64, f64::max) - v.iter().cloned().fold(9.0_f64, f64::min);
        assert!(any);
        assert!(start > 0.02 && end < 0.0105, "{start} {end}");
    }

    #[test]
    fn discharge_to_cutoff_opens_contactor() {
        let mut b = BatteryPack::new(BatteryParams::new(8, 75.0), 5, 0.05, 0.0);
        let mut m = BatteryMonitoringUnit::new();
        let mut cut = false;
        let mut i_load = 300.0;
        for _ in 0..4000 {
            let o = b.step(1_000_000, i_load, 25.0, [false; MAX_CELLS]);
            let r = m.step(&BmuInput {
                dt_us: 1_000_000,
                cells: 8,
                cell_v: o.cell_v,
                temp_c: o.temp_c,
                current_a: i_load,
                reset: false,
                bmu_internal_fail: false,
            });
            if r.disconnect {
                cut = true;
                i_load = 0.0;
            }
        }
        assert!(cut);
        assert!(b.soc_mean() > 0.0);
    }
}
