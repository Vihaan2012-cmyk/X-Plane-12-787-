use std::collections::BTreeMap;

pub const IS_PARTITIONS: [&str; 6] = [
    "CCRSM",
    "CCSHM",
    "DATA LOAD",
    "TIME MANAGER",
    "FIDO",
    "FILE SYSTEM",
];
pub const APPROX_MAJOR_FRAME_US: u64 = 25_000;
pub const APPROX_IS_WINDOW_US: u64 = 1_000;
pub const APPROX_PARTITION_RESTART_US: u64 = 2_000_000;
pub const APPROX_RESTARTS_BEFORE_HOLD: u32 = 3;
pub const APPROX_RESTART_WINDOW_US: u64 = 60_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartState {
    Running,
    Restarting { until_us: u64 },
    Held,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Partition {
    pub name: String,
    pub hosted: bool,
    pub window_us: u64,
    pub offset_us: u64,
    pub state: PartState,
    pub crash_requested: bool,
    pub overrun_requested: bool,
    pub restarts: Vec<u64>,
    pub overruns: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GpmSoftware {
    pub name: String,
    pub partitions: Vec<Partition>,
    pub lane_failed: [bool; 2],
    pub data_load: bool,
    pub health_events: u64,
}

impl GpmSoftware {
    pub fn new(name: &str, hosted: &BTreeMap<String, Vec<String>>) -> GpmSoftware {
        let mut names: Vec<(String, bool)> = IS_PARTITIONS.iter().map(|s| (s.to_string(), false)).collect();
        for (app, gpms) in hosted {
            if gpms.iter().any(|g| g == name) {
                names.push((app.clone(), true));
            }
        }
        let hosted_n = names.iter().filter(|n| n.1).count().max(1) as u64;
        let app_window =
            (APPROX_MAJOR_FRAME_US - APPROX_IS_WINDOW_US * IS_PARTITIONS.len() as u64) / hosted_n;
        let mut off = 0;
        let mut partitions = Vec::new();
        for (n, h) in names {
            let w = if h { app_window } else { APPROX_IS_WINDOW_US };
            partitions.push(Partition {
                name: n,
                hosted: h,
                window_us: w,
                offset_us: off,
                state: PartState::Running,
                crash_requested: false,
                overrun_requested: false,
                restarts: Vec::new(),
                overruns: 0,
            });
            off += w;
        }
        GpmSoftware {
            name: name.to_string(),
            partitions,
            lane_failed: [false; 2],
            data_load: false,
            health_events: 0,
        }
    }

    pub fn hardware_ok(&self) -> bool {
        !(self.lane_failed[0] && self.lane_failed[1])
    }

    pub fn single_lane(&self) -> bool {
        self.lane_failed[0] != self.lane_failed[1]
    }

    pub fn schedule_fits(&self) -> bool {
        let used: u64 = self.partitions.iter().map(|p| p.window_us).sum();
        used <= APPROX_MAJOR_FRAME_US
    }

    pub fn running(&self, part: &str) -> bool {
        if self.data_load && part != "DATA LOAD" {
            return false;
        }
        if !self.data_load && part == "DATA LOAD" {
            return false;
        }
        self.partitions
            .iter()
            .any(|p| p.name == part && p.state == PartState::Running)
    }

    pub fn partition_mut(&mut self, part: &str) -> Option<&mut Partition> {
        self.partitions.iter_mut().find(|p| p.name == part)
    }

    pub fn reset(&mut self) {
        for p in self.partitions.iter_mut() {
            p.state = PartState::Running;
            p.restarts.clear();
        }
        self.data_load = false;
    }

    pub fn step(&mut self, now_us: u64) {
        let mut events = 0;
        for p in self.partitions.iter_mut() {
            if p.overrun_requested && p.state == PartState::Running {
                p.overruns += 1;
                p.crash_requested = true;
            }
            match p.state {
                PartState::Running if p.crash_requested => {
                    p.restarts
                        .retain(|t| now_us.saturating_sub(*t) < APPROX_RESTART_WINDOW_US);
                    p.restarts.push(now_us);
                    events += 1;
                    p.state = if p.restarts.len() as u32 >= APPROX_RESTARTS_BEFORE_HOLD {
                        PartState::Held
                    } else {
                        PartState::Restarting {
                            until_us: now_us + APPROX_PARTITION_RESTART_US,
                        }
                    };
                }
                PartState::Restarting { until_us } if now_us >= until_us => {
                    p.state = PartState::Running;
                }
                _ => {}
            }
            p.crash_requested = false;
        }
        self.health_events += events;
    }
}
