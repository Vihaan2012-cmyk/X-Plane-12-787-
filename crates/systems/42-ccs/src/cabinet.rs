use crate::pcm::{Pcm, PcmEvent};

pub const FULL_PBIT_MIN_US: u64 = 120_000_000;
pub const FULL_PBIT_MAX_US: u64 = 180_000_000;
pub const INHIBITED_DISPLAYS_US: u64 = 60_000_000;
pub const INHIBITED_HOSTED_MIN_US: u64 = 55_000_000;
pub const INHIBITED_HOSTED_MAX_US: u64 = 75_000_000;
pub const GPM_COUNT: usize = 8;
pub const APPROX_BOOT_W: f64 = 220.0;
pub const APPROX_RUN_W: f64 = 250.0;
pub const APPROX_FAN_W: f64 = 40.0;
pub const APPROX_HEAT_CAP_J_PER_K: f64 = 40_000.0;
pub const APPROX_G_DUCT_W_PER_K: f64 = 40.0;
pub const APPROX_G_FAN_W_PER_K: f64 = 22.0;
pub const APPROX_G_NONE_W_PER_K: f64 = 4.0;
pub const APPROX_AAF_OVERTEMP_C: f64 = 55.0;
pub const APPROX_GROUND_SHUTDOWN_C: f64 = 65.0;
pub const APPROX_GPM_STOP_C: f64 = 75.0;
pub const APPROX_GPM_RESTART_C: f64 = 60.0;
pub const APPROX_GPM_TEMP_SPREAD_K: f64 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    L,
    R,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartMode {
    Uninhibited,
    Inhibited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CabinetState {
    Off,
    Booting { mode: StartMode, since_us: u64 },
    Operational { mode: StartMode, since_us: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gpm {
    pub failed: bool,
    pub overtemp_stop: bool,
    pub temp_offset_k: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cabinet {
    pub side: Side,
    pub pcm: [Pcm; 2],
    pub state: CabinetState,
    pub gpm: [Gpm; GPM_COUNT],
    pub temp_c: f64,
    pub fan_on: bool,
    pub valve_closed: bool,
    pub fan_failed: bool,
    pub valve_failed: bool,
    pub thermal_off: bool,
    pub nvm_saves: u32,
    pub hard_offs: u32,
    pub boots: u32,
    pbit_us: u64,
    inhibited_hosted_us: u64,
    pub startup_events: u32,
}

pub struct CabinetInput {
    pub now_us: u64,
    pub dt_us: u64,
    pub primary_v: [f64; 2],
    pub battery_v: f64,
    pub on_ground: bool,
    pub inhibit: bool,
    pub cooling_supply_ok: bool,
    pub cooling_air_c: f64,
    pub ambient_c: f64,
    pub lps_available: bool,
}

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn frac(seed: u64, k: u64) -> f64 {
    (mix(seed ^ mix(k)) >> 11) as f64 / (1u64 << 53) as f64
}

impl Cabinet {
    pub fn new(side: Side, seed: u64, ambient_c: f64) -> Cabinet {
        let s = seed ^ if side == Side::L { 0x4C } else { 0x52 };
        let span = |lo: u64, hi: u64, k: u64| lo + ((hi - lo) as f64 * frac(s, k)) as u64;
        let mut gpm = [Gpm {
            failed: false,
            overtemp_stop: false,
            temp_offset_k: 0.0,
        }; GPM_COUNT];
        for (i, g) in gpm.iter_mut().enumerate() {
            g.temp_offset_k = APPROX_GPM_TEMP_SPREAD_K * frac(s, 100 + i as u64);
        }
        Cabinet {
            side,
            pcm: [Pcm::default(); 2],
            state: CabinetState::Off,
            gpm,
            temp_c: ambient_c,
            fan_on: false,
            valve_closed: false,
            fan_failed: false,
            valve_failed: false,
            thermal_off: false,
            nvm_saves: 0,
            hard_offs: 0,
            boots: 0,
            pbit_us: span(FULL_PBIT_MIN_US, FULL_PBIT_MAX_US, 1),
            inhibited_hosted_us: span(INHIBITED_HOSTED_MIN_US, INHIBITED_HOSTED_MAX_US, 2),
            startup_events: 0,
        }
    }

    pub fn letter(&self) -> char {
        if self.side == Side::L { 'L' } else { 'R' }
    }

    pub fn powered(&self) -> bool {
        self.pcm.iter().any(|p| p.on)
    }

    pub fn on_battery(&self) -> bool {
        self.powered() && self.pcm.iter().filter(|p| p.on).all(|p| p.on_battery())
    }

    pub fn boot_elapsed_us(&self, now_us: u64) -> Option<u64> {
        match self.state {
            CabinetState::Booting { since_us, .. } => Some(now_us.saturating_sub(since_us)),
            _ => None,
        }
    }

    pub fn mode(&self) -> Option<StartMode> {
        match self.state {
            CabinetState::Off => None,
            CabinetState::Booting { mode, .. } | CabinetState::Operational { mode, .. } => Some(mode),
        }
    }

    fn hosted_after(&self, mode: StartMode) -> u64 {
        match mode {
            StartMode::Uninhibited => self.pbit_us,
            StartMode::Inhibited => self.inhibited_hosted_us,
        }
    }

    fn displays_after(&self, mode: StartMode) -> u64 {
        match mode {
            StartMode::Uninhibited => self.pbit_us,
            StartMode::Inhibited => INHIBITED_DISPLAYS_US,
        }
    }

    pub fn hosted_ready(&self) -> bool {
        matches!(self.state, CabinetState::Operational { .. })
    }

    pub fn displays_ready(&self, now_us: u64) -> bool {
        match self.state {
            CabinetState::Operational { .. } => true,
            CabinetState::Booting { mode, since_us } => {
                now_us.saturating_sub(since_us) >= self.displays_after(mode)
            }
            CabinetState::Off => false,
        }
    }

    pub fn gpm_running(&self, i: usize) -> bool {
        self.hosted_ready() && !self.gpm[i].failed && !self.gpm[i].overtemp_stop
    }

    pub fn power_w(&self) -> f64 {
        let base = match self.state {
            CabinetState::Off => 0.0,
            CabinetState::Booting { .. } => APPROX_BOOT_W,
            CabinetState::Operational { .. } => APPROX_RUN_W,
        };
        if !self.powered() {
            return 0.0;
        }
        base + if self.fan_on { APPROX_FAN_W } else { 0.0 }
    }

    pub fn press_reset(&mut self, now_us: u64) {
        for p in self.pcm.iter_mut() {
            p.press_reset(now_us);
        }
    }

    pub fn step(&mut self, i: &CabinetInput) {
        let mut ev = [PcmEvent::None; 2];
        let off = self.thermal_off;
        for ((e, p), v) in ev.iter_mut().zip(self.pcm.iter_mut()).zip(i.primary_v) {
            let primary = if off { 0.0 } else { v };
            let battery = if off { 0.0 } else { i.battery_v };
            *e = p.step(i.now_us, primary, battery);
        }
        if !self.powered() {
            if self.state != CabinetState::Off {
                if ev.contains(&PcmEvent::HardOff) {
                    self.hard_offs += 1;
                } else {
                    self.nvm_saves += 1;
                }
            }
            self.state = CabinetState::Off;
        } else if self.state == CabinetState::Off {
            let mode = if i.inhibit {
                StartMode::Inhibited
            } else {
                StartMode::Uninhibited
            };
            self.state = CabinetState::Booting {
                mode,
                since_us: i.now_us,
            };
            self.boots += 1;
        }
        if let CabinetState::Booting { mode, since_us } = self.state
            && i.now_us.saturating_sub(since_us) >= self.hosted_after(mode).max(self.displays_after(mode))
        {
            self.state = CabinetState::Operational {
                mode,
                since_us: i.now_us,
            };
            self.startup_events += 1;
        }
        self.step_thermal(i);
    }

    fn step_thermal(&mut self, i: &CabinetInput) {
        let dt = i.dt_us as f64 * 1.0e-6;
        let running = self.hosted_ready();
        let lps_low = !i.cooling_supply_ok && i.lps_available;
        let aaf_hot = running && self.temp_c > APPROX_AAF_OVERTEMP_C;
        let want_aux = self.powered() && (lps_low || aaf_hot);
        self.fan_on = want_aux && !self.fan_failed;
        self.valve_closed = if self.valve_failed {
            self.valve_closed
        } else {
            want_aux
        };
        let duct = i.cooling_supply_ok && !self.valve_closed;
        let (g, air) = if duct {
            (APPROX_G_DUCT_W_PER_K, i.cooling_air_c)
        } else if self.fan_on {
            (APPROX_G_FAN_W_PER_K, i.ambient_c)
        } else {
            (APPROX_G_NONE_W_PER_K, i.ambient_c)
        };
        let heat = self.power_w();
        self.temp_c += (heat - g * (self.temp_c - air)) * dt / APPROX_HEAT_CAP_J_PER_K;
        for gp in self.gpm.iter_mut() {
            let t = self.temp_c + gp.temp_offset_k;
            if t >= APPROX_GPM_STOP_C {
                gp.overtemp_stop = true;
            } else if t <= APPROX_GPM_RESTART_C {
                gp.overtemp_stop = false;
            }
        }
        if i.on_ground && running && self.temp_c >= APPROX_GROUND_SHUTDOWN_C {
            self.thermal_off = true;
        } else if self.thermal_off && self.temp_c <= APPROX_GPM_RESTART_C {
            self.thermal_off = false;
        }
    }
}
