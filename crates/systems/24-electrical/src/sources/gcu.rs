pub const OV_TRIP_RESET_V: f64 = 239.0;
pub const OV_TIERS: [(f64, f64); 9] = [
    (320.0, 0.03),
    (310.0, 0.05),
    (300.0, 0.1),
    (290.0, 0.2),
    (280.0, 0.4),
    (270.0, 0.6),
    (260.0, 1.0),
    (250.0, 1.75),
    (240.0, 2.8),
];
pub const UV_SLOW_V: f64 = 216.0;
pub const UV_SLOW_DELAY_S: f64 = 4.5;
pub const UV_FAST_V: f64 = 214.5;
pub const UV_FAST_DELAY_S: f64 = 0.5;
pub const UV_RESET_V: f64 = 222.0;
pub const UV_INHIBIT_OC_VFSG_A: f64 = 620.0;
pub const UV_INHIBIT_OC_ASG_A: f64 = 399.0;
pub const UF_PMG_HZ: f64 = 1083.0;
pub const UF_DELAY_S: f64 = 4.0;
pub const OF_PMG_HZ_VFSG: f64 = 2397.0;
pub const OF_RESET_PMG_HZ_VFSG: f64 = 2391.0;
pub const OF_DELAY_S: f64 = 0.1;
pub const OF_PMG_HZ_ASG: f64 = 1998.0;
pub const OF_RESET_PMG_HZ_ASG: f64 = 1984.5;
pub const OF_DELAY_ASG_S: f64 = 2.1;
pub const OF2_PMG_HZ_ASG: f64 = 2065.5;
pub const OF2_RESET_PMG_HZ_ASG: f64 = 2052.0;
pub const OF2_DELAY_ASG_S: f64 = 0.1;
pub const UF_PMG_HZ_ASG: f64 = 1624.5;
pub const UF_RESET_PMG_HZ_ASG: f64 = 1638.0;
pub const UF_DELAY_ASG_S: f64 = 1.5;
pub const UF2_PMG_HZ_ASG: f64 = 1579.5;
pub const UF2_RESET_PMG_HZ_ASG: f64 = 1539.0;
pub const UF2_DELAY_ASG_S: f64 = 0.2;
pub const DIFF_THRESH_LOW_A: f64 = 30.0;
pub const DIFF_THRESH_HIGH_A: f64 = 45.0;
pub const DIFF_SPLIT_A: f64 = 450.0;
pub const DIFF_DELAY_S: f64 = 0.03;
pub const OC_VFSG_A: f64 = 375.0;
pub const OC_VFSG_RESET_A: f64 = 360.0;
pub const OC_ASG_A: f64 = 340.0;
pub const OC_ASG_RESET_A: f64 = 325.0;
pub const OC_MIN_DELAY_S: f64 = 5.1;
pub const OC_MAX_DELAY_S: f64 = 330.0;
pub const OC_FAST_RATIO_VFSG: f64 = 620.0 / 375.0;
pub const OC_FAST_RATIO_ASG: f64 = 399.0 / 340.0;
pub const OPEN_PHASE_LOW_A: f64 = 11.0;
pub const OPEN_PHASE_SPREAD_A: f64 = 55.0;
pub const OPEN_PHASE_DELAY_S: f64 = 4.0;
pub const PHASE_SEQ_DELAY_S: f64 = 0.1;
pub const GROUND_FAULT_A: f64 = 15.0;
pub const GROUND_FAULT_DELAY_S: f64 = 0.05;
pub const DC_CONTENT_A: f64 = 50.0;
pub const DC_CONTENT_RESET_A: f64 = 32.0;
pub const DC_CONTENT_DELAY_S: f64 = 0.175;
pub const SRD_DELAY_S: f64 = 4.0;
pub const SRD_CEILING_FRACTION: f64 = 0.98;
pub const ENGINE_IDLE_PMG_HZ: f64 = 1110.0;
pub const UNBALANCED_FEEDER_A: f64 = 45.0;
pub const FIELD_DRIVER_ERR_A: f64 = 1.0;
pub const FIELD_DRIVER_FAULT_S: f64 = 0.015;
pub const FIELD_DRIVER_VR_DISABLE_S: f64 = 0.01;
pub const FIELD_DRIVER_BREAKER_OPEN_S: f64 = 0.05;
pub const UNBALANCED_FEEDER_DELAY_S: f64 = 2.75;
pub const OVTPU_RESET_S: f64 = 0.2;
pub const OVTPU_UV_INHIBIT_S: f64 = 0.1;
pub const OVTPU_SRD_INHIBIT_S: f64 = 0.01;
pub const UV_ARM_V: f64 = 222.0;
pub const APPROX_KP_A_PER_V: f64 = 0.004;
pub const APPROX_KI_A_PER_V_S: f64 = 0.05;
pub const APPROX_SOFT_START_S: f64 = 1.5;
pub const APPROX_FEEDFORWARD_EXC_A: f64 = 1.7;
pub const APPROX_INTEGRATE_WINDOW_V: f64 = 12.0;
pub const APPROX_COMPOUND_EXC_A_PER_PU: f64 = 0.35;
pub const APPROX_INTEGRATE_OUTSIDE_FRACTION: f64 = 0.15;
pub const RAT_BREAKER_CLOSE_HZ: f64 = 380.0;
pub const RAT_REGULATION_BAND_V: f64 = 4.0;
pub const RAT_MAX_VA: f64 = 10_000.0;
pub const POWERED_TICK_US: u64 = 10_000;
pub const POWERED_TICK_LIMIT: u64 = 1 << 31;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GcuFault {
    None = 0,
    OverVoltage = 1,
    UnderVoltage = 2,
    OverFreq = 3,
    UnderFreq = 4,
    Differential = 5,
    OverCurrent = 6,
    OpenPhase = 7,
    PhaseSequence = 8,
    PoweredTimeOverflow = 9,
    GroundFault = 10,
    DcContent = 11,
    ShortedRotatingDiode = 12,
    UnbalancedFeeder = 13,
    FieldDriver = 14,
}

impl GcuFault {
    pub fn code(self) -> u16 {
        self as u16
    }

    pub fn mask(self) -> u16 {
        match self {
            GcuFault::None => 0,
            f => 1 << (f as u16 - 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GcuParams {
    pub is_asg: bool,
    pub regulated_v: f64,
    pub kp: f64,
    pub ki: f64,
    pub exciter_max_a: f64,
    pub feedforward_exc_a: f64,
    pub compound_exc_a_per_pu: f64,
    pub rated_a: f64,
    pub pmg_ref_hz: f64,
    pub uf_pmg_hz: f64,
    pub of_pmg_hz: f64,
    pub of_reset_pmg_hz: f64,
    pub oc_a: f64,
    pub oc_reset_a: f64,
    pub uv_inhibit_oc_a: f64,
    pub idle_pmg_hz: Option<f64>,
}

impl GcuParams {
    pub fn vfsg() -> Self {
        Self {
            is_asg: false,
            regulated_v: 235.0,
            kp: APPROX_KP_A_PER_V,
            ki: APPROX_KI_A_PER_V_S,
            exciter_max_a: 6.0,
            feedforward_exc_a: APPROX_FEEDFORWARD_EXC_A,
            compound_exc_a_per_pu: APPROX_COMPOUND_EXC_A_PER_PU,
            rated_a: 250_000.0 / (3.0 * 235.0),
            pmg_ref_hz: 1080.0,
            uf_pmg_hz: UF_PMG_HZ,
            of_pmg_hz: OF_PMG_HZ_VFSG,
            of_reset_pmg_hz: OF_RESET_PMG_HZ_VFSG,
            oc_a: OC_VFSG_A,
            oc_reset_a: OC_VFSG_RESET_A,
            uv_inhibit_oc_a: UV_INHIBIT_OC_VFSG_A,
            idle_pmg_hz: Some(ENGINE_IDLE_PMG_HZ),
        }
    }

    pub fn asg() -> Self {
        Self {
            is_asg: true,
            regulated_v: 235.0,
            rated_a: 225_000.0 / (3.0 * 235.0),
            pmg_ref_hz: 1620.0,
            uf_pmg_hz: UF_PMG_HZ_ASG,
            of_pmg_hz: OF_PMG_HZ_ASG,
            of_reset_pmg_hz: OF_RESET_PMG_HZ_ASG,
            oc_a: OC_ASG_A,
            oc_reset_a: OC_ASG_RESET_A,
            uv_inhibit_oc_a: UV_INHIBIT_OC_ASG_A,
            idle_pmg_hz: None,
            ..Self::vfsg()
        }
    }

    pub fn rat() -> Self {
        Self {
            regulated_v: 230.0,
            rated_a: 10_000.0 / (3.0 * 230.0),
            pmg_ref_hz: 1200.0,
            idle_pmg_hz: None,
            ..Self::vfsg()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GcuInput {
    pub dt_us: u64,
    pub pmg_available: bool,
    pub pmg_freq_hz: f64,
    pub gcs_on: bool,
    pub reset_cmd: bool,
    pub phase_v: [f64; 3],
    pub gen_i: [f64; 3],
    pub feeder_i: [f64; 3],
    pub gcb_closed: bool,
    pub engine_start_mode: bool,
    pub idle_valid: bool,
    pub engine_at_idle: bool,
    pub phase_seq_abc: bool,
    pub residual_a: f64,
    pub dc_content_a: [f64; 3],
    pub feeder_unbalance_a: f64,
    pub exciter_feedback_a: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GcuOutput {
    pub exciter_current_a: f64,
    pub vr_enabled: bool,
    pub field_off: bool,
    pub gcb_trip_cmd: bool,
    pub gcb_close_permit: bool,
    pub trip_latched: bool,
    pub trip_mask: u16,
    pub bite_code: u16,
    pub phase_seq_fault: bool,
    pub regulated_v: f64,
    pub powered_ticks: u64,
    pub voltage_established: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gcu {
    pub params: GcuParams,
    integ_a: f64,
    ramp_v: f64,
    latched_mask: u16,
    established: bool,
    powered_us: u64,
    prev_gcs: bool,
    t_ov: u64,
    t_uv_slow: u64,
    t_uv_fast: u64,
    t_uf: u64,
    uf_armed: bool,
    t_of: u64,
    t_uf2: u64,
    t_of2: u64,
    t_diff: u64,
    t_oc: u64,
    t_op: u64,
    t_seq: u64,
    ov_active: bool,
    of_active: bool,
    oc_active: bool,
    diff_active: bool,
    t_gf: u64,
    t_dc: u64,
    t_srd: u64,
    t_ub: u64,
    t_fd: u64,
    since_fd_us: Option<u64>,
    since_clear_us: u64,
    ovtpu_hold_us: u64,
    last_exc: f64,
}

fn accum(timer: &mut u64, active: bool, clear: bool, dt: u64) {
    if active {
        *timer = timer.saturating_add(dt);
    } else if clear {
        *timer = 0;
    }
}

fn us(s: f64) -> u64 {
    (s * 1.0e6).round() as u64
}

pub fn rat_breaker_close_permit(elec_freq_hz: f64, gcu: &GcuOutput) -> bool {
    elec_freq_hz >= RAT_BREAKER_CLOSE_HZ && gcu.vr_enabled && gcu.voltage_established
}

pub fn ov_delay_s(v: f64) -> Option<f64> {
    OV_TIERS.iter().find(|(t, _)| v >= *t).map(|(_, d)| *d)
}

impl Gcu {
    pub fn new(params: GcuParams) -> Self {
        Self {
            params,
            integ_a: 0.0,
            ramp_v: 0.0,
            latched_mask: 0,
            established: false,
            powered_us: 0,
            prev_gcs: false,
            t_ov: 0,
            t_uv_slow: 0,
            t_uv_fast: 0,
            t_uf: 0,
            uf_armed: false,
            t_of: 0,
            t_uf2: 0,
            t_of2: 0,
            t_diff: 0,
            t_oc: 0,
            t_op: 0,
            t_seq: 0,
            ov_active: false,
            of_active: false,
            oc_active: false,
            diff_active: false,
            t_gf: 0,
            t_dc: 0,
            t_srd: 0,
            t_ub: 0,
            t_fd: 0,
            since_fd_us: None,
            since_clear_us: u64::MAX,
            ovtpu_hold_us: 0,
            last_exc: 0.0,
        }
    }

    pub fn powered_time_us(&self) -> u64 {
        self.powered_us
    }

    pub fn set_powered_time_us(&mut self, us: u64) {
        self.powered_us = us;
    }

    pub fn generator_mode(&self) -> bool {
        self.uf_armed
    }

    pub fn latched_mask(&self) -> u16 {
        self.latched_mask
    }

    fn clear_timers(&mut self) {
        self.t_ov = 0;
        self.t_uv_slow = 0;
        self.t_uv_fast = 0;
        self.t_uf = 0;
        self.t_of = 0;
        self.t_uf2 = 0;
        self.t_of2 = 0;
        self.t_diff = 0;
        self.t_oc = 0;
        self.t_op = 0;
        self.ov_active = false;
        self.of_active = false;
        self.oc_active = false;
        self.diff_active = false;
        self.t_gf = 0;
        self.t_dc = 0;
        self.t_srd = 0;
        self.t_ub = 0;
        self.t_fd = 0;
    }

    fn at_idle(&self, input: &GcuInput) -> bool {
        match self.params.idle_pmg_hz {
            Some(hz) => input.pmg_freq_hz >= hz,
            None => input.engine_at_idle,
        }
    }

    pub fn step(&mut self, input: &GcuInput) -> GcuOutput {
        let p = self.params;
        let dt = input.dt_us;
        let dts = dt as f64 * 1.0e-6;
        let powered = input.pmg_available;
        if !powered {
            self.powered_us = 0;
            self.latched_mask = 0;
            self.integ_a = 0.0;
            self.ramp_v = 0.0;
            self.established = false;
            self.prev_gcs = false;
            self.t_seq = 0;
            self.uf_armed = false;
            self.clear_timers();
            return GcuOutput {
                field_off: true,
                gcb_trip_cmd: true,
                ..GcuOutput::default()
            };
        }
        self.powered_us = self.powered_us.saturating_add(dt);
        let ticks = self.powered_us / POWERED_TICK_US;
        let overflow = ticks >= POWERED_TICK_LIMIT;
        if overflow {
            self.latched_mask |= GcuFault::PoweredTimeOverflow.mask();
        }
        let gcs_edge_on = input.gcs_on && !self.prev_gcs;
        self.since_clear_us = self.since_clear_us.saturating_add(dt);
        if let Some(t) = self.since_fd_us.as_mut() {
            *t = t.saturating_add(dt);
        }
        self.ovtpu_hold_us = self.ovtpu_hold_us.saturating_sub(dt);
        if (!input.gcs_on || input.reset_cmd || gcs_edge_on) && !overflow {
            if self.latched_mask != 0 {
                if self.latched_mask & GcuFault::OverVoltage.mask() != 0 {
                    self.ovtpu_hold_us = us(OVTPU_RESET_S);
                }
                self.latched_mask = 0;
                self.since_clear_us = 0;
                self.since_fd_us = None;
            }
            if !input.gcs_on {
                self.established = false;
                self.integ_a = 0.0;
                self.clear_timers();
            }
        }
        self.prev_gcs = input.gcs_on;

        let v_hi = input.phase_v.iter().cloned().fold(f64::MIN, f64::max);
        let v_lo = input.phase_v.iter().cloned().fold(f64::MAX, f64::min);
        let v_avg = (input.phase_v[0] + input.phase_v[1] + input.phase_v[2]) / 3.0;
        let i_hi = input.gen_i.iter().cloned().fold(f64::MIN, f64::max);
        let mut sorted = input.gen_i;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

        let vr_enabled_pre = input.gcs_on && self.latched_mask == 0;

        if vr_enabled_pre {
            if v_lo >= UV_ARM_V {
                self.established = true;
            }
            let ov_d = ov_delay_s(v_hi);
            if v_hi < OV_TRIP_RESET_V {
                self.t_ov = 0;
                self.ov_active = false;
            } else if let Some(d) = ov_d {
                self.ov_active = true;
                self.t_ov = self.t_ov.saturating_add(dt);
                if self.t_ov >= us(d) {
                    self.latched_mask |= GcuFault::OverVoltage.mask();
                }
            }
            let uv_inhibit =
                i_hi > p.uv_inhibit_oc_a || !self.established || self.since_clear_us < us(OVTPU_UV_INHIBIT_S);
            accum(
                &mut self.t_uv_slow,
                !uv_inhibit && v_lo <= UV_SLOW_V,
                v_lo >= UV_RESET_V || uv_inhibit,
                dt,
            );
            accum(
                &mut self.t_uv_fast,
                !uv_inhibit && v_lo < UV_FAST_V,
                v_lo >= UV_RESET_V || uv_inhibit,
                dt,
            );
            if self.t_uv_slow >= us(UV_SLOW_DELAY_S) || self.t_uv_fast >= us(UV_FAST_DELAY_S) {
                self.latched_mask |= GcuFault::UnderVoltage.mask();
            }
            if input.engine_start_mode {
                self.uf_armed = false;
            } else if input.idle_valid && self.at_idle(input) {
                self.uf_armed = true;
            }
            let uf_cond = input.pmg_freq_hz < p.uf_pmg_hz && !input.engine_start_mode && self.uf_armed;
            let (uf_delay, uf_reset) = if p.is_asg {
                (UF_DELAY_ASG_S, UF_RESET_PMG_HZ_ASG)
            } else {
                (UF_DELAY_S, p.uf_pmg_hz)
            };
            accum(&mut self.t_uf, uf_cond, input.pmg_freq_hz >= uf_reset, dt);
            if self.t_uf >= us(uf_delay) {
                self.latched_mask |= GcuFault::UnderFreq.mask();
            }
            let of_cond = input.pmg_freq_hz > p.of_pmg_hz;
            accum(
                &mut self.t_of,
                of_cond,
                input.pmg_freq_hz <= p.of_reset_pmg_hz,
                dt,
            );
            self.of_active = of_cond;
            let of_delay = if p.is_asg { OF_DELAY_ASG_S } else { OF_DELAY_S };
            if self.t_of >= us(of_delay) {
                self.latched_mask |= GcuFault::OverFreq.mask();
            }
            if p.is_asg {
                let uf2_cond =
                    input.pmg_freq_hz < UF2_PMG_HZ_ASG && !input.engine_start_mode && self.uf_armed;
                accum(
                    &mut self.t_uf2,
                    uf2_cond,
                    input.pmg_freq_hz >= UF2_RESET_PMG_HZ_ASG,
                    dt,
                );
                if self.t_uf2 >= us(UF2_DELAY_ASG_S) {
                    self.latched_mask |= GcuFault::UnderFreq.mask();
                }
                let of2_cond = input.pmg_freq_hz > OF2_PMG_HZ_ASG;
                accum(
                    &mut self.t_of2,
                    of2_cond,
                    input.pmg_freq_hz <= OF2_RESET_PMG_HZ_ASG,
                    dt,
                );
                if self.t_of2 >= us(OF2_DELAY_ASG_S) {
                    self.latched_mask |= GcuFault::OverFreq.mask();
                }
            }
            let mut diff_max = 0.0_f64;
            for k in 0..3 {
                diff_max = diff_max.max((input.gen_i[k] - input.feeder_i[k]).abs());
            }
            let diff_thr = if i_hi >= DIFF_SPLIT_A {
                DIFF_THRESH_HIGH_A
            } else {
                DIFF_THRESH_LOW_A
            };
            let diff_cond = input.gcb_closed && diff_max > diff_thr;
            accum(&mut self.t_diff, diff_cond, !diff_cond, dt);
            self.diff_active = diff_cond;
            if self.t_diff >= us(DIFF_DELAY_S) {
                self.latched_mask |= GcuFault::Differential.mask();
            }
            let oc_cond = i_hi > p.oc_a;
            let fast_ratio = if p.is_asg {
                OC_FAST_RATIO_ASG
            } else {
                OC_FAST_RATIO_VFSG
            };
            let frac = ((i_hi - p.oc_a) / (p.oc_a * (fast_ratio - 1.0))).clamp(0.0, 1.0);
            let oc_delay = OC_MAX_DELAY_S + (OC_MIN_DELAY_S - OC_MAX_DELAY_S) * frac;
            accum(&mut self.t_oc, oc_cond, i_hi < p.oc_reset_a, dt);
            self.oc_active = oc_cond;
            if self.t_oc >= us(oc_delay) {
                self.latched_mask |= GcuFault::OverCurrent.mask();
            }
            let gf_cond = input.residual_a > GROUND_FAULT_A;
            accum(&mut self.t_gf, gf_cond, !gf_cond, dt);
            if self.t_gf >= us(GROUND_FAULT_DELAY_S) {
                self.latched_mask |= GcuFault::GroundFault.mask();
            }
            let dc_hi = input.dc_content_a.iter().cloned().fold(0.0, f64::max);
            accum(
                &mut self.t_dc,
                dc_hi > DC_CONTENT_A,
                dc_hi < DC_CONTENT_RESET_A,
                dt,
            );
            if self.t_dc >= us(DC_CONTENT_DELAY_S) {
                self.latched_mask |= GcuFault::DcContent.mask();
            }
            let srd_cond = self.since_clear_us >= us(OVTPU_SRD_INHIBIT_S)
                && self.established
                && input.gcb_closed
                && self.ramp_v >= p.regulated_v
                && self.last_exc >= SRD_CEILING_FRACTION * p.exciter_max_a
                && i_hi < p.rated_a;
            accum(&mut self.t_srd, srd_cond, !srd_cond, dt);
            if self.t_srd >= us(SRD_DELAY_S) {
                self.latched_mask |= GcuFault::ShortedRotatingDiode.mask();
            }
            let fd_cond = input
                .exciter_feedback_a
                .is_some_and(|fb| (fb - self.last_exc).abs() > FIELD_DRIVER_ERR_A);
            accum(&mut self.t_fd, fd_cond, !fd_cond, dt);
            if self.t_fd >= us(FIELD_DRIVER_FAULT_S) && self.latched_mask & GcuFault::FieldDriver.mask() == 0
            {
                self.latched_mask |= GcuFault::FieldDriver.mask();
                self.since_fd_us = Some(0);
            }
            let ub_cond = input.gcb_closed && input.feeder_unbalance_a > UNBALANCED_FEEDER_A;
            accum(&mut self.t_ub, ub_cond, !ub_cond, dt);
            if self.t_ub >= us(UNBALANCED_FEEDER_DELAY_S) {
                self.latched_mask |= GcuFault::UnbalancedFeeder.mask();
            }
            let op_cond = input.gcb_closed
                && !oc_cond
                && sorted[0] < OPEN_PHASE_LOW_A
                && sorted[1] - sorted[0] > OPEN_PHASE_SPREAD_A;
            accum(&mut self.t_op, op_cond, !op_cond, dt);
            if self.t_op >= us(OPEN_PHASE_DELAY_S) {
                self.latched_mask |= GcuFault::OpenPhase.mask();
            }
        }
        accum(&mut self.t_seq, !input.phase_seq_abc, input.phase_seq_abc, dt);
        let seq_fault = self.t_seq >= us(PHASE_SEQ_DELAY_S);

        let tripped = self.latched_mask != 0;
        let enabled = input.gcs_on && !tripped && self.ovtpu_hold_us == 0;
        let fd_only = self.latched_mask == GcuFault::FieldDriver.mask();
        let fd_age = self.since_fd_us.unwrap_or(u64::MAX);
        let mut exc = 0.0;
        if enabled {
            self.ramp_v = (self.ramp_v + p.regulated_v * dts / APPROX_SOFT_START_S).min(p.regulated_v);
            let err = self.ramp_v - v_avg;
            let sched = (p.pmg_ref_hz / input.pmg_freq_hz.max(p.pmg_ref_hz * 0.5)).powi(2);
            let (kp, ki) = (p.kp * sched, p.ki * sched);
            let i_avg = (input.gen_i[0] + input.gen_i[1] + input.gen_i[2]) / 3.0;
            let feedforward = (p.feedforward_exc_a * self.ramp_v / p.regulated_v
                + p.compound_exc_a_per_pu * i_avg / p.rated_a)
                * sched;
            let span = p.exciter_max_a;
            let unclamped = feedforward + kp * err + self.integ_a;
            let saturated = (unclamped >= span && err > 0.0) || (unclamped <= 0.0 && err < 0.0);
            if !saturated && self.ramp_v >= p.regulated_v {
                let rate = if err.abs() < APPROX_INTEGRATE_WINDOW_V {
                    1.0
                } else {
                    APPROX_INTEGRATE_OUTSIDE_FRACTION
                };
                self.integ_a = (self.integ_a + ki * rate * err * dts).clamp(-span, span);
            }
            exc = (feedforward + kp * err + self.integ_a).clamp(0.0, span);
        } else {
            self.integ_a = 0.0;
            self.ramp_v = v_avg;
        }
        self.last_exc = exc;
        let first = (0..16u16)
            .find(|b| self.latched_mask & (1 << b) != 0)
            .map(|b| b + 1)
            .unwrap_or(if seq_fault {
                GcuFault::PhaseSequence.code()
            } else {
                0
            });
        let field_off = if fd_only {
            fd_age >= us(FIELD_DRIVER_VR_DISABLE_S)
        } else {
            !enabled
        };
        let gcb_trip_cmd = if fd_only {
            fd_age >= us(FIELD_DRIVER_BREAKER_OPEN_S)
        } else {
            tripped
        };
        GcuOutput {
            exciter_current_a: exc,
            vr_enabled: enabled,
            field_off,
            gcb_trip_cmd,
            gcb_close_permit: enabled && !seq_fault && self.established,
            trip_latched: tripped,
            trip_mask: self.latched_mask,
            bite_code: first,
            phase_seq_fault: seq_fault,
            regulated_v: v_avg,
            powered_ticks: ticks,
            voltage_established: self.established,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::machine::*;

    const DT: u64 = 10_000;

    fn base_input() -> GcuInput {
        GcuInput {
            dt_us: DT,
            pmg_available: true,
            pmg_freq_hz: 2000.0,
            gcs_on: true,
            phase_v: [235.0; 3],
            gen_i: [100.0; 3],
            feeder_i: [100.0; 3],
            gcb_closed: true,
            idle_valid: true,
            engine_at_idle: false,
            phase_seq_abc: true,
            ..GcuInput::default()
        }
    }

    fn established(g: &mut Gcu) {
        let i = base_input();
        g.step(&i);
    }

    fn run_until_trip(g: &mut Gcu, input: &GcuInput, max_steps: usize) -> Option<usize> {
        for n in 1..=max_steps {
            let o = g.step(input);
            if o.trip_latched {
                return Some(n);
            }
        }
        None
    }

    fn loop_sim(
        params: MachineParams,
        gp: GcuParams,
        rpm: f64,
        load: &dyn Fn(f64) -> f64,
        secs: f64,
    ) -> (SyncMachine, Vec<(f64, f64)>) {
        let mut m = SyncMachine::new(params, 60.0);
        let mut g = Gcu::new(gp);
        let mut exc = 0.0;
        let mut vt = 0.0;
        let mut log = Vec::new();
        let steps = (secs * 100.0) as usize;
        for n in 0..steps {
            let t = n as f64 * 0.01;
            let mo = m.step(&MachineInput {
                dt_us: DT,
                shaft_rpm: rpm,
                exciter_current_cmd_a: exc,
                load_current_a: load(t),
                load_pf_angle_rad: 0.45,
                oil_inlet_c: 60.0,
                ..MachineInput::default()
            });
            vt = mo.terminal_phase_v;
            let go = g.step(&GcuInput {
                dt_us: DT,
                pmg_available: mo.gcu_power_available,
                pmg_freq_hz: mo.pmg_freq_hz,
                gcs_on: true,
                phase_v: [vt; 3],
                gen_i: [load(t); 3],
                feeder_i: [load(t); 3],
                gcb_closed: true,
                idle_valid: true,
                phase_seq_abc: true,
                ..GcuInput::default()
            });
            exc = go.exciter_current_a;
            log.push((t, vt));
        }
        let _ = vt;
        (m, log)
    }

    #[test]
    fn no_load_emf_scales_with_speed() {
        let p = MachineParams::vfsg();
        let e1 = p.open_circuit_emf(7200.0, 6.0);
        let e2 = p.open_circuit_emf(14400.0, 6.0);
        assert!((e1 - 235.0).abs() < 0.01);
        assert!((e2 / e1 - 2.0).abs() < 1e-9);
        assert!(p.open_circuit_emf(7200.0, 20.0) < 235.0 * 2.2);
        assert!(p.open_circuit_emf(7200.0, 12.0) / p.open_circuit_emf(7200.0, 6.0) < 2.0);
    }

    #[test]
    fn frequency_mapping_matches_facts() {
        let p = MachineParams::vfsg();
        assert!((p.electrical_freq_hz(7200.0) - 360.0).abs() < 1e-9);
        assert!((p.pmg_freq_hz(7200.0) - 1080.0).abs() < 1e-9);
        assert!((p.electrical_freq_hz(16000.0) - 800.0).abs() < 1e-9);
        let a = MachineParams::asg();
        assert!((a.pmg_freq_hz(13_230.0) - 1984.5).abs() < 1e-6);
    }

    #[test]
    fn regulation_holds_under_load_steps() {
        let load = |t: f64| {
            if t < 3.0 {
                50.0
            } else if t < 6.0 {
                200.0
            } else {
                120.0
            }
        };
        let (_, log) = loop_sim(MachineParams::vfsg(), GcuParams::vfsg(), 9000.0, &load, 9.0);
        let at = |t: f64| log.iter().find(|(x, _)| *x >= t).map(|(_, v)| *v).unwrap();
        assert!((at(2.9) - 235.0).abs() < 2.0, "{}", at(2.9));
        assert!((at(5.9) - 235.0).abs() < 2.0, "{}", at(5.9));
        assert!((at(8.9) - 235.0).abs() < 2.0, "{}", at(8.9));
        let dip = log
            .iter()
            .filter(|(t, _)| *t > 3.0 && *t < 3.5)
            .map(|(_, v)| *v)
            .fold(f64::MAX, f64::min);
        assert!(dip > 150.0, "{dip}");
    }

    #[test]
    fn regulation_holds_over_speed_range() {
        for rpm in [7200.0, 11000.0, 15900.0] {
            let (_, log) = loop_sim(MachineParams::vfsg(), GcuParams::vfsg(), rpm, &|_| 200.0, 8.0);
            let v = log.last().unwrap().1;
            assert!((v - 235.0).abs() < 2.0, "{rpm}: {v}");
        }
    }

    #[test]
    fn rat_breaker_needs_380hz() {
        let o = GcuOutput {
            vr_enabled: true,
            voltage_established: true,
            ..GcuOutput::default()
        };
        assert!(!rat_breaker_close_permit(379.9, &o));
        assert!(rat_breaker_close_permit(380.0, &o));
        assert!(!rat_breaker_close_permit(400.0, &GcuOutput::default()));
    }

    #[test]
    fn rat_regulates_230() {
        let (_, log) = loop_sim(MachineParams::rat(), GcuParams::rat(), 9000.0, &|_| 20.0, 8.0);
        assert!((log.last().unwrap().1 - 230.0).abs() < RAT_REGULATION_BAND_V);
    }

    #[test]
    fn disconnect_is_one_way() {
        let mut m = SyncMachine::new(MachineParams::vfsg(), 60.0);
        let inp = MachineInput {
            dt_us: DT,
            shaft_rpm: 9000.0,
            exciter_current_cmd_a: 3.0,
            oil_inlet_c: 60.0,
            ..MachineInput::default()
        };
        for _ in 0..100 {
            m.step(&inp);
        }
        assert!(m.main_field_a() > 1.0);
        m.step(&MachineInput {
            disconnect_cmd: true,
            ..inp
        });
        for _ in 0..200 {
            let o = m.step(&inp);
            assert!(o.disconnected);
            assert_eq!(o.elec_freq_hz, 0.0);
            assert_eq!(o.emf_phase_v, 0.0);
        }
    }

    #[test]
    fn losses_heat_oil() {
        let mut m = SyncMachine::new(MachineParams::vfsg(), 40.0);
        let mut last = 0.0;
        for _ in 0..6000 {
            let o = m.step(&MachineInput {
                dt_us: DT,
                shaft_rpm: 9000.0,
                exciter_current_cmd_a: 2.5,
                load_current_a: 300.0,
                load_pf_angle_rad: 0.45,
                oil_inlet_c: 40.0,
                ..MachineInput::default()
            });
            last = o.oil_temp_c;
            assert!(o.loss_w > 0.0);
        }
        assert!(last > 45.0 && last < 200.0, "{last}");
    }

    #[test]
    fn motoring_draws_power() {
        let mut m = SyncMachine::new(MachineParams::vfsg(), 40.0);
        let o = m.step(&MachineInput {
            dt_us: DT,
            shaft_rpm: 3000.0,
            start_mode: true,
            motor_shaft_power_w: 60_000.0,
            load_current_a: 150.0,
            oil_inlet_c: 40.0,
            ..MachineInput::default()
        });
        assert!(o.motoring_elec_power_w > 66_000.0);
        assert_eq!(o.emf_phase_v, 0.0);
    }

    #[test]
    fn thevenin_angle_advances() {
        let mut m = SyncMachine::new(MachineParams::vfsg(), 40.0);
        let o1 = m.step(&MachineInput {
            dt_us: 1000,
            shaft_rpm: 7200.0,
            ..MachineInput::default()
        });
        assert!((o1.thevenin.angle_rad - core::f64::consts::TAU * 360.0 * 0.001).abs() < 1e-9);
        assert!(o1.thevenin.r_ohm > 0.0 && o1.thevenin.x_ohm > 0.0);
    }

    #[test]
    fn overvoltage_inverse_time() {
        for (v, d) in [
            (320.0, 0.03),
            (310.0, 0.05),
            (300.0, 0.1),
            (290.0, 0.2),
            (280.0, 0.4),
            (270.0, 0.6),
            (260.0, 1.0),
            (250.0, 1.75),
            (245.0, 2.8),
            (240.0, 2.8),
        ] {
            let mut g = Gcu::new(GcuParams::vfsg());
            established(&mut g);
            let mut i = base_input();
            i.phase_v = [v, 235.0, 235.0];
            i.dt_us = 1000;
            let n = run_until_trip(&mut g, &i, 4000).unwrap();
            let t = n as f64 * 0.001;
            assert!(t >= d - 1e-9 && t < d + 0.0021, "{v}: {t}");
        }
    }

    #[test]
    fn overvoltage_below_threshold_no_trip() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.phase_v = [239.5, 235.0, 235.0];
        assert!(run_until_trip(&mut g, &i, 1000).is_none());
    }

    #[test]
    fn undervoltage_slow_trips_at_4_5s() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.phase_v = [235.0, 215.5, 235.0];
        for _ in 0..449 {
            assert!(!g.step(&i).trip_latched);
        }
        assert!(g.step(&i).trip_latched);
        assert_eq!(g.latched_mask(), GcuFault::UnderVoltage.mask());
    }

    #[test]
    fn undervoltage_fast_trips_at_0_5s() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.phase_v = [235.0, 200.0, 235.0];
        for _ in 0..49 {
            assert!(!g.step(&i).trip_latched);
        }
        assert!(g.step(&i).trip_latched);
    }

    #[test]
    fn undervoltage_inhibited_by_overcurrent() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.phase_v = [235.0, 200.0, 235.0];
        i.gen_i = [650.0; 3];
        i.feeder_i = [650.0; 3];
        for _ in 0..100 {
            let o = g.step(&i);
            assert_eq!(o.trip_mask & GcuFault::UnderVoltage.mask(), 0);
        }
    }

    #[test]
    fn undervoltage_not_armed_before_established() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        i.phase_v = [50.0; 3];
        assert!(run_until_trip(&mut g, &i, 1000).is_none());
    }

    #[test]
    fn underfrequency_trips_after_4s_only_once_pmg_has_reached_idle() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        i.pmg_freq_hz = 1000.0;
        assert!(run_until_trip(&mut g, &i, 1000).is_none());
        i.pmg_freq_hz = ENGINE_IDLE_PMG_HZ - 1.0;
        assert!(run_until_trip(&mut g, &i, 100).is_none());
        i.pmg_freq_hz = ENGINE_IDLE_PMG_HZ;
        g.step(&i);
        i.pmg_freq_hz = 1000.0;
        for _ in 0..399 {
            assert!(!g.step(&i).trip_latched);
        }
        assert!(g.step(&i).trip_latched);
        let mut g2 = Gcu::new(GcuParams::vfsg());
        established(&mut g2);
        i.engine_start_mode = true;
        assert!(run_until_trip(&mut g2, &i, 1000).is_none());
    }

    #[test]
    fn field_driver_fault_sets_in_15ms_disables_field_in_10ms_and_opens_breaker_in_50ms() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        for _ in 0..300 {
            let o = g.step(&i);
            i.exciter_feedback_a = Some(o.exciter_current_a);
        }
        i.exciter_feedback_a = Some(GcuParams::vfsg().exciter_max_a);
        let mut set_at = None;
        let mut field_off_at = None;
        let mut trip_at = None;
        for n in 1..=20u64 {
            let o = g.step(&i);
            if set_at.is_none() && g.latched_mask() & GcuFault::FieldDriver.mask() != 0 {
                set_at = Some(n);
            }
            if field_off_at.is_none() && o.field_off {
                field_off_at = Some(n);
            }
            if trip_at.is_none() && o.gcb_trip_cmd {
                trip_at = Some(n);
            }
        }
        let s = set_at.expect("fault set");
        assert_eq!(s, 2, "15 ms at 10 ms steps sets on the second step");
        assert_eq!(field_off_at, Some(s + 1));
        assert_eq!(trip_at, Some(s + 5));
    }

    #[test]
    fn unbalanced_parallel_feeder_trips_after_2_75s() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        i.feeder_unbalance_a = UNBALANCED_FEEDER_A + 1.0;
        assert_eq!(run_until_trip(&mut g, &i, 1000), Some(275));
        assert_ne!(g.latched_mask() & GcuFault::UnbalancedFeeder.mask(), 0);
        let mut g = Gcu::new(GcuParams::vfsg());
        i.feeder_unbalance_a = UNBALANCED_FEEDER_A - 1.0;
        assert!(run_until_trip(&mut g, &i, 1000).is_none());
    }

    #[test]
    fn ground_fault_trips_on_15a_residual_after_50ms() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        i.residual_a = GROUND_FAULT_A - 1.0;
        assert!(run_until_trip(&mut g, &i, 500).is_none());
        let mut g = Gcu::new(GcuParams::vfsg());
        i.residual_a = GROUND_FAULT_A + 1.0;
        assert_eq!(run_until_trip(&mut g, &i, 500), Some(5));
        assert_ne!(g.latched_mask() & GcuFault::GroundFault.mask(), 0);
    }

    #[test]
    fn dc_content_trips_after_175ms_and_resets_below_32a() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        i.dc_content_a = [DC_CONTENT_A + 5.0, 0.0, 0.0];
        for _ in 0..10 {
            assert!(!g.step(&i).trip_latched);
        }
        i.dc_content_a = [40.0, 0.0, 0.0];
        for _ in 0..7 {
            assert!(!g.step(&i).trip_latched);
        }
        i.dc_content_a = [DC_CONTENT_A + 5.0, 0.0, 0.0];
        for _ in 0..7 {
            assert!(!g.step(&i).trip_latched);
        }
        assert!(g.step(&i).trip_latched);
        let mut g = Gcu::new(GcuParams::vfsg());
        i.dc_content_a = [DC_CONTENT_A + 5.0, 0.0, 0.0];
        for _ in 0..10 {
            g.step(&i);
        }
        i.dc_content_a = [DC_CONTENT_RESET_A - 1.0, 0.0, 0.0];
        g.step(&i);
        i.dc_content_a = [DC_CONTENT_A + 5.0, 0.0, 0.0];
        for _ in 0..17 {
            assert!(!g.step(&i).trip_latched);
        }
        assert!(g.step(&i).trip_latched);
    }

    #[test]
    fn shorted_rotating_diode_trips_when_exciter_pinned_at_normal_load_for_4s() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        for _ in 0..300 {
            g.step(&i);
        }
        i.pmg_freq_hz = ENGINE_IDLE_PMG_HZ;
        i.phase_v = [225.0; 3];
        let ceiling = SRD_CEILING_FRACTION * GcuParams::vfsg().exciter_max_a;
        let mut pinned = false;
        for _ in 0..5000 {
            if g.step(&i).exciter_current_a >= ceiling {
                pinned = true;
                break;
            }
        }
        assert!(pinned);
        let n = run_until_trip(&mut g, &i, 1000).expect("srd trip");
        assert!((399..=401).contains(&n), "{n}");
        assert_ne!(g.latched_mask() & GcuFault::ShortedRotatingDiode.mask(), 0);
    }

    #[test]
    fn ovtpu_holds_field_off_200ms_after_overvoltage_reset() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let mut i = base_input();
        i.phase_v = [330.0; 3];
        assert!(run_until_trip(&mut g, &i, 100).is_some());
        i.phase_v = [235.0; 3];
        i.reset_cmd = true;
        let o = g.step(&i);
        assert!(!o.trip_latched && !o.vr_enabled);
        i.reset_cmd = false;
        for _ in 0..19 {
            assert!(!g.step(&i).vr_enabled);
        }
        assert!(g.step(&i).vr_enabled);
    }

    #[test]
    fn overfrequency_trips_in_100ms() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.pmg_freq_hz = 2400.0;
        for _ in 0..9 {
            assert!(!g.step(&i).trip_latched);
        }
        assert!(g.step(&i).trip_latched);
        let mut g2 = Gcu::new(GcuParams::vfsg());
        established(&mut g2);
        i.pmg_freq_hz = 2390.0;
        assert!(run_until_trip(&mut g2, &i, 500).is_none());
    }

    #[test]
    fn differential_trips_in_30ms() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.feeder_i = [100.0, 60.0, 100.0];
        assert!(!g.step(&i).trip_latched);
        assert!(!g.step(&i).trip_latched);
        let o = g.step(&i);
        assert!(o.trip_latched && o.gcb_trip_cmd && o.field_off);
        assert_eq!(o.bite_code, GcuFault::Differential.code());
        let mut g2 = Gcu::new(GcuParams::vfsg());
        established(&mut g2);
        i.feeder_i = [100.0, 71.0, 100.0];
        assert!(run_until_trip(&mut g2, &i, 100).is_none());
        let mut g3 = Gcu::new(GcuParams::vfsg());
        established(&mut g3);
        i.gen_i = [500.0; 3];
        i.feeder_i = [500.0, 460.0, 500.0];
        assert!(run_until_trip(&mut g3, &i, 100).is_none());
        i.feeder_i = [500.0, 450.0, 500.0];
        assert!(run_until_trip(&mut g3, &i, 100).is_some());
    }

    #[test]
    fn overcurrent_inverse_time_bounds() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.gen_i = [620.0, 100.0, 100.0];
        i.feeder_i = i.gen_i;
        i.dt_us = 1000;
        let n = run_until_trip(&mut g, &i, 20_000).unwrap();
        assert!((n as f64 * 0.001 - 5.1).abs() < 0.01);
        let mut g2 = Gcu::new(GcuParams::vfsg());
        established(&mut g2);
        i.gen_i = [376.0, 100.0, 100.0];
        i.feeder_i = i.gen_i;
        i.dt_us = 100_000;
        let n2 = run_until_trip(&mut g2, &i, 5000).unwrap();
        assert!(n2 as f64 * 0.1 > 250.0 && n2 as f64 * 0.1 <= 330.5);
        let mut g3 = Gcu::new(GcuParams::vfsg());
        established(&mut g3);
        i.gen_i = [374.0, 100.0, 100.0];
        i.feeder_i = i.gen_i;
        assert!(run_until_trip(&mut g3, &i, 5000).is_none());
    }

    #[test]
    fn asg_overcurrent_threshold() {
        let mut g = Gcu::new(GcuParams::asg());
        established(&mut g);
        let mut i = base_input();
        i.gen_i = [345.0, 100.0, 100.0];
        i.feeder_i = i.gen_i;
        i.pmg_freq_hz = 1700.0;
        i.dt_us = 100_000;
        assert!(run_until_trip(&mut g, &i, 5000).is_some());
        let mut g2 = Gcu::new(GcuParams::asg());
        established(&mut g2);
        i.gen_i = [399.0, 100.0, 100.0];
        i.feeder_i = i.gen_i;
        i.dt_us = 1000;
        let n = run_until_trip(&mut g2, &i, 20_000).unwrap();
        assert!((n as f64 * 0.001 - 5.1).abs() < 0.01);
    }

    #[test]
    fn asg_frequency_thresholds() {
        let mut g = Gcu::new(GcuParams::asg());
        established(&mut g);
        let mut i = base_input();
        i.dt_us = 1000;
        i.pmg_freq_hz = 2000.0;
        let n = run_until_trip(&mut g, &i, 5000).unwrap();
        assert!((n as f64 * 0.001 - 2.1).abs() < 0.012);
        let mut g2 = Gcu::new(GcuParams::asg());
        established(&mut g2);
        i.pmg_freq_hz = 2066.0;
        let n2 = run_until_trip(&mut g2, &i, 5000).unwrap();
        assert!((n2 as f64 * 0.001 - 0.1).abs() < 0.002);
        let mut g3 = Gcu::new(GcuParams::asg());
        established(&mut g3);
        i.pmg_freq_hz = 1990.0;
        assert!(run_until_trip(&mut g3, &i, 5000).is_none());
        let mut g4 = Gcu::new(GcuParams::asg());
        established(&mut g4);
        i.engine_at_idle = true;
        i.pmg_freq_hz = 1620.0;
        let n4 = run_until_trip(&mut g4, &i, 5000).unwrap();
        assert!((n4 as f64 * 0.001 - 1.5).abs() < 0.002);
        let mut g5 = Gcu::new(GcuParams::asg());
        established(&mut g5);
        i.pmg_freq_hz = 1575.0;
        let n5 = run_until_trip(&mut g5, &i, 5000).unwrap();
        assert!((n5 as f64 * 0.001 - 0.2).abs() < 0.002);
        assert_eq!(GcuParams::asg().of_reset_pmg_hz, 1984.5);
        assert_eq!(GcuParams::asg().uf_pmg_hz, 1624.5);
    }

    #[test]
    fn open_phase_trips_after_4s() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.gen_i = [5.0, 100.0, 100.0];
        i.feeder_i = i.gen_i;
        for _ in 0..399 {
            assert!(!g.step(&i).trip_latched);
        }
        assert!(g.step(&i).trip_latched);
        let mut g2 = Gcu::new(GcuParams::vfsg());
        established(&mut g2);
        i.gen_i = [5.0, 40.0, 100.0];
        i.feeder_i = i.gen_i;
        assert!(run_until_trip(&mut g2, &i, 1000).is_none());
    }

    #[test]
    fn phase_sequence_inhibits_close_after_100ms() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.phase_seq_abc = false;
        for _ in 0..9 {
            assert!(g.step(&i).gcb_close_permit);
        }
        let o = g.step(&i);
        assert!(!o.gcb_close_permit && o.phase_seq_fault && !o.trip_latched);
        assert_eq!(o.bite_code, GcuFault::PhaseSequence.code());
    }

    #[test]
    fn trip_latches_and_resets_on_gcs_cycle() {
        let mut g = Gcu::new(GcuParams::vfsg());
        established(&mut g);
        let mut i = base_input();
        i.pmg_freq_hz = 2500.0;
        run_until_trip(&mut g, &i, 100).unwrap();
        i.pmg_freq_hz = 2000.0;
        let o = g.step(&i);
        assert!(o.trip_latched && o.field_off && o.exciter_current_a == 0.0);
        i.gcs_on = false;
        g.step(&i);
        i.gcs_on = true;
        let o = g.step(&i);
        assert!(!o.trip_latched && o.vr_enabled);
        i.pmg_freq_hz = 2500.0;
        run_until_trip(&mut g, &i, 100).unwrap();
        i.pmg_freq_hz = 2000.0;
        i.reset_cmd = true;
        assert!(!g.step(&i).trip_latched);
    }

    #[test]
    fn overflow_248_days_drops_generator() {
        let mut g = Gcu::new(GcuParams::vfsg());
        let i = base_input();
        g.step(&i);
        let limit_us = POWERED_TICK_LIMIT * POWERED_TICK_US;
        let days = limit_us as f64 / 86_400.0e6;
        assert!((days - 248.55).abs() < 0.01);
        g.set_powered_time_us(limit_us - 2 * DT);
        assert!(g.step(&i).vr_enabled);
        let o = g.step(&i);
        assert!(!o.vr_enabled);
        let o = g.step(&i);
        assert!(o.trip_latched && o.gcb_trip_cmd && o.field_off);
        assert_eq!(o.bite_code, GcuFault::PoweredTimeOverflow.code());
        let mut j = i;
        j.reset_cmd = true;
        j.gcs_on = false;
        g.step(&j);
        j.gcs_on = true;
        assert!(g.step(&j).trip_latched);
        let mut dead = i;
        dead.pmg_available = false;
        g.step(&dead);
        let o = g.step(&i);
        assert!(!o.trip_latched && o.vr_enabled);
        assert!(g.powered_time_us() < 2 * DT);
    }

    #[test]
    fn fault_masks_distinct() {
        let all = [
            GcuFault::OverVoltage,
            GcuFault::UnderVoltage,
            GcuFault::OverFreq,
            GcuFault::UnderFreq,
            GcuFault::Differential,
            GcuFault::OverCurrent,
            GcuFault::OpenPhase,
            GcuFault::PhaseSequence,
            GcuFault::PoweredTimeOverflow,
        ];
        let mut acc = 0u16;
        for f in all {
            assert_eq!(acc & f.mask(), 0);
            acc |= f.mask();
        }
        assert_eq!(GcuFault::None.mask(), 0);
    }
}
