pub const RPDU_RATINGS_A: [f64; 4] = [2.5, 5.0, 7.5, 10.0];
pub const SPDU_RATINGS_A: [f64; 5] = [15.0, 22.5, 35.0, 45.0, 50.0];
pub const I2T_CONSTANT_S: f64 = 10.0;
pub const INSTANT_TRIP_MULTIPLE: f64 = 15.0;
pub const COOLING_TAU_S: f64 = 20.0;
pub const SOFT_START_US: u64 = 5_000;
pub const SOFT_START_MIN_LIMIT_FRACTION: f64 = 0.2;
pub const SOFT_START_LIMIT_MULTIPLE: f64 = 5.0;
pub const MIN_DC_INPUT_V: f64 = 18.0;
pub const MIN_AC_INPUT_V: f64 = 80.0;
pub const SERIES_ARC_NOISE_RATIO: f64 = 0.2;
pub const SERIES_ARC_MIN_CURRENT_A: f64 = 0.5;
pub const SERIES_ARC_TRIP_US: u64 = 60_000;
pub const PARALLEL_ARC_NOISE_RATING_FRACTION: f64 = 0.5;
pub const PARALLEL_ARC_TRIP_US: u64 = 8_000;
pub const TELEMETRY_VOLT_RES_V: f64 = 0.1;
pub const TELEMETRY_AMP_RES_A: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SspcState {
    Open,
    Closed,
    Tripped,
    LockedOpen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SspcKind {
    DcSoftStart,
    AcZeroCross,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandSource {
    None,
    Local,
    System,
    LoadShed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TripCause {
    I2t,
    Instantaneous,
    SeriesArc,
    ParallelArc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SspcHealth {
    Ok,
    OverCurrentTripped,
    ArcFaultDetected,
    LoadFeedUndervoltage,
    LockedInop,
}

#[derive(Clone, Copy, Debug)]
pub struct SspcConfig {
    pub rating_a: f64,
    pub kind: SspcKind,
    pub phases: u8,
}

impl SspcConfig {
    pub fn dc(rating_a: f64) -> Self {
        Self {
            rating_a,
            kind: SspcKind::DcSoftStart,
            phases: 1,
        }
    }

    pub fn ac(rating_a: f64, phases: u8) -> Self {
        Self {
            rating_a,
            kind: SspcKind::AcZeroCross,
            phases: phases.clamp(1, 3),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SspcInput {
    pub dt_us: u64,
    pub v_in: f64,
    pub currents_a: [f64; 3],
    pub noise_a: f64,
    pub close_cmd: bool,
    pub source: CommandSource,
    pub reset: bool,
    pub lock: bool,
    pub v_zero_cross: bool,
    pub i_zero_cross: bool,
}

impl Default for SspcInput {
    fn default() -> Self {
        Self {
            dt_us: 1_000,
            v_in: 28.0,
            currents_a: [0.0; 3],
            noise_a: 0.0,
            close_cmd: false,
            source: CommandSource::None,
            reset: false,
            lock: false,
            v_zero_cross: true,
            i_zero_cross: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SspcTelemetry {
    pub v_in: f64,
    pub v_out: f64,
    pub current_a: f64,
    pub health: SspcHealth,
}

#[derive(Clone, Copy, Debug)]
pub struct SspcOutput {
    pub state: SspcState,
    pub conducting: bool,
    pub v_out: f64,
    pub current_a: [f64; 3],
    pub current_limit_a: f64,
    pub soft_starting: bool,
    pub inop_tag: bool,
    pub trip_cause: Option<TripCause>,
    pub telemetry: SspcTelemetry,
}

pub fn quantize(x: f64, res: f64) -> f64 {
    (x / res).round() * res
}

#[derive(Clone, Debug)]
pub struct Sspc {
    pub config: SspcConfig,
    state: SspcState,
    heat_a2s: [f64; 3],
    soft_elapsed_us: u64,
    pending_close: bool,
    pending_open: bool,
    series_us: u64,
    parallel_us: u64,
    trip_cause: Option<TripCause>,
    last_source: CommandSource,
    inop_tag: bool,
    undervoltage: bool,
}

impl Sspc {
    pub fn new(config: SspcConfig) -> Self {
        Self {
            config,
            state: SspcState::Open,
            heat_a2s: [0.0; 3],
            soft_elapsed_us: 0,
            pending_close: false,
            pending_open: false,
            series_us: 0,
            parallel_us: 0,
            trip_cause: None,
            last_source: CommandSource::None,
            inop_tag: false,
            undervoltage: false,
        }
    }

    pub fn state(&self) -> SspcState {
        self.state
    }

    pub fn rating_a(&self) -> f64 {
        self.config.rating_a
    }

    pub fn heat_fraction(&self) -> f64 {
        let limit = self.config.rating_a * self.config.rating_a * I2T_CONSTANT_S;
        self.heat_a2s.iter().cloned().fold(0.0, f64::max) / limit
    }

    pub fn last_source(&self) -> CommandSource {
        self.last_source
    }

    pub fn inop_tag(&self) -> bool {
        self.inop_tag
    }

    fn soft_fraction(&self) -> f64 {
        if self.config.kind != SspcKind::DcSoftStart {
            return 1.0;
        }
        (self.soft_elapsed_us as f64 / SOFT_START_US as f64).min(1.0)
    }

    fn trip(&mut self, cause: TripCause) {
        self.state = SspcState::Tripped;
        self.trip_cause = Some(cause);
        self.pending_close = false;
        self.pending_open = false;
    }

    fn current_limit(&self) -> f64 {
        let f = self.soft_fraction();
        if f >= 1.0 {
            f64::MAX
        } else {
            self.config.rating_a * (SOFT_START_LIMIT_MULTIPLE * f).max(SOFT_START_MIN_LIMIT_FRACTION)
        }
    }

    fn cool(&mut self, dt_s: f64) {
        for h in self.heat_a2s.iter_mut() {
            *h *= (-dt_s / COOLING_TAU_S).exp();
        }
        self.series_us = 0;
        self.parallel_us = 0;
    }

    pub fn step(&mut self, input: &SspcInput) -> SspcOutput {
        let dt_s = input.dt_us as f64 * 1e-6;
        let min_v = match self.config.kind {
            SspcKind::DcSoftStart => MIN_DC_INPUT_V,
            SspcKind::AcZeroCross => MIN_AC_INPUT_V,
        };
        self.undervoltage = input.v_in < min_v;

        if input.lock {
            self.state = SspcState::LockedOpen;
            self.inop_tag = true;
            self.pending_close = false;
            self.pending_open = false;
        } else if self.state == SspcState::LockedOpen {
            self.state = SspcState::Open;
            self.inop_tag = false;
        }

        let mut currents = [0.0; 3];
        match self.state {
            SspcState::LockedOpen => {}
            SspcState::Tripped => {
                if input.reset && !input.close_cmd {
                    self.state = SspcState::Open;
                    self.trip_cause = None;
                }
                self.cool(dt_s);
            }
            SspcState::Open => {
                self.cool(dt_s);
                if input.close_cmd && !self.undervoltage {
                    self.last_source = input.source;
                    match self.config.kind {
                        SspcKind::DcSoftStart => {
                            self.state = SspcState::Closed;
                            self.soft_elapsed_us = 0;
                        }
                        SspcKind::AcZeroCross => {
                            self.pending_close = true;
                            if input.v_zero_cross {
                                self.state = SspcState::Closed;
                                self.pending_close = false;
                            }
                        }
                    }
                } else if !input.close_cmd {
                    self.pending_close = false;
                }
            }
            SspcState::Closed => {
                if input.close_cmd {
                    self.last_source = input.source;
                    self.pending_open = false;
                } else {
                    match self.config.kind {
                        SspcKind::DcSoftStart => self.state = SspcState::Open,
                        SspcKind::AcZeroCross => self.pending_open = true,
                    }
                }
                if self.state == SspcState::Closed {
                    currents = self.closed_step(input, dt_s);
                }
            }
        }

        let conducting = self.state == SspcState::Closed;
        let frac = if conducting { self.soft_fraction() } else { 0.0 };
        let v_out = if conducting { input.v_in * frac } else { 0.0 };
        let total_out = currents.iter().sum::<f64>() / self.config.phases as f64;
        let health = match (self.state, self.trip_cause) {
            (SspcState::LockedOpen, _) => SspcHealth::LockedInop,
            (SspcState::Tripped, Some(TripCause::SeriesArc | TripCause::ParallelArc)) => {
                SspcHealth::ArcFaultDetected
            }
            (SspcState::Tripped, _) => SspcHealth::OverCurrentTripped,
            _ if self.undervoltage => SspcHealth::LoadFeedUndervoltage,
            _ => SspcHealth::Ok,
        };
        SspcOutput {
            state: self.state,
            conducting,
            v_out,
            current_a: currents,
            current_limit_a: self.current_limit(),
            soft_starting: conducting && self.soft_fraction() < 1.0,
            inop_tag: self.inop_tag,
            trip_cause: self.trip_cause,
            telemetry: SspcTelemetry {
                v_in: quantize(input.v_in, TELEMETRY_VOLT_RES_V),
                v_out: quantize(v_out, TELEMETRY_VOLT_RES_V),
                current_a: quantize(total_out, TELEMETRY_AMP_RES_A),
                health,
            },
        }
    }

    fn closed_step(&mut self, input: &SspcInput, dt_s: f64) -> [f64; 3] {
        let rating = self.config.rating_a;
        let mut currents = [0.0; 3];
        self.soft_elapsed_us = self.soft_elapsed_us.saturating_add(input.dt_us);
        let soft = self.soft_fraction() < 1.0;
        let limit = self.current_limit();
        let active = self.config.phases as usize;
        let mut total = 0.0;
        let mut tripped = None;
        #[allow(clippy::needless_range_loop)]
        for p in 0..active {
            let i = input.currents_a[p].abs();
            currents[p] = i.min(limit);
            total += i;
            if !soft && i >= rating * INSTANT_TRIP_MULTIPLE {
                tripped = Some(TripCause::Instantaneous);
            }
            let h = self.heat_a2s[p] + (currents[p] * currents[p] - rating * rating) * dt_s;
            self.heat_a2s[p] = if currents[p] < rating {
                h.max(0.0) * (-dt_s / COOLING_TAU_S).exp()
            } else {
                h.max(0.0)
            };
            if self.heat_a2s[p] >= rating * rating * I2T_CONSTANT_S {
                tripped = tripped.or(Some(TripCause::I2t));
            }
        }
        let mean = total / active as f64;
        let series = mean >= SERIES_ARC_MIN_CURRENT_A
            && input.noise_a > SERIES_ARC_NOISE_RATIO * mean
            && mean <= rating * 1.2;
        let parallel = input.noise_a > PARALLEL_ARC_NOISE_RATING_FRACTION * rating && mean > rating;
        self.series_us = if series {
            self.series_us + input.dt_us
        } else {
            self.series_us / 2
        };
        self.parallel_us = if parallel {
            self.parallel_us + input.dt_us
        } else {
            self.parallel_us / 2
        };
        if self.parallel_us >= PARALLEL_ARC_TRIP_US {
            tripped = tripped.or(Some(TripCause::ParallelArc));
        }
        if self.series_us >= SERIES_ARC_TRIP_US {
            tripped = tripped.or(Some(TripCause::SeriesArc));
        }
        if let Some(c) = tripped {
            self.trip(c);
            return [0.0; 3];
        }
        if self.pending_open && input.i_zero_cross {
            self.state = SspcState::Open;
            self.pending_open = false;
        }
        currents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closed_dc(rating: f64) -> Sspc {
        let mut s = Sspc::new(SspcConfig::dc(rating));
        let inp = SspcInput {
            close_cmd: true,
            ..Default::default()
        };
        for _ in 0..10 {
            s.step(&inp);
        }
        assert_eq!(s.state(), SspcState::Closed);
        s
    }

    fn trip_time_ms(rating: f64, mult: f64) -> Option<u64> {
        let mut s = closed_dc(rating);
        let inp = SspcInput {
            close_cmd: true,
            currents_a: [rating * mult, 0.0, 0.0],
            ..Default::default()
        };
        (1..=60_000u64).find(|_| s.step(&inp).state == SspcState::Tripped)
    }

    #[test]
    fn no_trip_at_rating() {
        assert_eq!(trip_time_ms(10.0, 1.0), None);
    }

    #[test]
    fn trip_times_follow_i2t_curve() {
        let t15 = trip_time_ms(10.0, 1.5).unwrap();
        let t2 = trip_time_ms(10.0, 2.0).unwrap();
        let t5 = trip_time_ms(10.0, 5.0).unwrap();
        let t10 = trip_time_ms(10.0, 10.0).unwrap();
        assert!((7_900..=8_100).contains(&t15), "{t15}");
        assert!((3_300..=3_400).contains(&t2), "{t2}");
        assert!((410..=425).contains(&t5), "{t5}");
        assert!((100..=105).contains(&t10), "{t10}");
        assert!(t15 > t2 && t2 > t5 && t5 > t10);
    }

    #[test]
    fn instantaneous_trip_on_hard_short() {
        let mut s = closed_dc(5.0);
        let o = s.step(&SspcInput {
            close_cmd: true,
            currents_a: [80.0, 0.0, 0.0],
            ..Default::default()
        });
        assert_eq!(o.state, SspcState::Tripped);
        assert_eq!(o.trip_cause, Some(TripCause::Instantaneous));
        assert_eq!(o.current_a[0], 0.0);
    }

    #[test]
    fn thermal_memory_cools_down() {
        let mut s = closed_dc(10.0);
        let hot = SspcInput {
            close_cmd: true,
            currents_a: [20.0, 0.0, 0.0],
            ..Default::default()
        };
        for _ in 0..1_000 {
            s.step(&hot);
        }
        let h1 = s.heat_fraction();
        assert!(h1 > 0.2);
        let idle = SspcInput {
            close_cmd: true,
            currents_a: [1.0, 0.0, 0.0],
            ..Default::default()
        };
        for _ in 0..20_000 {
            s.step(&idle);
        }
        assert!(s.heat_fraction() < h1 * 0.5);
    }

    #[test]
    fn series_arc_trips() {
        let mut s = closed_dc(10.0);
        let inp = SspcInput {
            close_cmd: true,
            currents_a: [5.0, 0.0, 0.0],
            noise_a: 2.0,
            ..Default::default()
        };
        let mut n = 0;
        while s.step(&inp).state == SspcState::Closed {
            n += 1;
            assert!(n < 1000);
        }
        assert!((55..=65).contains(&n), "{n}");
        assert_eq!(s.step(&inp).telemetry.health, SspcHealth::ArcFaultDetected);
    }

    #[test]
    fn parallel_arc_trips_fast() {
        let mut s = closed_dc(10.0);
        let inp = SspcInput {
            close_cmd: true,
            currents_a: [14.0, 0.0, 0.0],
            noise_a: 6.0,
            ..Default::default()
        };
        let mut n = 0;
        let mut o = s.step(&inp);
        while o.state == SspcState::Closed {
            n += 1;
            o = s.step(&inp);
        }
        assert!(n <= 10);
        assert_eq!(o.trip_cause, Some(TripCause::ParallelArc));
    }

    #[test]
    fn clean_load_no_arc() {
        let mut s = closed_dc(10.0);
        let inp = SspcInput {
            close_cmd: true,
            currents_a: [5.0, 0.0, 0.0],
            noise_a: 0.05,
            ..Default::default()
        };
        for _ in 0..2_000 {
            assert_eq!(s.step(&inp).state, SspcState::Closed);
        }
    }

    #[test]
    fn soft_start_limits_inrush() {
        let mut s = Sspc::new(SspcConfig::dc(10.0));
        let inp = SspcInput {
            dt_us: 500,
            close_cmd: true,
            currents_a: [60.0, 0.0, 0.0],
            ..Default::default()
        };
        let first = s.step(&inp);
        assert!(first.soft_starting);
        assert!(first.current_a[0] <= 10.0 * SOFT_START_LIMIT_MULTIPLE * 0.2 + 1e-9);
        assert_eq!(first.state, SspcState::Closed);
        assert!(first.v_out < 28.0 * 0.2);
        let mut last = first;
        for _ in 0..20 {
            last = s.step(&inp);
            assert_ne!(last.state, SspcState::Tripped);
        }
        assert!(!last.soft_starting);
        assert!((last.v_out - 28.0).abs() < 1e-9);
        assert!(last.current_a[0] > 50.0);
    }

    #[test]
    fn ac_closes_on_voltage_zero_and_opens_on_current_zero() {
        let mut s = Sspc::new(SspcConfig::ac(5.0, 1));
        let mut inp = SspcInput {
            v_in: 115.0,
            close_cmd: true,
            v_zero_cross: false,
            ..Default::default()
        };
        assert_eq!(s.step(&inp).state, SspcState::Open);
        inp.v_zero_cross = true;
        assert_eq!(s.step(&inp).state, SspcState::Closed);
        inp.close_cmd = false;
        inp.i_zero_cross = false;
        inp.currents_a = [3.0, 0.0, 0.0];
        assert_eq!(s.step(&inp).state, SspcState::Closed);
        inp.i_zero_cross = true;
        assert_eq!(s.step(&inp).state, SspcState::Open);
    }

    #[test]
    fn three_phase_trips_on_one_phase() {
        let mut s = Sspc::new(SspcConfig::ac(10.0, 3));
        let mut inp = SspcInput {
            v_in: 115.0,
            close_cmd: true,
            ..Default::default()
        };
        s.step(&inp);
        inp.currents_a = [5.0, 5.0, 40.0];
        let mut n = 0;
        while s.step(&inp).state == SspcState::Closed {
            n += 1;
            assert!(n < 10_000);
        }
        assert_eq!(s.state(), SspcState::Tripped);
    }

    #[test]
    fn single_phase_ignores_unused_phases() {
        let mut s = Sspc::new(SspcConfig::ac(10.0, 1));
        let mut inp = SspcInput {
            v_in: 115.0,
            close_cmd: true,
            ..Default::default()
        };
        s.step(&inp);
        inp.currents_a = [5.0, 100.0, 100.0];
        assert_eq!(s.step(&inp).state, SspcState::Closed);
    }

    #[test]
    fn trip_requires_reset_and_lock_ignores_close() {
        let mut s = closed_dc(5.0);
        s.step(&SspcInput {
            close_cmd: true,
            currents_a: [100.0, 0.0, 0.0],
            ..Default::default()
        });
        assert_eq!(s.state(), SspcState::Tripped);
        let o = s.step(&SspcInput {
            close_cmd: true,
            ..Default::default()
        });
        assert_eq!(o.state, SspcState::Tripped);
        let o = s.step(&SspcInput {
            reset: true,
            ..Default::default()
        });
        assert_eq!(o.state, SspcState::Open);
        let o = s.step(&SspcInput {
            lock: true,
            close_cmd: true,
            ..Default::default()
        });
        assert_eq!(o.state, SspcState::LockedOpen);
        assert!(o.inop_tag);
        assert!(!o.conducting);
        assert_eq!(o.telemetry.health, SspcHealth::LockedInop);
        let o = s.step(&SspcInput::default());
        assert_eq!(o.state, SspcState::Open);
        assert!(!o.inop_tag);
    }

    #[test]
    fn telemetry_is_quantized_and_undervoltage_blocks() {
        let mut s = closed_dc(10.0);
        let o = s.step(&SspcInput {
            v_in: 27.934,
            close_cmd: true,
            currents_a: [3.15159, 0.0, 0.0],
            ..Default::default()
        });
        assert!((o.telemetry.v_in - 27.9).abs() < 1e-9);
        assert!((o.telemetry.current_a - 3.15).abs() < 1e-9);
        let mut s2 = Sspc::new(SspcConfig::dc(10.0));
        let o = s2.step(&SspcInput {
            v_in: 10.0,
            close_cmd: true,
            ..Default::default()
        });
        assert_eq!(o.state, SspcState::Open);
        assert_eq!(o.telemetry.health, SspcHealth::LoadFeedUndervoltage);
    }
}
