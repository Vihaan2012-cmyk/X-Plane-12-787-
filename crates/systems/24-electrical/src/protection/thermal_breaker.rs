pub const HEATING_TAU_S: f64 = 30.0;
pub const COOLING_TAU_S: f64 = 60.0;
pub const THETA_AT_RATING: f64 = 0.7;
pub const AMBIENT_REF_C: f64 = 25.0;
pub const AMBIENT_THETA_PER_C: f64 = 0.003;
pub const RESET_THETA_MAX: f64 = 0.9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Tripped,
    Pulled,
}

#[derive(Clone, Copy, Debug)]
pub struct BreakerInput {
    pub dt_us: u64,
    pub current_a: f64,
    pub ambient_c: f64,
    pub pull: bool,
    pub push: bool,
}

impl Default for BreakerInput {
    fn default() -> Self {
        Self {
            dt_us: 10_000,
            current_a: 0.0,
            ambient_c: AMBIENT_REF_C,
            pull: false,
            push: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BreakerOutput {
    pub state: BreakerState,
    pub closed: bool,
    pub theta: f64,
    pub current_a: f64,
}

#[derive(Clone, Debug)]
pub struct ThermalBreaker {
    rating_a: f64,
    theta: f64,
    state: BreakerState,
}

impl ThermalBreaker {
    pub fn new(rating_a: f64) -> Self {
        Self {
            rating_a,
            theta: 0.0,
            state: BreakerState::Closed,
        }
    }

    pub fn rating_a(&self) -> f64 {
        self.rating_a
    }

    pub fn state(&self) -> BreakerState {
        self.state
    }

    pub fn theta(&self) -> f64 {
        self.theta
    }

    pub fn step(&mut self, input: &BreakerInput) -> BreakerOutput {
        let dt_s = input.dt_us as f64 * 1e-6;
        let amb = (input.ambient_c - AMBIENT_REF_C) * AMBIENT_THETA_PER_C;
        let closed_now = self.state == BreakerState::Closed;
        let i = if closed_now { input.current_a.abs() } else { 0.0 };
        let ratio = i / self.rating_a;
        let (theta_ss, tau) = if closed_now {
            let ss = THETA_AT_RATING * ratio * ratio + amb;
            (
                ss,
                if ss >= self.theta {
                    HEATING_TAU_S
                } else {
                    COOLING_TAU_S
                },
            )
        } else {
            (amb.max(0.0), COOLING_TAU_S)
        };
        self.theta = theta_ss + (self.theta - theta_ss) * (-dt_s / tau).exp();
        if input.pull {
            self.state = BreakerState::Pulled;
        } else if closed_now && self.theta >= 1.0 {
            self.state = BreakerState::Tripped;
        } else if input.push && self.state != BreakerState::Closed && self.theta < RESET_THETA_MAX {
            self.state = BreakerState::Closed;
        }
        let closed = self.state == BreakerState::Closed;
        BreakerOutput {
            state: self.state,
            closed,
            theta: self.theta,
            current_a: if closed { input.current_a } else { 0.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trip_time_s(mult: f64) -> Option<f64> {
        let mut b = ThermalBreaker::new(5.0);
        let inp = BreakerInput {
            current_a: 5.0 * mult,
            ..Default::default()
        };
        for n in 1..=1_000_000u64 {
            if b.step(&inp).state == BreakerState::Tripped {
                return Some(n as f64 * 0.01);
            }
        }
        None
    }

    #[test]
    fn no_trip_at_rating() {
        assert_eq!(trip_time_s(1.0), None);
    }

    #[test]
    fn ms_band_135_percent_trips_within_hour_and_200_percent_in_seconds() {
        let t135 = trip_time_s(1.35).unwrap();
        assert!(t135 > 10.0 && t135 < 3600.0, "{t135}");
        let t2 = trip_time_s(2.0).unwrap();
        assert!(t2 > 5.0 && t2 < 20.0, "{t2}");
        let t6 = trip_time_s(6.0).unwrap();
        assert!(t6 < 2.0, "{t6}");
        assert!(t135 > t2 && t2 > t6);
    }

    #[test]
    fn hot_ambient_trips_sooner() {
        let mut b = ThermalBreaker::new(5.0);
        let mut h = ThermalBreaker::new(5.0);
        let cold = BreakerInput {
            current_a: 10.0,
            ..Default::default()
        };
        let hot = BreakerInput {
            ambient_c: 70.0,
            ..cold
        };
        let mut tc = 0;
        let mut th = 0;
        while b.step(&cold).state == BreakerState::Closed {
            tc += 1;
        }
        while h.step(&hot).state == BreakerState::Closed {
            th += 1;
        }
        assert!(th < tc);
    }

    #[test]
    fn pull_and_reset() {
        let mut b = ThermalBreaker::new(5.0);
        let o = b.step(&BreakerInput {
            pull: true,
            current_a: 3.0,
            ..Default::default()
        });
        assert_eq!(o.state, BreakerState::Pulled);
        assert_eq!(o.current_a, 0.0);
        let o = b.step(&BreakerInput {
            push: true,
            current_a: 3.0,
            ..Default::default()
        });
        assert!(o.closed);
        assert_eq!(o.current_a, 3.0);
    }

    #[test]
    fn trip_free_cannot_reset_while_hot() {
        let mut b = ThermalBreaker::new(5.0);
        let inp = BreakerInput {
            current_a: 15.0,
            ..Default::default()
        };
        while b.step(&inp).state == BreakerState::Closed {}
        let o = b.step(&BreakerInput {
            push: true,
            ..Default::default()
        });
        assert_eq!(o.state, BreakerState::Tripped);
        let mut n = 0;
        loop {
            let o = b.step(&BreakerInput {
                push: true,
                ..Default::default()
            });
            n += 1;
            if o.closed {
                break;
            }
            assert!(n < 100_000);
        }
        assert!(b.theta() < RESET_THETA_MAX);
    }

    #[test]
    fn push_held_during_overload_retrips() {
        let mut b = ThermalBreaker::new(5.0);
        let inp = BreakerInput {
            current_a: 50.0,
            push: true,
            ..Default::default()
        };
        let mut trips = 0;
        for _ in 0..1_000 {
            if b.step(&inp).state == BreakerState::Tripped {
                trips += 1;
            }
        }
        assert!(trips > 0);
    }
}
