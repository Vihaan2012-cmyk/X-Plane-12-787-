use super::machine::Thevenin;

pub const EP_OV_V: f64 = 130.0;
pub const EP_OV_RESET_V: f64 = 128.0;
pub const EP_OV_MAX_DELAY_S: f64 = 10.0;
pub const APPROX_EP_OV_TIERS: [(f64, f64); 5] = [
    (160.0, 0.05),
    (150.0, 0.2),
    (140.0, 1.0),
    (135.0, 3.0),
    (130.0, 10.0),
];
pub const EP_UV_V: f64 = 104.5;
pub const EP_UV_RESET_V: f64 = 107.5;
pub const EP_UV_DELAY_S: f64 = 9.0;
pub const EP_OF_HZ: f64 = 429.0;
pub const EP_OF_RESET_HZ: f64 = 427.0;
pub const EP_OF_DELAY_S: f64 = 1.0;
pub const EP_UF_HZ: f64 = 371.0;
pub const EP_UF_RESET_HZ: f64 = 373.0;
pub const EP_UF_DELAY_S: f64 = 1.0;
pub const EP_PHASE_SEQ_DELAY_S: f64 = 0.1;
pub const EP_POWER_QUALITY_DELAY_S: f64 = 0.15;
pub const EP_OC1_A: f64 = 270.0;
pub const EP_OC1_DELAY_S: f64 = 300.0;
pub const EP_OC1_RESET_A: f64 = 250.0;
pub const EP_OC2_A: f64 = 326.0;
pub const EP_OC2_DELAY_S: f64 = 5.2;
pub const EP_OC2_RESET_A: f64 = 306.0;
pub const EP_RECOMMENDED_SOURCE_VA: f64 = 90_000.0;
pub const EP_OPEN_PHASE_LOW_A: f64 = 11.0;
pub const EP_OPEN_PHASE_SPREAD_A: f64 = 55.0;
pub const EP_OPEN_PHASE_DELAY_S: f64 = 4.0;
pub const EP_OPEN_PHASE_RESET_LOW_A: f64 = 20.0;
pub const EP_OPEN_PHASE_RESET_SPREAD_A: f64 = 45.0;
pub const APPROX_EP_SOURCE_R_OHM: f64 = 0.02;
pub const APPROX_EP_SOURCE_X_OHM: f64 = 0.03;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receptacle {
    LeftFwd,
    RightFwd,
    Aft,
}

impl Receptacle {
    pub fn is_aft(self) -> bool {
        matches!(self, Receptacle::Aft)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpFault {
    None,
    NotConnected,
    OverVoltage,
    UnderVoltage,
    OverFreq,
    UnderFreq,
    OpenPhase,
    PhaseSequence,
    OverCurrent,
    PinEfInterlock,
    StartModeActive,
    AtruSaturation,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtPowerInput {
    pub dt_us: u64,
    pub connected: bool,
    pub pin_ef_ok: bool,
    pub phase_v: [f64; 3],
    pub phase_i: [f64; 3],
    pub freq_hz: f64,
    pub phase_seq_abc: bool,
    pub engine_start_mode_active: bool,
    pub atru_saturation: bool,
}

impl Default for ExtPowerInput {
    fn default() -> Self {
        Self {
            dt_us: 10_000,
            connected: false,
            pin_ef_ok: false,
            phase_v: [0.0; 3],
            phase_i: [0.0; 3],
            freq_hz: 0.0,
            phase_seq_abc: true,
            engine_start_mode_active: false,
            atru_saturation: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtPowerOutput {
    pub avail: bool,
    pub power_ready: bool,
    pub power_quality_good: bool,
    pub epc_close_permit: bool,
    pub tripped: bool,
    pub fault: EpFault,
    pub source: Thevenin,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtPowerMonitor {
    pub receptacle: Receptacle,
    t_ov: u64,
    t_uv: u64,
    t_of: u64,
    t_uf: u64,
    t_seq: u64,
    t_oc1: u64,
    t_oc2: u64,
    t_good: u64,
    t_op: u64,
    op: bool,
    ov: bool,
    uv: bool,
    of: bool,
    uf: bool,
    ps: bool,
    oc: bool,
}

fn us(s: f64) -> u64 {
    (s * 1.0e6).round() as u64
}

pub fn ep_ov_delay_s(v: f64) -> Option<f64> {
    APPROX_EP_OV_TIERS.iter().find(|(t, _)| v > *t).map(|(_, d)| *d)
}

impl ExtPowerMonitor {
    pub fn new(receptacle: Receptacle) -> Self {
        Self {
            receptacle,
            t_ov: 0,
            t_uv: 0,
            t_of: 0,
            t_uf: 0,
            t_seq: 0,
            t_oc1: 0,
            t_oc2: 0,
            t_good: 0,
            t_op: 0,
            op: false,
            ov: false,
            uv: false,
            of: false,
            uf: false,
            ps: false,
            oc: false,
        }
    }

    pub fn step(&mut self, input: &ExtPowerInput) -> ExtPowerOutput {
        let dt = input.dt_us;
        if !input.connected {
            *self = Self::new(self.receptacle);
            return ExtPowerOutput {
                avail: false,
                power_ready: false,
                power_quality_good: false,
                epc_close_permit: false,
                tripped: false,
                fault: EpFault::NotConnected,
                source: Thevenin {
                    freq_hz: input.freq_hz,
                    r_ohm: APPROX_EP_SOURCE_R_OHM,
                    x_ohm: APPROX_EP_SOURCE_X_OHM,
                    ..Thevenin::default()
                },
            };
        }
        let hi = input.phase_v.iter().cloned().fold(f64::MIN, f64::max);
        let lo = input.phase_v.iter().cloned().fold(f64::MAX, f64::min);
        let mut sorted_i = input.phase_i;
        sorted_i.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let spread = sorted_i[1] - sorted_i[0];
        if sorted_i[0] < EP_OPEN_PHASE_LOW_A && spread > EP_OPEN_PHASE_SPREAD_A {
            self.t_op = self.t_op.saturating_add(dt);
            if self.t_op >= us(EP_OPEN_PHASE_DELAY_S) {
                self.op = true;
            }
        } else if sorted_i[0] > EP_OPEN_PHASE_RESET_LOW_A || spread < EP_OPEN_PHASE_RESET_SPREAD_A {
            self.t_op = 0;
            self.op = false;
        }
        let open_phase = self.op;

        if hi > EP_OV_V {
            let d = ep_ov_delay_s(hi)
                .unwrap_or(EP_OV_MAX_DELAY_S)
                .min(EP_OV_MAX_DELAY_S);
            self.t_ov = self.t_ov.saturating_add(dt);
            if self.t_ov >= us(d) {
                self.ov = true;
            }
        } else {
            self.t_ov = 0;
        }
        if self.ov && hi < EP_OV_RESET_V {
            self.ov = false;
        }

        let uv_cond = !open_phase && lo < EP_UV_V;
        if uv_cond {
            self.t_uv = self.t_uv.saturating_add(dt);
            if self.t_uv >= us(EP_UV_DELAY_S) {
                self.uv = true;
            }
        } else {
            self.t_uv = 0;
        }
        if self.uv && hi > EP_UV_RESET_V {
            self.uv = false;
        }
        let uv_active = uv_cond || self.uv;

        if input.freq_hz > EP_OF_HZ && !uv_active {
            self.t_of = self.t_of.saturating_add(dt);
            if self.t_of >= us(EP_OF_DELAY_S) {
                self.of = true;
            }
        } else {
            self.t_of = 0;
        }
        if self.of && input.freq_hz < EP_OF_RESET_HZ {
            self.of = false;
        }
        if input.freq_hz < EP_UF_HZ && !uv_active {
            self.t_uf = self.t_uf.saturating_add(dt);
            if self.t_uf >= us(EP_UF_DELAY_S) {
                self.uf = true;
            }
        } else {
            self.t_uf = 0;
        }
        if self.uf && input.freq_hz > EP_UF_RESET_HZ {
            self.uf = false;
        }

        if input.phase_seq_abc {
            self.t_seq = 0;
            self.ps = false;
        } else {
            self.t_seq = self.t_seq.saturating_add(dt);
            if self.t_seq >= us(EP_PHASE_SEQ_DELAY_S) {
                self.ps = true;
            }
        }

        let i_hi = input.phase_i.iter().cloned().fold(f64::MIN, f64::max);
        if i_hi > EP_OC1_A {
            self.t_oc1 = self.t_oc1.saturating_add(dt);
        } else if i_hi < EP_OC1_RESET_A {
            self.t_oc1 = 0;
        }
        if i_hi > EP_OC2_A {
            self.t_oc2 = self.t_oc2.saturating_add(dt);
        } else if i_hi < EP_OC2_RESET_A {
            self.t_oc2 = 0;
        }
        if self.t_oc1 >= us(EP_OC1_DELAY_S) || self.t_oc2 >= us(EP_OC2_DELAY_S) {
            self.oc = true;
        }

        let quality_clear = !self.ov && !self.uv && !self.of && !self.uf && !uv_cond && !open_phase;
        if quality_clear {
            self.t_good = self.t_good.saturating_add(dt);
        } else {
            self.t_good = 0;
        }
        let quality_good = self.t_good >= us(EP_POWER_QUALITY_DELAY_S);

        let fault = if open_phase {
            EpFault::OpenPhase
        } else if self.ov {
            EpFault::OverVoltage
        } else if self.uv || uv_cond {
            EpFault::UnderVoltage
        } else if self.of {
            EpFault::OverFreq
        } else if self.uf {
            EpFault::UnderFreq
        } else if self.ps {
            EpFault::PhaseSequence
        } else if self.oc {
            EpFault::OverCurrent
        } else if !input.pin_ef_ok {
            EpFault::PinEfInterlock
        } else if input.engine_start_mode_active && !self.receptacle.is_aft() {
            EpFault::StartModeActive
        } else if self.receptacle.is_aft() && input.atru_saturation {
            EpFault::AtruSaturation
        } else {
            EpFault::None
        };
        let ready = quality_good && fault == EpFault::None;
        let v_mean = (input.phase_v[0] + input.phase_v[1] + input.phase_v[2]) / 3.0;
        ExtPowerOutput {
            avail: ready,
            power_ready: ready,
            power_quality_good: quality_good,
            epc_close_permit: ready,
            tripped: self.ov || self.uv || self.of || self.uf || self.ps || self.oc,
            fault,
            source: Thevenin {
                emf_v: v_mean,
                angle_rad: 0.0,
                r_ohm: APPROX_EP_SOURCE_R_OHM,
                x_ohm: APPROX_EP_SOURCE_X_OHM,
                freq_hz: input.freq_hz,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> ExtPowerInput {
        ExtPowerInput {
            connected: true,
            pin_ef_ok: true,
            phase_v: [115.0; 3],
            phase_i: [50.0; 3],
            freq_hz: 400.0,
            ..ExtPowerInput::default()
        }
    }

    fn run(m: &mut ExtPowerMonitor, i: &ExtPowerInput, steps: usize) -> ExtPowerOutput {
        let mut o = m.step(i);
        for _ in 1..steps {
            o = m.step(i);
        }
        o
    }

    #[test]
    fn avail_after_150ms() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = good();
        assert!(!run(&mut m, &i, 14).avail);
        let o = m.step(&i);
        assert!(o.avail && o.power_quality_good && o.epc_close_permit);
    }

    #[test]
    fn overvoltage_inverse_time_and_reset() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            phase_v: [131.0, 115.0, 115.0],
            ..good()
        };
        assert!(!run(&mut m, &i, 999).tripped);
        assert!(m.step(&i).tripped);
        let mut m2 = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let hot = ExtPowerInput {
            phase_v: [170.0, 115.0, 115.0],
            ..good()
        };
        assert!(!run(&mut m2, &hot, 4).tripped);
        assert!(run(&mut m2, &hot, 3).tripped);
        let o = run(
            &mut m2,
            &ExtPowerInput {
                phase_v: [129.0; 3],
                ..good()
            },
            100,
        );
        assert!(o.tripped && !o.avail && o.fault == EpFault::OverVoltage);
        let o = run(&mut m2, &good(), 20);
        assert!(!o.tripped && o.avail);
    }

    #[test]
    fn overvoltage_at_130_does_not_trip() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            phase_v: [130.0; 3],
            ..good()
        };
        assert!(!run(&mut m, &i, 1500).tripped);
    }

    #[test]
    fn undervoltage_9s_and_reset() {
        let mut m = ExtPowerMonitor::new(Receptacle::RightFwd);
        let i = ExtPowerInput {
            phase_v: [104.0; 3],
            ..good()
        };
        let o = run(&mut m, &i, 899);
        assert!(!o.tripped && !o.avail);
        let o = m.step(&i);
        assert!(o.tripped && o.fault == EpFault::UnderVoltage);
        let o = run(
            &mut m,
            &ExtPowerInput {
                phase_v: [107.0; 3],
                ..good()
            },
            50,
        );
        assert!(o.tripped);
        let o = run(&mut m, &good(), 20);
        assert!(!o.tripped && o.avail);
    }

    #[test]
    fn overfrequency_1s_reset_427() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            freq_hz: 430.0,
            ..good()
        };
        assert!(!run(&mut m, &i, 99).tripped);
        assert!(m.step(&i).tripped);
        let o = run(
            &mut m,
            &ExtPowerInput {
                freq_hz: 428.0,
                ..good()
            },
            50,
        );
        assert!(o.tripped);
        assert!(!run(&mut m, &good(), 20).tripped);
    }

    #[test]
    fn underfrequency_1s_reset_373() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            freq_hz: 370.0,
            ..good()
        };
        assert!(!run(&mut m, &i, 99).tripped);
        assert!(m.step(&i).tripped);
        let o = run(
            &mut m,
            &ExtPowerInput {
                freq_hz: 372.0,
                ..good()
            },
            50,
        );
        assert!(o.tripped);
        assert!(!run(&mut m, &good(), 20).tripped);
        let mut m2 = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let edge = ExtPowerInput {
            freq_hz: 371.0,
            ..good()
        };
        assert!(!run(&mut m2, &edge, 300).tripped);
    }

    #[test]
    fn uv_inhibits_frequency_protection() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            phase_v: [100.0; 3],
            freq_hz: 300.0,
            ..good()
        };
        let o = run(&mut m, &i, 899);
        assert!(!o.tripped);
        let o = m.step(&i);
        assert_eq!(o.fault, EpFault::UnderVoltage);
    }

    #[test]
    fn phase_sequence_100ms() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            phase_seq_abc: false,
            ..good()
        };
        assert!(!run(&mut m, &i, 9).tripped);
        let o = m.step(&i);
        assert!(o.tripped && !o.avail && o.fault == EpFault::PhaseSequence);
        let o = run(&mut m, &good(), 20);
        assert!(o.avail);
    }

    #[test]
    fn overcurrent_levels() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            phase_i: [330.0, 50.0, 50.0],
            ..good()
        };
        assert!(!run(&mut m, &i, 519).tripped);
        let o = m.step(&i);
        assert!(o.tripped && o.fault == EpFault::OverCurrent);
        let mut m2 = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i2 = ExtPowerInput {
            phase_i: [280.0, 50.0, 50.0],
            ..good()
        };
        assert!(!run(&mut m2, &i2, 29_999).tripped);
        assert!(m2.step(&i2).tripped);
        let mut m3 = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i3 = ExtPowerInput {
            phase_i: [260.0, 50.0, 50.0],
            ..good()
        };
        assert!(!run(&mut m3, &i3, 40_000).tripped);
    }

    #[test]
    fn interlock_and_mode_faults() {
        let cases: [(Receptacle, ExtPowerInput, EpFault); 3] = [
            (
                Receptacle::LeftFwd,
                ExtPowerInput {
                    pin_ef_ok: false,
                    ..good()
                },
                EpFault::PinEfInterlock,
            ),
            (
                Receptacle::RightFwd,
                ExtPowerInput {
                    engine_start_mode_active: true,
                    ..good()
                },
                EpFault::StartModeActive,
            ),
            (
                Receptacle::Aft,
                ExtPowerInput {
                    atru_saturation: true,
                    ..good()
                },
                EpFault::AtruSaturation,
            ),
        ];
        for (r, i, f) in cases {
            let mut m = ExtPowerMonitor::new(r);
            let o = run(&mut m, &i, 100);
            assert!(!o.avail && !o.epc_close_permit);
            assert_eq!(o.fault, f);
        }
        let mut m = ExtPowerMonitor::new(Receptacle::Aft);
        let i = ExtPowerInput {
            engine_start_mode_active: true,
            ..good()
        };
        assert!(run(&mut m, &i, 100).avail);
    }

    #[test]
    fn open_phase_current_rule_4s_and_reset() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let i = ExtPowerInput {
            phase_i: [5.0, 100.0, 100.0],
            ..good()
        };
        assert_ne!(run(&mut m, &i, 399).fault, EpFault::OpenPhase);
        let o = m.step(&i);
        assert_eq!(o.fault, EpFault::OpenPhase);
        assert!(!o.avail);
        let hold = ExtPowerInput {
            phase_i: [15.0, 100.0, 100.0],
            ..good()
        };
        assert_eq!(run(&mut m, &hold, 10).fault, EpFault::OpenPhase);
        let o = run(&mut m, &good(), 20);
        assert_eq!(o.fault, EpFault::None);
        let mut m2 = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let small_spread = ExtPowerInput {
            phase_i: [5.0, 50.0, 100.0],
            ..good()
        };
        assert_ne!(run(&mut m2, &small_spread, 1000).fault, EpFault::OpenPhase);
        let mut m3 = ExtPowerMonitor::new(Receptacle::LeftFwd);
        let high_low = ExtPowerInput {
            phase_i: [12.0, 100.0, 100.0],
            ..good()
        };
        assert_ne!(run(&mut m3, &high_low, 1000).fault, EpFault::OpenPhase);
    }

    #[test]
    fn disconnect_drops_avail_and_clears_lockout() {
        let mut m = ExtPowerMonitor::new(Receptacle::LeftFwd);
        assert!(run(&mut m, &good(), 100).avail);
        let o = m.step(&ExtPowerInput::default());
        assert!(!o.avail);
        assert_eq!(o.source.emf_v, 0.0);
        let bad = ExtPowerInput {
            phase_seq_abc: false,
            ..good()
        };
        run(&mut m, &bad, 20);
        m.step(&ExtPowerInput::default());
        assert!(run(&mut m, &good(), 20).avail);
    }
}
