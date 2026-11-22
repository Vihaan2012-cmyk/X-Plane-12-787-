use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Signal {
    pub value: f64,
    pub valid: bool,
    pub timestamp_us: u64,
}

#[derive(Clone, Debug, Default)]
pub struct AppIo {
    inputs: BTreeMap<String, Signal>,
    outputs: BTreeMap<String, Signal>,
}

impl AppIo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_input(&mut self, name: &str, value: f64, valid: bool, timestamp_us: u64) {
        self.inputs.insert(
            name.to_string(),
            Signal {
                value,
                valid,
                timestamp_us,
            },
        );
    }

    pub fn input(&self, name: &str) -> Option<Signal> {
        self.inputs.get(name).copied()
    }

    pub fn input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).filter(|s| s.valid).map(|s| s.value)
    }

    pub fn write(&mut self, name: &str, value: f64, now_us: u64) {
        self.outputs.insert(
            name.to_string(),
            Signal {
                value,
                valid: true,
                timestamp_us: now_us,
            },
        );
    }

    pub fn write_invalid(&mut self, name: &str, now_us: u64) {
        let value = self.outputs.get(name).map_or(0.0, |s| s.value);
        self.outputs.insert(
            name.to_string(),
            Signal {
                value,
                valid: false,
                timestamp_us: now_us,
            },
        );
    }

    pub fn output(&self, name: &str) -> Option<Signal> {
        self.outputs.get(name).copied()
    }

    pub fn output_names(&self) -> Vec<&str> {
        self.outputs.keys().map(String::as_str).collect()
    }

    pub fn input_names(&self) -> Vec<&str> {
        self.inputs.keys().map(String::as_str).collect()
    }

    pub fn invalidate_outputs(&mut self) {
        for s in self.outputs.values_mut() {
            s.valid = false;
        }
    }
}

pub trait HostedApp {
    fn name(&self) -> &str;
    fn period_us(&self) -> u64;
    fn budget_us(&self) -> u64;
    fn run(&mut self, now_us: u64, io: &mut AppIo);
    fn exec_time_us(&self) -> u64 {
        self.budget_us()
    }
    fn reset(&mut self) {}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub app: String,
    pub offset_us: u64,
    pub duration_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schedule {
    pub major_frame_us: u64,
    pub windows: Vec<Window>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    ZeroFrame,
    WindowOutsideFrame,
    WindowsOverlap,
}

impl Schedule {
    pub fn new(major_frame_us: u64) -> Self {
        Self {
            major_frame_us,
            windows: Vec::new(),
        }
    }

    pub fn window(mut self, app: &str, offset_us: u64, duration_us: u64) -> Self {
        self.windows.push(Window {
            app: app.to_string(),
            offset_us,
            duration_us,
        });
        self
    }

    pub fn validate(&self) -> Result<(), ScheduleError> {
        if self.major_frame_us == 0 {
            return Err(ScheduleError::ZeroFrame);
        }
        let mut sorted: Vec<&Window> = self.windows.iter().collect();
        sorted.sort_by_key(|w| w.offset_us);
        let mut end = 0;
        for w in sorted {
            if w.offset_us + w.duration_us > self.major_frame_us {
                return Err(ScheduleError::WindowOutsideFrame);
            }
            if w.offset_us < end {
                return Err(ScheduleError::WindowsOverlap);
            }
            end = w.offset_us + w.duration_us;
        }
        Ok(())
    }

    pub fn windows_between(&self, origin_us: u64, from_us: u64, to_us: u64) -> Vec<(u64, usize)> {
        let mut out = Vec::new();
        if self.major_frame_us == 0 || to_us < from_us || to_us < origin_us {
            return out;
        }
        let mf = self.major_frame_us;
        let first = from_us.saturating_sub(origin_us) / mf;
        let last = (to_us - origin_us) / mf;
        let mut order: Vec<usize> = (0..self.windows.len()).collect();
        order.sort_by_key(|&i| (self.windows[i].offset_us, i));
        for f in first..=last {
            for &i in &order {
                let start = origin_us + f * mf + self.windows[i].offset_us;
                if start >= from_us && start <= to_us {
                    out.push((start, i));
                }
            }
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverrunAction {
    LogOnly,
    RestartPartition,
    ResetBox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecRecord {
    pub time_us: u64,
    pub slot: usize,
    pub overrun: bool,
}
