use core::f64::consts::TAU;

pub const APPROX_PMG_MIN_FREQ_HZ: f64 = 200.0;
pub const APPROX_EXCITER_TAU_S: f64 = 0.03;
pub const APPROX_FIELD_TAU_S: f64 = 0.12;
pub const APPROX_IRON_LOSS_REF_W: f64 = 3500.0;
pub const APPROX_WINDAGE_W_AT_REF: f64 = 2500.0;
pub const APPROX_FIELD_LOSS_W_PER_A2: f64 = 9.0;
pub const APPROX_OIL_THERMAL_MASS_J_PER_K: f64 = 45_000.0;
pub const APPROX_OIL_COOLING_W_PER_K: f64 = 220.0;
pub const APPROX_MOTORING_EFFICIENCY: f64 = 0.9;
pub const VFSG_FREQ_RANGE_HZ: (f64, f64) = (360.0, 800.0);
pub const VFSG_THERMAL_DISCONNECT_F: f64 = 390.0;
pub const VFSG_OIL_NORMAL_F: (f64, f64) = (104.0, 265.0);

pub fn f_to_c(f: f64) -> f64 {
    (f - 32.0) / 1.8
}

pub fn c_to_f(c: f64) -> f64 {
    c * 1.8 + 32.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineKind {
    Vfsg,
    Asg,
    RatGen,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MachineParams {
    pub kind: MachineKind,
    pub pole_pairs: f64,
    pub pmg_pole_pairs: f64,
    pub rated_va: f64,
    pub nominal_phase_v: f64,
    pub ref_rpm: f64,
    pub i_sat_a: f64,
    pub i_field_nominal_a: f64,
    pub i_field_max_a: f64,
    pub i_exciter_max_a: f64,
    pub exciter_gain: f64,
    pub exciter_tau_s: f64,
    pub field_tau_s: f64,
    pub r_pu: f64,
    pub xs_pu: f64,
    pub pmg_v_per_rpm: f64,
    pub oil_thermal_mass_j_per_k: f64,
    pub oil_cooling_w_per_k: f64,
    pub thermal_disconnect_c: Option<f64>,
}

impl MachineParams {
    pub fn vfsg() -> Self {
        Self {
            kind: MachineKind::Vfsg,
            pole_pairs: 3.0,
            pmg_pole_pairs: 9.0,
            rated_va: 250_000.0,
            nominal_phase_v: 235.0,
            ref_rpm: 7200.0,
            i_sat_a: 12.0,
            i_field_nominal_a: 6.0,
            i_field_max_a: 20.0,
            i_exciter_max_a: 6.0,
            exciter_gain: 3.333,
            exciter_tau_s: APPROX_EXCITER_TAU_S,
            field_tau_s: APPROX_FIELD_TAU_S,
            r_pu: 0.02,
            xs_pu: 0.2,
            pmg_v_per_rpm: 0.012,
            oil_thermal_mass_j_per_k: APPROX_OIL_THERMAL_MASS_J_PER_K,
            oil_cooling_w_per_k: APPROX_OIL_COOLING_W_PER_K,
            thermal_disconnect_c: Some(f_to_c(VFSG_THERMAL_DISCONNECT_F)),
        }
    }

    pub fn asg() -> Self {
        Self {
            kind: MachineKind::Asg,
            pole_pairs: 2.0,
            pmg_pole_pairs: 9.0,
            ref_rpm: 10_800.0,
            pmg_v_per_rpm: 0.008,
            rated_va: 225_000.0,
            nominal_phase_v: 235.0,
            oil_thermal_mass_j_per_k: 30_000.0,
            oil_cooling_w_per_k: 90.0,
            thermal_disconnect_c: None,
            ..Self::vfsg()
        }
    }

    pub fn rat() -> Self {
        Self {
            kind: MachineKind::RatGen,
            pole_pairs: 3.0,
            pmg_pole_pairs: 9.0,
            rated_va: 10_000.0,
            nominal_phase_v: 230.0,
            ref_rpm: 8000.0,
            oil_thermal_mass_j_per_k: 6_000.0,
            oil_cooling_w_per_k: 25.0,
            thermal_disconnect_c: None,
            ..Self::vfsg()
        }
    }

    pub fn rated_phase_current_a(&self) -> f64 {
        self.rated_va / (3.0 * self.nominal_phase_v)
    }

    pub fn z_base_ohm(&self) -> f64 {
        self.nominal_phase_v / self.rated_phase_current_a()
    }

    pub fn r_ohm(&self) -> f64 {
        self.r_pu * self.z_base_ohm()
    }

    pub fn xs_ohm_at_ref(&self) -> f64 {
        self.xs_pu * self.z_base_ohm()
    }

    pub fn saturation(&self, field_a: f64) -> f64 {
        self.i_sat_a * (field_a.max(0.0) / self.i_sat_a).tanh()
    }

    pub fn emf_per_rpm(&self) -> f64 {
        self.nominal_phase_v / (self.ref_rpm * self.saturation(self.i_field_nominal_a))
    }

    pub fn open_circuit_emf(&self, rpm: f64, main_field_a: f64) -> f64 {
        self.emf_per_rpm() * rpm.max(0.0) * self.saturation(main_field_a)
    }

    pub fn electrical_freq_hz(&self, rpm: f64) -> f64 {
        rpm.max(0.0) / 60.0 * self.pole_pairs
    }

    pub fn pmg_freq_hz(&self, rpm: f64) -> f64 {
        rpm.max(0.0) / 60.0 * self.pmg_pole_pairs
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Thevenin {
    pub emf_v: f64,
    pub angle_rad: f64,
    pub r_ohm: f64,
    pub x_ohm: f64,
    pub freq_hz: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MachineInput {
    pub dt_us: u64,
    pub shaft_rpm: f64,
    pub exciter_current_cmd_a: f64,
    pub disconnect_cmd: bool,
    pub start_mode: bool,
    pub motor_shaft_power_w: f64,
    pub load_current_a: f64,
    pub load_pf_angle_rad: f64,
    pub oil_inlet_c: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MachineOutput {
    pub elec_freq_hz: f64,
    pub pmg_freq_hz: f64,
    pub pmg_phase_v: f64,
    pub gcu_power_available: bool,
    pub emf_phase_v: f64,
    pub terminal_phase_v: f64,
    pub thevenin: Thevenin,
    pub exciter_field_a: f64,
    pub main_field_a: f64,
    pub loss_w: f64,
    pub shaft_power_w: f64,
    pub oil_temp_c: f64,
    pub disconnected: bool,
    pub motoring_elec_power_w: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyncMachine {
    pub params: MachineParams,
    exciter_field_a: f64,
    main_field_a: f64,
    oil_temp_c: f64,
    angle_rad: f64,
    disconnected: bool,
}

impl SyncMachine {
    pub fn new(params: MachineParams, oil_temp_c: f64) -> Self {
        Self {
            params,
            exciter_field_a: 0.0,
            main_field_a: 0.0,
            oil_temp_c,
            angle_rad: 0.0,
            disconnected: false,
        }
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected
    }

    pub fn oil_temp_c(&self) -> f64 {
        self.oil_temp_c
    }

    pub fn main_field_a(&self) -> f64 {
        self.main_field_a
    }

    pub fn step(&mut self, input: &MachineInput) -> MachineOutput {
        let p = self.params;
        let dt = input.dt_us as f64 * 1.0e-6;
        if input.disconnect_cmd || p.thermal_disconnect_c.is_some_and(|t| self.oil_temp_c >= t) {
            self.disconnected = true;
        }
        let rpm = if self.disconnected {
            0.0
        } else {
            input.shaft_rpm.max(0.0)
        };
        let pmg_f = p.pmg_freq_hz(rpm);
        let pmg_v = p.pmg_v_per_rpm * rpm;
        let gcu_power = pmg_f >= APPROX_PMG_MIN_FREQ_HZ;
        let exc_cmd = if gcu_power && !input.start_mode {
            input.exciter_current_cmd_a.clamp(0.0, p.i_exciter_max_a)
        } else {
            0.0
        };
        let a_e = (dt / p.exciter_tau_s).min(1.0);
        self.exciter_field_a += (exc_cmd - self.exciter_field_a) * a_e;
        let speed_ratio = rpm / p.ref_rpm;
        let field_ss = (self.exciter_field_a * p.exciter_gain * speed_ratio).min(p.i_field_max_a);
        let a_f = (dt / p.field_tau_s).min(1.0);
        self.main_field_a += (field_ss - self.main_field_a) * a_f;
        let freq = p.electrical_freq_hz(rpm);
        self.angle_rad = (self.angle_rad + TAU * freq * dt) % TAU;
        let emf = if input.start_mode {
            0.0
        } else {
            p.open_circuit_emf(rpm, self.main_field_a)
        };
        let r = p.r_ohm();
        let x = p.xs_ohm_at_ref() * speed_ratio;
        let i = input.load_current_a.max(0.0);
        let (cp, sp) = (input.load_pf_angle_rad.cos(), input.load_pf_angle_rad.sin());
        let a = i * (r * cp + x * sp);
        let b = i * (x * cp - r * sp);
        let v_term = (((emf * emf - b * b).max(0.0)).sqrt() - a).max(0.0);
        let copper_w = 3.0 * i * i * r;
        let field_w = APPROX_FIELD_LOSS_W_PER_A2
            * (self.main_field_a * self.main_field_a + self.exciter_field_a * self.exciter_field_a);
        let core_w =
            APPROX_IRON_LOSS_REF_W * speed_ratio * speed_ratio * (emf / p.nominal_phase_v).powi(2).min(4.0);
        let windage_w = APPROX_WINDAGE_W_AT_REF * speed_ratio * speed_ratio;
        let loss_w = copper_w + field_w + core_w + windage_w;
        let elec_out_w = 3.0 * v_term * i * cp;
        let motoring_w = if input.start_mode {
            input.motor_shaft_power_w.max(0.0) / APPROX_MOTORING_EFFICIENCY + copper_w
        } else {
            0.0
        };
        let shaft_w = if input.start_mode {
            -input.motor_shaft_power_w.max(0.0)
        } else {
            elec_out_w + loss_w
        };
        let heat_w = if self.disconnected { 0.0 } else { loss_w };
        let cool_w = p.oil_cooling_w_per_k * (self.oil_temp_c - input.oil_inlet_c);
        self.oil_temp_c += (heat_w - cool_w) * dt / p.oil_thermal_mass_j_per_k;
        MachineOutput {
            elec_freq_hz: freq,
            pmg_freq_hz: pmg_f,
            pmg_phase_v: pmg_v,
            gcu_power_available: gcu_power,
            emf_phase_v: emf,
            terminal_phase_v: v_term,
            thevenin: Thevenin {
                emf_v: emf,
                angle_rad: self.angle_rad,
                r_ohm: r,
                x_ohm: x,
                freq_hz: freq,
            },
            exciter_field_a: self.exciter_field_a,
            main_field_a: self.main_field_a,
            loss_w: heat_w,
            shaft_power_w: shaft_w,
            oil_temp_c: self.oil_temp_c,
            disconnected: self.disconnected,
            motoring_elec_power_w: motoring_w,
        }
    }
}

#[cfg(test)]
mod thermal_disconnect_tests {
    use super::*;

    fn input(oil_c: f64) -> MachineInput {
        MachineInput {
            dt_us: 100_000,
            shaft_rpm: 7600.0,
            exciter_current_cmd_a: 1.0,
            disconnect_cmd: false,
            start_mode: false,
            motor_shaft_power_w: 0.0,
            load_current_a: 0.0,
            load_pf_angle_rad: 0.0,
            oil_inlet_c: oil_c,
        }
    }

    #[test]
    fn vfsg_solder_pellet_melts_at_390f_but_asg_has_none() {
        let mut v = SyncMachine::new(MachineParams::vfsg(), 190.0);
        let mut a = SyncMachine::new(MachineParams::asg(), 190.0);
        for _ in 0..50 {
            v.step(&input(190.0));
            a.step(&input(190.0));
        }
        assert!(!v.disconnected());
        for _ in 0..20_000 {
            v.step(&input(230.0));
            a.step(&input(230.0));
        }
        assert!(v.disconnected());
        assert!(!a.disconnected());
        assert!((c_to_f(MachineParams::vfsg().thermal_disconnect_c.unwrap()) - 390.0).abs() < 1e-9);
    }
}
