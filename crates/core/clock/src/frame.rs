use crate::scheduler::{Fired, Scheduler};
use crate::time::{SimDuration, SimTime};

const FRAC_BITS: u32 = 32;
const FRAC_MASK: u128 = (1u128 << FRAC_BITS) - 1;
const FRAC_SCALE: f64 = 4_294_967_296.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    InvalidDt,
    TimeOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overrun {
    pub frame: u64,
    pub cost: u64,
    pub budget: u64,
    pub excess: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameReport {
    pub frame: u64,
    pub start: SimTime,
    pub end: SimTime,
    pub overrun: Option<Overrun>,
}

impl FrameReport {
    pub fn advanced(&self) -> SimDuration {
        self.end - self.start
    }
}

#[derive(Debug, Clone)]
pub struct FrameDriver {
    now: SimTime,
    frac: u64,
    speed: u32,
    budget: u64,
    frames: u64,
    overruns: u64,
    worst_excess: u64,
    last_overrun: Option<Overrun>,
}

impl FrameDriver {
    pub fn new(budget: u64) -> Self {
        FrameDriver {
            now: SimTime::ZERO,
            frac: 0,
            speed: 1,
            budget,
            frames: 0,
            overruns: 0,
            worst_excess: 0,
            last_overrun: None,
        }
    }

    pub fn starting_at(now: SimTime, budget: u64) -> Self {
        let mut d = Self::new(budget);
        d.now = now;
        d
    }

    pub fn now(&self) -> SimTime {
        self.now
    }

    pub fn fraction_units(&self) -> u64 {
        self.frac
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    pub fn budget(&self) -> u64 {
        self.budget
    }

    pub fn set_budget(&mut self, budget: u64) {
        self.budget = budget;
    }

    pub fn speed(&self) -> u32 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed;
    }

    pub fn overrun_count(&self) -> u64 {
        self.overruns
    }

    pub fn worst_excess(&self) -> u64 {
        self.worst_excess
    }

    pub fn last_overrun(&self) -> Option<Overrun> {
        self.last_overrun
    }

    fn dt_to_units(dt: f64) -> Result<u128, FrameError> {
        if !dt.is_finite() || dt < 0.0 {
            return Err(FrameError::InvalidDt);
        }
        let units = (dt * 1.0e6 * FRAC_SCALE).round();
        if units >= 3.0e38 {
            return Err(FrameError::TimeOverflow);
        }
        Ok(units as u128)
    }

    fn apply(&mut self, units: u128, cost: u64) -> Result<FrameReport, FrameError> {
        let scaled = units
            .checked_mul(u128::from(self.speed))
            .ok_or(FrameError::TimeOverflow)?;
        let total = u128::from(self.frac) + scaled;
        let whole = total >> FRAC_BITS;
        let whole = u64::try_from(whole).map_err(|_| FrameError::TimeOverflow)?;
        let end = self
            .now
            .checked_add(SimDuration::from_micros(whole))
            .ok_or(FrameError::TimeOverflow)?;
        let start = self.now;
        self.now = end;
        self.frac = (total & FRAC_MASK) as u64;
        let frame = self.frames;
        self.frames += 1;
        let overrun = if cost > self.budget {
            let o = Overrun {
                frame,
                cost,
                budget: self.budget,
                excess: cost - self.budget,
            };
            self.overruns += 1;
            self.worst_excess = self.worst_excess.max(o.excess);
            self.last_overrun = Some(o);
            Some(o)
        } else {
            None
        };
        Ok(FrameReport {
            frame,
            start,
            end,
            overrun,
        })
    }

    pub fn step(&mut self, dt_seconds: f64, cost: u64) -> Result<FrameReport, FrameError> {
        let units = Self::dt_to_units(dt_seconds)?;
        self.apply(units, cost)
    }

    pub fn step_exact(&mut self, dt: SimDuration, cost: u64) -> Result<FrameReport, FrameError> {
        let units = u128::from(dt.as_micros()) << FRAC_BITS;
        self.apply(units, cost)
    }

    pub fn drive<E>(
        &mut self,
        scheduler: &mut Scheduler<E>,
        dt_seconds: f64,
        cost: u64,
    ) -> Result<(FrameReport, Vec<Fired<E>>), FrameError> {
        let report = self.step(dt_seconds, cost)?;
        let fired = scheduler.advance_to(report.end);
        Ok((report, fired))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rate::RateTable;

    fn simulate(fps: u32, until: SimTime) -> Vec<(u64, u32, u64)> {
        let mut rates = RateTable::new();
        rates
            .register(SimDuration::from_micros(1_000), SimDuration::ZERO, 1)
            .unwrap();
        rates
            .register(SimDuration::from_micros(3_333), SimDuration::from_micros(17), 0)
            .unwrap();
        rates
            .register(SimDuration::from_micros(50_000), SimDuration::ZERO, 2)
            .unwrap();
        let mut sched: Scheduler<(u32, u64)> = Scheduler::new();
        let mut driver = FrameDriver::new(1_000);
        let dt = 1.0 / f64::from(fps);
        let mut log = Vec::new();
        let horizon = until.saturating_add(SimDuration::from_millis(100));
        while driver.now() < horizon {
            let report = driver.step(dt, 10).unwrap();
            for k in rates.collect_until(report.end) {
                sched.schedule(k.time, k.priority, (k.rate.index(), k.index));
            }
            for f in sched.advance_to(report.end) {
                if f.time <= until {
                    log.push((f.time.as_micros(), f.priority, f.id.sequence()));
                }
            }
        }
        log.into_iter().map(|(t, p, _)| (t, p, 0)).collect()
    }

    #[test]
    fn identical_event_times_at_20_60_144_fps() {
        let until = SimTime::from_secs(10);
        let a = simulate(20, until);
        let b = simulate(60, until);
        let c = simulate(144, until);
        assert!(a.len() > 10_000);
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn no_drift_over_many_frames() {
        for fps in [20u32, 60, 144] {
            let mut d = FrameDriver::new(0);
            let frames = u64::from(fps) * 3_600;
            for _ in 0..frames {
                d.step(1.0 / f64::from(fps), 0).unwrap();
            }
            let err = d.now().signed_micros_since(SimTime::from_secs(3_600)).abs();
            assert!(err <= 2, "fps {fps} drift {err} us");
        }
    }

    #[test]
    fn fractional_remainder_is_kept() {
        let mut d = FrameDriver::new(0);
        let r1 = d.step_exact(SimDuration::from_micros(3), 0).unwrap();
        assert_eq!(r1.advanced().as_micros(), 3);
        let mut d = FrameDriver::new(0);
        let mut total = 0;
        for _ in 0..3 {
            total += d.step(0.000_000_333_333, 0).unwrap().advanced().as_micros();
        }
        assert_eq!(total, d.now().as_micros());
        for _ in 0..1_000_000 {
            d.step(0.000_000_333_333, 0).unwrap();
        }
        let expect = 1_000_003.0 * 0.333_333;
        assert!((d.now().as_micros() as f64 - expect).abs() <= 1.0);
    }

    #[test]
    fn pause_advances_nothing() {
        let mut d = FrameDriver::new(5);
        let r = d.step(0.0, 0).unwrap();
        assert_eq!(r.advanced(), SimDuration::ZERO);
        assert_eq!(d.now(), SimTime::ZERO);
        assert_eq!(d.frames(), 1);
    }

    #[test]
    fn speed_multiplier_scales_exactly() {
        let mut a = FrameDriver::new(0);
        let mut b = FrameDriver::new(0);
        b.set_speed(4);
        for _ in 0..1_000 {
            a.step_exact(SimDuration::from_micros(7_001), 0).unwrap();
            b.step_exact(SimDuration::from_micros(7_001), 0).unwrap();
        }
        assert_eq!(b.now().as_micros(), 4 * a.now().as_micros());
        assert_eq!(b.speed(), 4);
    }

    #[test]
    fn invalid_dt_rejected_and_state_unchanged() {
        let mut d = FrameDriver::new(0);
        d.step(0.01, 0).unwrap();
        let now = d.now();
        assert_eq!(d.step(-0.1, 0), Err(FrameError::InvalidDt));
        assert_eq!(d.step(f64::NAN, 0), Err(FrameError::InvalidDt));
        assert_eq!(d.step(f64::INFINITY, 0), Err(FrameError::InvalidDt));
        assert_eq!(d.step(1.0e40, 0), Err(FrameError::TimeOverflow));
        assert_eq!(d.now(), now);
        assert_eq!(d.frames(), 1);
    }

    #[test]
    fn overrun_reported_without_stretching() {
        let mut d = FrameDriver::new(100);
        let ok = d.step(0.016, 100).unwrap();
        assert!(ok.overrun.is_none());
        let bad = d.step(0.016, 250).unwrap();
        let o = bad.overrun.unwrap();
        assert_eq!((o.cost, o.budget, o.excess, o.frame), (250, 100, 150, 1));
        assert_eq!(bad.advanced().as_micros(), 16_000);
        assert_eq!(d.overrun_count(), 1);
        assert_eq!(d.worst_excess(), 150);
        assert_eq!(d.last_overrun(), Some(o));
        d.set_budget(1_000);
        assert!(d.step(0.016, 250).unwrap().overrun.is_none());
        assert_eq!(d.budget(), 1_000);
    }

    #[test]
    fn drive_runs_scheduler_to_frame_end() {
        let mut s = Scheduler::new();
        s.schedule(SimTime::from_millis(5), 0, 'a');
        s.schedule(SimTime::from_millis(25), 0, 'b');
        let mut d = FrameDriver::new(10);
        let (r, fired) = d.drive(&mut s, 0.016, 0).unwrap();
        assert_eq!(fired.len(), 1);
        assert_eq!(s.now(), r.end);
        let (_, fired) = d.drive(&mut s, 0.016, 0).unwrap();
        assert_eq!(fired[0].event, 'b');
    }

    #[test]
    fn starting_at_offsets_time() {
        let mut d = FrameDriver::starting_at(SimTime::from_secs(10), 0);
        d.step_exact(SimDuration::from_secs(1), 0).unwrap();
        assert_eq!(d.now(), SimTime::from_secs(11));
        assert_eq!(d.fraction_units(), 0);
    }
}
