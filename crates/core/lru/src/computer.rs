use crate::app::{AppIo, ExecRecord, HostedApp, OverrunAction, Schedule, ScheduleError, Signal};
use crate::clock::{BoxClock, ClockConfig, OverflowAction, PoweredCounter};
use crate::identity::{Expected, Identity, IdentityMismatch};
use crate::nvm::{Nvm, NvmValue, Severity};
use crate::power::{PowerConfig, PowerInput, PowerStage, PowerStatus};

pub const FAULT_POWER_LOSS: u32 = 1;
pub const FAULT_WATCHDOG: u32 = 2;
pub const FAULT_OVERRUN: u32 = 3;
pub const FAULT_CONFIG_MISMATCH: u32 = 4;
pub const FAULT_POST: u32 = 5;
pub const FAULT_COUNTER_OVERFLOW: u32 = 6;
pub const FAULT_COMMANDED_RESET: u32 = 7;
pub const FAULT_DATA_LOAD: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootStage {
    PowerGood,
    Post,
    ConfigCheck,
    SoftwareCheck,
    Init,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Off,
    PowerOnReset,
    Booting(BootStage),
    Operational,
    Degraded,
    Failed,
    Bite,
    DataLoad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetCause {
    PowerUp,
    PowerLoss,
    Watchdog,
    Overrun,
    Commanded,
    CounterOverflow,
    DataLoad,
}

#[derive(Clone, Debug)]
pub struct BootTimings {
    pub por_us: u64,
    pub power_good_us: u64,
    pub post_us: u64,
    pub config_us: u64,
    pub software_us: u64,
    pub init_us: u64,
}

impl Default for BootTimings {
    fn default() -> Self {
        Self {
            por_us: 100_000,
            power_good_us: 50_000,
            post_us: 2_000_000,
            config_us: 200_000,
            software_us: 300_000,
            init_us: 1_000_000,
        }
    }
}

impl BootTimings {
    pub fn total_us(&self) -> u64 {
        self.por_us + self.power_good_us + self.post_us + self.config_us + self.software_us + self.init_us
    }

    fn stage_end_offsets(&self) -> [(BootStage, u64); 5] {
        let a = self.por_us + self.power_good_us;
        let b = a + self.post_us;
        let c = b + self.config_us;
        let d = c + self.software_us;
        let e = d + self.init_us;
        [
            (BootStage::PowerGood, a),
            (BootStage::Post, b),
            (BootStage::ConfigCheck, c),
            (BootStage::SoftwareCheck, d),
            (BootStage::Init, e),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct LruConfig {
    pub identity: Identity,
    pub expected: Expected,
    pub power: PowerConfig,
    pub boot: BootTimings,
    pub watchdog_timeout_us: u64,
    pub nvm_endurance: u32,
    pub fault_log_capacity: usize,
    pub overrun_action: OverrunAction,
    pub partition_restart_us: u64,
    pub clock: ClockConfig,
    pub counter_bits: u32,
    pub counter_tick_us: u64,
    pub counter_overflow: OverflowAction,
    pub min_start_temp_c: f64,
    pub bite_duration_us: u64,
    pub load_w: f64,
}

impl LruConfig {
    pub fn new(identity: Identity) -> Self {
        let expected = Expected::matching(&identity);
        Self {
            identity,
            expected,
            power: PowerConfig::default(),
            boot: BootTimings::default(),
            watchdog_timeout_us: 500_000,
            nvm_endurance: 100_000,
            fault_log_capacity: 64,
            overrun_action: OverrunAction::RestartPartition,
            partition_restart_us: 200_000,
            clock: ClockConfig::default(),
            counter_bits: 32,
            counter_tick_us: 1_000,
            counter_overflow: OverflowAction::Wrap,
            min_start_temp_c: -40.0,
            bite_duration_us: 3_000_000,
            load_w: 20.0,
        }
    }
}

struct Slot {
    app: Box<dyn HostedApp>,
    io: AppIo,
    last_run_us: Option<u64>,
    live: bool,
    restart_until_us: u64,
    overruns: u32,
    runs: u32,
}

pub struct Lru {
    cfg: LruConfig,
    identity: Identity,
    power: PowerStage,
    mode: Mode,
    mode_since_us: u64,
    boot_start_us: u64,
    boot_stage_idx: usize,
    prior_mode: Mode,
    mode_end_us: u64,
    pending_identity: Option<Identity>,
    nvm: Nvm,
    clock: BoxClock,
    counter: PoweredCounter,
    slots: Vec<Slot>,
    schedule: Schedule,
    origin_us: u64,
    cursor_us: u64,
    last_kick_us: u64,
    hung: bool,
    last_us: u64,
    resets: u32,
    last_reset: Option<ResetCause>,
    mismatches: Vec<IdentityMismatch>,
    post_fault: bool,
    active_faults: Vec<(u32, Severity)>,
    trace: Vec<ExecRecord>,
    started: bool,
}

const TRACE_CAP: usize = 1024;

impl Lru {
    pub fn new(cfg: LruConfig) -> Self {
        let nvm = Nvm::new(cfg.nvm_endurance, cfg.fault_log_capacity);
        Self {
            identity: cfg.identity.clone(),
            power: PowerStage::new(cfg.power.clone()),
            clock: BoxClock::new(cfg.clock.clone()),
            counter: PoweredCounter::new(cfg.counter_bits, cfg.counter_tick_us),
            cfg,
            mode: Mode::Off,
            mode_since_us: 0,
            boot_start_us: 0,
            boot_stage_idx: 0,
            prior_mode: Mode::Off,
            mode_end_us: 0,
            pending_identity: None,
            nvm,
            slots: Vec::new(),
            schedule: Schedule::new(1),
            origin_us: 0,
            cursor_us: 0,
            last_kick_us: 0,
            hung: false,
            last_us: 0,
            resets: 0,
            last_reset: None,
            mismatches: Vec::new(),
            post_fault: false,
            active_faults: Vec::new(),
            trace: Vec::new(),
            started: false,
        }
    }

    pub fn add_app(&mut self, app: Box<dyn HostedApp>) {
        self.slots.push(Slot {
            app,
            io: AppIo::new(),
            last_run_us: None,
            live: false,
            restart_until_us: 0,
            overruns: 0,
            runs: 0,
        });
    }

    pub fn set_schedule(&mut self, schedule: Schedule) -> Result<(), ScheduleError> {
        schedule.validate()?;
        self.schedule = schedule;
        Ok(())
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn is_running(&self) -> bool {
        matches!(self.mode, Mode::Operational | Mode::Degraded)
    }

    pub fn is_no_go(&self) -> bool {
        !self.mismatches.is_empty() || self.mode == Mode::Failed
    }

    pub fn mismatches(&self) -> &[IdentityMismatch] {
        &self.mismatches
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    pub fn config(&self) -> &LruConfig {
        &self.cfg
    }

    pub fn power(&self) -> &PowerStage {
        &self.power
    }

    pub fn power_mut(&mut self) -> &mut PowerStage {
        &mut self.power
    }

    pub fn nvm(&self) -> &Nvm {
        &self.nvm
    }

    pub fn nvm_mut(&mut self) -> &mut Nvm {
        &mut self.nvm
    }

    pub fn clock(&self) -> &BoxClock {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut BoxClock {
        &mut self.clock
    }

    pub fn counter(&self) -> &PoweredCounter {
        &self.counter
    }

    pub fn counter_mut(&mut self) -> &mut PoweredCounter {
        &mut self.counter
    }

    pub fn reset_count(&self) -> u32 {
        self.resets
    }

    pub fn last_reset_cause(&self) -> Option<ResetCause> {
        self.last_reset
    }

    pub fn operational_since_us(&self) -> Option<u64> {
        self.is_running().then_some(self.origin_us)
    }

    pub fn trace(&self) -> &[ExecRecord] {
        &self.trace
    }

    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }

    pub fn overruns(&self, app: &str) -> u32 {
        self.slot_index(app).map_or(0, |i| self.slots[i].overruns)
    }

    pub fn runs(&self, app: &str) -> u32 {
        self.slot_index(app).map_or(0, |i| self.slots[i].runs)
    }

    pub fn set_software_hang(&mut self, hung: bool) {
        self.hung = hung;
    }

    pub fn set_post_fault(&mut self, fault: bool) {
        self.post_fault = fault;
    }

    pub fn boot_stage(&self) -> Option<BootStage> {
        match self.mode {
            Mode::Booting(s) => Some(s),
            _ => None,
        }
    }

    pub fn active_faults(&self) -> &[(u32, Severity)] {
        &self.active_faults
    }

    fn slot_index(&self, app: &str) -> Option<usize> {
        self.slots.iter().position(|s| s.app.name() == app)
    }

    pub fn set_input(&mut self, app: &str, name: &str, value: f64, valid: bool, ts_us: u64) {
        if let Some(i) = self.slot_index(app) {
            self.slots[i].io.set_input(name, value, valid, ts_us);
        }
    }

    pub fn output(&self, app: &str, name: &str) -> Option<Signal> {
        let i = self.slot_index(app)?;
        let slot = &self.slots[i];
        let mut sig = slot.io.output(name)?;
        let live = self.is_running() && slot.live && self.last_us >= slot.restart_until_us;
        if !live {
            sig.valid = false;
        }
        Some(sig)
    }

    pub fn report_fault(&mut self, code: u32, severity: Severity, text: &str, now_us: u64) {
        self.nvm.log_fault(now_us, code, severity, text);
        let key = format!("fault_count_{code}");
        let _ = self.nvm.increment(&key);
        if !self.active_faults.iter().any(|(c, _)| *c == code) {
            self.active_faults.push((code, severity));
        }
        if self.is_running() {
            if severity == Severity::Critical {
                self.enter_failed(now_us);
            } else if severity >= Severity::Minor {
                self.set_mode(Mode::Degraded, now_us);
            }
        }
    }

    pub fn clear_fault(&mut self, code: u32, now_us: u64) {
        self.active_faults.retain(|(c, _)| *c != code);
        if self.mode == Mode::Degraded && !self.active_faults.iter().any(|(_, s)| *s >= Severity::Minor) {
            self.set_mode(Mode::Operational, now_us);
        }
    }

    pub fn command_reset(&mut self, now_us: u64) {
        if self.mode != Mode::Off {
            self.reset(ResetCause::Commanded, now_us);
        }
    }

    pub fn start_bite(&mut self, now_us: u64) -> bool {
        if !self.is_running() {
            return false;
        }
        self.prior_mode = self.mode;
        self.mode_end_us = now_us + self.cfg.bite_duration_us;
        self.set_mode(Mode::Bite, now_us);
        true
    }

    pub fn start_data_load(&mut self, new_identity: Identity, duration_us: u64, now_us: u64) -> bool {
        if matches!(self.mode, Mode::Off | Mode::DataLoad) {
            return false;
        }
        self.pending_identity = Some(new_identity);
        self.mode_end_us = now_us + duration_us;
        self.kill_apps();
        self.set_mode(Mode::DataLoad, now_us);
        true
    }

    pub fn resync_clock(&mut self, reference_us: u64) {
        self.clock.resync(reference_us);
    }

    fn set_mode(&mut self, mode: Mode, now_us: u64) {
        if self.mode != mode {
            self.mode = mode;
            self.mode_since_us = now_us;
        }
    }

    fn kill_apps(&mut self) {
        for s in &mut self.slots {
            s.live = false;
            s.last_run_us = None;
            s.io.invalidate_outputs();
        }
    }

    fn enter_failed(&mut self, now_us: u64) {
        self.kill_apps();
        self.set_mode(Mode::Failed, now_us);
    }

    fn start_por(&mut self, cause: ResetCause, now_us: u64) {
        self.last_reset = Some(cause);
        self.kill_apps();
        for s in &mut self.slots {
            s.app.reset();
            s.restart_until_us = 0;
        }
        self.mismatches.clear();
        self.boot_start_us = now_us;
        self.boot_stage_idx = 0;
        self.hung = false;
        let _ = self.nvm.increment("boot_count");
        self.mode = Mode::PowerOnReset;
        self.mode_since_us = now_us;
    }

    fn reset(&mut self, cause: ResetCause, now_us: u64) {
        self.resets += 1;
        let (code, text) = match cause {
            ResetCause::Watchdog => (FAULT_WATCHDOG, "watchdog reset"),
            ResetCause::Overrun => (FAULT_OVERRUN, "partition overrun reset"),
            ResetCause::CounterOverflow => (FAULT_COUNTER_OVERFLOW, "counter overflow reset"),
            ResetCause::Commanded => (FAULT_COMMANDED_RESET, "commanded reset"),
            ResetCause::DataLoad => (FAULT_DATA_LOAD, "data load complete"),
            _ => (FAULT_POWER_LOSS, "power loss"),
        };
        self.nvm.log_fault(now_us, code, Severity::Major, text);
        let _ = self.nvm.increment("reset_count");
        self.start_por(cause, now_us);
    }

    fn power_down(&mut self, now_us: u64) {
        self.nvm
            .log_fault(now_us, FAULT_POWER_LOSS, Severity::Minor, "power loss");
        let _ = self
            .nvm
            .write("counter_ticks", NvmValue::U64(self.counter.value()));
        self.kill_apps();
        self.counter.reset();
        self.active_faults.clear();
        self.mode = Mode::Off;
        self.mode_since_us = now_us;
    }

    pub fn step(&mut self, now_us: u64, inputs: &[PowerInput], temp_c: f64) {
        let status = self.power.step(now_us, inputs, temp_c, self.cfg.load_w);
        let dt = if self.started {
            now_us.saturating_sub(self.last_us)
        } else {
            0
        };
        self.started = true;
        match status {
            PowerStatus::Lost => {
                if self.mode != Mode::Off {
                    self.resets += 1;
                    self.power_down(now_us);
                }
                self.last_us = now_us;
                return;
            }
            PowerStatus::Good => {
                if self.mode == Mode::Off {
                    if temp_c < self.cfg.min_start_temp_c {
                        self.last_us = now_us;
                        return;
                    }
                    self.clock.power_up(now_us);
                    self.start_por(ResetCause::PowerUp, now_us);
                    self.last_kick_us = now_us;
                }
            }
            PowerStatus::HoldUp => {}
        }
        self.clock.step(now_us, temp_c);
        if self.mode != Mode::Off {
            let wraps = self.counter.advance(dt);
            if wraps > 0 {
                self.nvm.log_fault(
                    now_us,
                    FAULT_COUNTER_OVERFLOW,
                    Severity::Major,
                    "powered counter overflow",
                );
                match self.cfg.counter_overflow {
                    OverflowAction::Wrap => {}
                    OverflowAction::FailBox => self.enter_failed(now_us),
                    OverflowAction::ResetBox => {
                        self.last_us = now_us;
                        self.reset(ResetCause::CounterOverflow, now_us);
                    }
                }
            }
        }
        self.last_us = now_us;
        self.advance_modes(now_us);
    }

    fn advance_modes(&mut self, now_us: u64) {
        loop {
            match self.mode {
                Mode::Off | Mode::Failed => return,
                Mode::PowerOnReset | Mode::Booting(_) => {
                    if !self.advance_boot(now_us) {
                        return;
                    }
                }
                Mode::Operational | Mode::Degraded => {
                    self.run_partitions(now_us);
                    return;
                }
                Mode::Bite => {
                    if now_us >= self.mode_end_us {
                        let prior = self.prior_mode;
                        self.set_mode(prior, self.mode_end_us);
                    } else {
                        return;
                    }
                }
                Mode::DataLoad => {
                    if now_us >= self.mode_end_us {
                        if let Some(id) = self.pending_identity.take() {
                            self.identity = id;
                        }
                        let end = self.mode_end_us;
                        self.reset(ResetCause::DataLoad, end);
                    } else {
                        return;
                    }
                }
            }
        }
    }

    fn advance_boot(&mut self, now_us: u64) -> bool {
        let offsets = self.cfg.boot.stage_end_offsets();
        let por_end = self.boot_start_us + self.cfg.boot.por_us;
        if self.mode == Mode::PowerOnReset {
            if now_us < por_end {
                return false;
            }
            self.set_mode(Mode::Booting(BootStage::PowerGood), por_end);
        }
        while self.boot_stage_idx < offsets.len() {
            let (stage, off) = offsets[self.boot_stage_idx];
            let end = self.boot_start_us + off;
            self.mode = Mode::Booting(stage);
            if now_us < end {
                return false;
            }
            if stage == BootStage::Post && self.post_fault {
                self.nvm
                    .log_fault(end, FAULT_POST, Severity::Critical, "POST failure");
                self.enter_failed(end);
                return false;
            }
            if stage == BootStage::ConfigCheck {
                let found = self.cfg.expected.check(&self.identity);
                if !found.is_empty() {
                    self.nvm.log_fault(
                        end,
                        FAULT_CONFIG_MISMATCH,
                        Severity::Critical,
                        "configuration mismatch",
                    );
                    self.mismatches = found;
                    self.enter_failed(end);
                    return false;
                }
            }
            self.boot_stage_idx += 1;
        }
        let done = self.boot_start_us + self.cfg.boot.total_us();
        self.origin_us = done;
        self.cursor_us = done;
        self.last_kick_us = done;
        let next = if self.active_faults.iter().any(|(_, s)| *s >= Severity::Minor) {
            Mode::Degraded
        } else {
            Mode::Operational
        };
        self.set_mode(next, done);
        true
    }

    fn run_partitions(&mut self, now_us: u64) {
        let events = self
            .schedule
            .windows_between(self.origin_us, self.cursor_us, now_us);
        self.cursor_us = now_us + 1;
        for (t, wi) in events {
            if t.saturating_sub(self.last_kick_us) > self.cfg.watchdog_timeout_us {
                self.watchdog_reset(t);
                return;
            }
            let window = self.schedule.windows[wi].clone();
            let Some(si) = self.slot_index(&window.app) else {
                continue;
            };
            let period = self.slots[si].app.period_us();
            let due = self.slots[si]
                .last_run_us
                .is_none_or(|l| t.saturating_sub(l) >= period);
            if !due || t < self.slots[si].restart_until_us {
                continue;
            }
            let mut scratch = self.slots[si].io.clone();
            self.slots[si].app.run(t, &mut scratch);
            let exec = self.slots[si].app.exec_time_us();
            let overrun = exec > window.duration_us;
            if self.trace.len() >= TRACE_CAP {
                self.trace.remove(0);
            }
            self.trace.push(ExecRecord {
                time_us: t,
                slot: si,
                overrun,
            });
            self.slots[si].last_run_us = Some(t);
            if !overrun {
                self.slots[si].io = scratch;
                self.slots[si].live = true;
                self.slots[si].runs += 1;
                if !self.hung {
                    self.last_kick_us = t;
                }
                continue;
            }
            self.slots[si].overruns += 1;
            self.nvm.log_fault(t, FAULT_OVERRUN, Severity::Major, &window.app);
            match self.cfg.overrun_action {
                OverrunAction::LogOnly => {
                    self.slots[si].io = scratch;
                    self.slots[si].live = true;
                    if !self.hung {
                        self.last_kick_us = t;
                    }
                }
                OverrunAction::RestartPartition => {
                    let restart = self.cfg.partition_restart_us;
                    let slot = &mut self.slots[si];
                    slot.app.reset();
                    slot.live = false;
                    slot.last_run_us = None;
                    slot.restart_until_us = t + restart;
                    slot.io.invalidate_outputs();
                }
                OverrunAction::ResetBox => {
                    self.reset(ResetCause::Overrun, t);
                    return;
                }
            }
        }
        if now_us.saturating_sub(self.last_kick_us) > self.cfg.watchdog_timeout_us {
            self.watchdog_reset(now_us);
        }
    }

    fn watchdog_reset(&mut self, now_us: u64) {
        self.reset(ResetCause::Watchdog, now_us);
    }
}
