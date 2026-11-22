pub const PCM_SWITCH_V: f64 = 18.0;
pub const HBB_HOLDUP_US: u64 = 11_000_000;
pub const PCM_OUT_V: f64 = 12.5;
pub const APPROX_RESET_OFF_US: u64 = 500_000;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pcm {
    pub on: bool,
    pub failed: bool,
    battery_since: Option<u64>,
    reset_until: u64,
    pub shutdowns: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PcmEvent {
    None,
    PowerUp,
    SaveAndShutdown,
    HardOff,
}

impl Pcm {
    pub fn on_battery(&self) -> bool {
        self.on && self.battery_since.is_some()
    }

    pub fn holdup_left_us(&self, now_us: u64) -> u64 {
        self.battery_since
            .map(|t| HBB_HOLDUP_US.saturating_sub(now_us.saturating_sub(t)))
            .unwrap_or(0)
    }

    pub fn press_reset(&mut self, now_us: u64) {
        self.reset_until = now_us + APPROX_RESET_OFF_US;
    }

    pub fn step(&mut self, now_us: u64, primary_v: f64, battery_v: f64) -> PcmEvent {
        let was = self.on;
        if self.failed || now_us < self.reset_until {
            self.on = false;
            self.battery_since = None;
            return if was { PcmEvent::HardOff } else { PcmEvent::None };
        }
        if primary_v >= PCM_SWITCH_V {
            self.on = true;
            self.battery_since = None;
        } else if self.on {
            let since = *self.battery_since.get_or_insert(now_us);
            if battery_v < PCM_SWITCH_V || now_us.saturating_sub(since) >= HBB_HOLDUP_US {
                self.on = false;
                self.battery_since = None;
                self.shutdowns += 1;
                return PcmEvent::SaveAndShutdown;
            }
        }
        if self.on && !was {
            PcmEvent::PowerUp
        } else {
            PcmEvent::None
        }
    }
}
