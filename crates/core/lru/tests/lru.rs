use std::cell::RefCell;
use std::rc::Rc;

use ::lru::*;

const MS: u64 = 1_000;
const S: u64 = 1_000_000;

struct Counter {
    name: String,
    period: u64,
    budget: u64,
    exec: u64,
    count: f64,
    log: Rc<RefCell<Vec<String>>>,
}

impl Counter {
    fn boxed(name: &str, period: u64, budget: u64, log: &Rc<RefCell<Vec<String>>>) -> Box<Counter> {
        Box::new(Counter {
            name: name.to_string(),
            period,
            budget,
            exec: budget,
            count: 0.0,
            log: log.clone(),
        })
    }
}

impl HostedApp for Counter {
    fn name(&self) -> &str {
        &self.name
    }
    fn period_us(&self) -> u64 {
        self.period
    }
    fn budget_us(&self) -> u64 {
        self.budget
    }
    fn exec_time_us(&self) -> u64 {
        self.exec
    }
    fn run(&mut self, now_us: u64, io: &mut AppIo) {
        self.count += 1.0;
        self.log.borrow_mut().push(self.name.clone());
        let gain = io.input_value("gain").unwrap_or(1.0);
        io.write("count", self.count * gain, now_us);
    }
    fn reset(&mut self) {
        self.count = 0.0;
    }
}

fn identity() -> Identity {
    Identity::new("PN-100", "SN-1", "SW-1", "CFG-1")
}

fn good() -> Vec<PowerInput> {
    vec![PowerInput::dc28(28.0)]
}

fn dead() -> Vec<PowerInput> {
    vec![PowerInput::dead(PowerKind::Dc28)]
}

fn schedule() -> Schedule {
    Schedule::new(100 * MS)
        .window("a", 0, 10 * MS)
        .window("b", 20 * MS, 10 * MS)
        .window("c", 50 * MS, 10 * MS)
}

fn make(cfg: LruConfig) -> (Lru, Rc<RefCell<Vec<String>>>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut lru = Lru::new(cfg);
    lru.add_app(Counter::boxed("c", 100 * MS, 10 * MS, &log));
    lru.add_app(Counter::boxed("a", 100 * MS, 10 * MS, &log));
    lru.add_app(Counter::boxed("b", 100 * MS, 10 * MS, &log));
    lru.set_schedule(schedule()).unwrap();
    (lru, log)
}

fn cfg() -> LruConfig {
    LruConfig::new(identity())
}

fn run(lru: &mut Lru, from: u64, to: u64, step: u64, inputs: &[PowerInput]) -> u64 {
    let mut t = from;
    while t <= to {
        lru.step(t, inputs, 25.0);
        t += step;
    }
    t - step
}

fn booted() -> (Lru, Rc<RefCell<Vec<String>>>, u64) {
    let (mut lru, log) = make(cfg());
    let end = run(&mut lru, 0, 4 * S, 10 * MS, &good());
    assert_eq!(lru.mode(), Mode::Operational);
    (lru, log, end)
}

#[test]
fn boot_timing_exact() {
    let (mut lru, _) = make(cfg());
    lru.step(0, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::PowerOnReset);
    lru.step(100 * MS, &good(), 25.0);
    assert_eq!(lru.boot_stage(), Some(BootStage::PowerGood));
    lru.step(160 * MS, &good(), 25.0);
    assert_eq!(lru.boot_stage(), Some(BootStage::Post));
    lru.step(2_200 * MS, &good(), 25.0);
    assert_eq!(lru.boot_stage(), Some(BootStage::ConfigCheck));
    lru.step(2_400 * MS, &good(), 25.0);
    assert_eq!(lru.boot_stage(), Some(BootStage::SoftwareCheck));
    lru.step(2_700 * MS, &good(), 25.0);
    assert_eq!(lru.boot_stage(), Some(BootStage::Init));
    lru.step(3_649_999, &good(), 25.0);
    assert!(!lru.is_running());
    lru.step(3_650_000, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::Operational);
}

#[test]
fn boot_timing_configurable() {
    let mut c = cfg();
    c.boot = BootTimings {
        por_us: 1,
        power_good_us: 1,
        post_us: 1,
        config_us: 1,
        software_us: 1,
        init_us: 5,
    };
    let (mut lru, _) = make(c);
    lru.step(0, &good(), 25.0);
    lru.step(9, &good(), 25.0);
    assert!(!lru.is_running());
    lru.step(10, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::Operational);
}

#[test]
fn coarse_step_boots_through_all_stages() {
    let (mut lru, _) = make(cfg());
    lru.step(0, &good(), 25.0);
    lru.step(10 * S, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::Operational);
    assert_eq!(lru.operational_since_us(), Some(3_650_000));
}

#[test]
fn brownout_within_holdup_rides_through() {
    let (mut lru, _, t) = booted();
    let hold = lru.power().full_holdup_us(20.0, 25.0);
    assert!(hold > 100 * MS);
    lru.step(t + 10 * MS, &dead(), 25.0);
    assert_eq!(lru.power().status(), PowerStatus::HoldUp);
    lru.step(t + 60 * MS, &dead(), 25.0);
    lru.step(t + 70 * MS, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::Operational);
    assert_eq!(lru.reset_count(), 0);
    assert_eq!(lru.power().stats().dips, 1);
}

#[test]
fn brownout_beyond_holdup_resets() {
    let (mut lru, _, t) = booted();
    let hold = lru.power().full_holdup_us(20.0, 25.0);
    lru.step(t + 10 * MS, &dead(), 25.0);
    lru.step(t + 10 * MS + hold + 1, &dead(), 25.0);
    assert_eq!(lru.mode(), Mode::Off);
    assert_eq!(lru.power().stats().resets, 1);
    lru.step(t + 10 * MS + hold + 100, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::PowerOnReset);
    assert!(lru.nvm().fault_count(FAULT_POWER_LOSS) >= 1);
}

#[test]
fn aged_capacitor_reboots_where_new_rides_through() {
    let gap = 100 * MS;
    let (mut new_box, _, t) = booted();
    new_box.step(t + 10 * MS, &dead(), 25.0);
    new_box.step(t + 10 * MS + gap, &dead(), 25.0);
    assert_eq!(new_box.power().status(), PowerStatus::HoldUp);

    let (mut old_box, _) = make(cfg());
    old_box.power_mut().set_age_equivalent_hours(200_000.0);
    old_box.power_mut().set_health(0.5);
    let t = run(&mut old_box, 0, 4 * S, 10 * MS, &good());
    old_box.step(t + 10 * MS, &dead(), 25.0);
    old_box.step(t + 10 * MS + gap, &dead(), 25.0);
    assert_eq!(old_box.power().status(), PowerStatus::Lost);
    assert_eq!(old_box.mode(), Mode::Off);
}

#[test]
fn holdup_shrinks_with_age_and_load() {
    let mut p = PowerStage::new(PowerConfig::default());
    let fresh = p.full_holdup_us(20.0, 25.0);
    p.set_age_equivalent_hours(50_000.0);
    let aged = p.full_holdup_us(20.0, 25.0);
    assert!(aged < fresh);
    assert!(p.full_holdup_us(40.0, 25.0) < aged);
    assert!(p.full_holdup_us(20.0, -30.0) < aged);
}

#[test]
fn capacitor_life_halves_per_ten_degrees() {
    let p = PowerStage::new(PowerConfig::default());
    let a = p.aging_acceleration(50.0);
    let b = p.aging_acceleration(60.0);
    assert!((b / a - 2.0).abs() < 1e-12);
    assert!((p.aging_acceleration(40.0) - 1.0).abs() < 1e-12);
}

#[test]
fn hot_box_ages_capacitor_faster() {
    let mut hot = PowerStage::new(PowerConfig::default());
    let mut cool = PowerStage::new(PowerConfig::default());
    let inputs = good();
    hot.step(0, &inputs, 70.0, 20.0);
    cool.step(0, &inputs, 40.0, 20.0);
    hot.step(3_600 * S, &inputs, 70.0, 20.0);
    cool.step(3_600 * S, &inputs, 40.0, 20.0);
    assert!((hot.age_equivalent_hours() / cool.age_equivalent_hours() - 8.0).abs() < 1e-6);
}

#[test]
fn second_gap_right_after_gap_has_less_holdup() {
    let (mut lru, _, t) = booted();
    lru.step(t + 10 * MS, &dead(), 25.0);
    lru.step(t + 100 * MS, &good(), 25.0);
    lru.step(t + 110 * MS, &dead(), 25.0);
    lru.step(t + 140 * MS, &dead(), 25.0);
    assert!(lru.power().charge() < 1.0);
}

#[test]
fn diode_or_two_inputs() {
    let (mut lru, _) = make(cfg());
    let inputs = vec![PowerInput::dc28(0.0), PowerInput::dc28(27.0)];
    run(&mut lru, 0, 4 * S, 10 * MS, &inputs);
    assert_eq!(lru.mode(), Mode::Operational);
    assert!((lru.power().bus_voltage_v() - 27.0).abs() < 1e-12);
    let both_low = vec![PowerInput::dc28(10.0), PowerInput::dc28(12.0)];
    lru.step(5 * S, &both_low, 25.0);
    assert_eq!(lru.power().status(), PowerStatus::HoldUp);
    assert_eq!(lru.power().stats().undervoltage_events, 1);
}

#[test]
fn ac_frequency_excursion_counts_as_gap() {
    let mut p = PowerStage::new(PowerConfig::default());
    p.step(0, &[PowerInput::ac115(115.0, 400.0)], 25.0, 20.0);
    p.step(S, &[PowerInput::ac115(115.0, 400.0)], 25.0, 20.0);
    p.step(2 * S, &[PowerInput::ac115(115.0, 300.0)], 25.0, 20.0);
    assert_eq!(p.status(), PowerStatus::HoldUp);
    assert_eq!(p.stats().frequency_excursions, 1);
    p.step(2 * S + 10 * MS, &[PowerInput::ac115(115.0, 400.0)], 25.0, 20.0);
    assert_eq!(p.status(), PowerStatus::Good);
    assert!(matches!(p.events()[0], PowerEvent::FrequencyExcursion { .. }));
}

#[test]
fn ac_and_dc_or() {
    let mut p = PowerStage::new(PowerConfig::default());
    let inputs = [PowerInput::ac115(115.0, 400.0), PowerInput::dc28(28.0)];
    p.step(0, &inputs, 25.0, 20.0);
    assert_eq!(p.status(), PowerStatus::Good);
    assert_eq!(
        p.input_state(&PowerInput::ac115(80.0, 400.0)),
        InputState::UnderVoltage
    );
}

#[test]
fn watchdog_resets_on_missed_kicks() {
    let (mut lru, _, t) = booted();
    lru.set_software_hang(true);
    let mut now = t;
    let mut reset_at = None;
    for _ in 0..100 {
        now += 10 * MS;
        lru.step(now, &good(), 25.0);
        if lru.reset_count() > 0 {
            reset_at = Some(now);
            break;
        }
    }
    let at = reset_at.unwrap();
    assert!(at - t >= lru.config().watchdog_timeout_us);
    assert_eq!(lru.last_reset_cause(), Some(ResetCause::Watchdog));
    assert_eq!(lru.mode(), Mode::PowerOnReset);
    assert!(lru.nvm().fault_count(FAULT_WATCHDOG) >= 1);
    let end = run(&mut lru, at, at + 5 * S, 10 * MS, &good());
    assert_eq!(lru.mode(), Mode::Operational);
    assert!(end > at);
}

#[test]
fn watchdog_does_not_fire_when_healthy() {
    let (mut lru, _, t) = booted();
    run(&mut lru, t, t + 10 * S, 10 * MS, &good());
    assert_eq!(lru.reset_count(), 0);
}

#[test]
fn partition_schedule_order() {
    let (mut lru, log, t) = booted();
    log.borrow_mut().clear();
    run(&mut lru, t + 10 * MS, t + 310 * MS, 10 * MS, &good());
    let l = log.borrow().clone();
    assert!(l.len() >= 6);
    let pattern = ["a", "b", "c"];
    for (i, n) in l.iter().enumerate() {
        assert_eq!(n, pattern[i % 3]);
    }
}

#[test]
fn partition_windows_at_exact_times() {
    let (mut lru, _, t) = booted();
    lru.clear_trace();
    run(&mut lru, t + 10 * MS, t + 210 * MS, 10 * MS, &good());
    let origin = lru.operational_since_us().unwrap();
    for rec in lru.trace() {
        assert_eq!((rec.time_us - origin) % (100 * MS) % (10 * MS), 0);
    }
    let times: Vec<u64> = lru.trace().iter().map(|r| r.time_us).collect();
    let mut sorted = times.clone();
    sorted.sort();
    assert_eq!(times, sorted);
}

#[test]
fn coarse_step_runs_all_windows_in_order() {
    let (mut lru, log, t) = booted();
    log.borrow_mut().clear();
    lru.step(t + 1_000 * MS, &good(), 25.0);
    assert_eq!(lru.reset_count(), 0);
    let l = log.borrow().clone();
    assert!(l.len() >= 27);
    assert_eq!(&l[0..3], &["a", "b", "c"]);
}

#[test]
fn schedule_validation() {
    assert_eq!(Schedule::new(0).validate(), Err(ScheduleError::ZeroFrame));
    assert_eq!(
        Schedule::new(10).window("a", 5, 10).validate(),
        Err(ScheduleError::WindowOutsideFrame)
    );
    assert_eq!(
        Schedule::new(100)
            .window("a", 0, 30)
            .window("b", 20, 10)
            .validate(),
        Err(ScheduleError::WindowsOverlap)
    );
    assert!(schedule().validate().is_ok());
}

#[test]
fn app_period_skips_windows() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut lru = Lru::new(cfg());
    lru.add_app(Counter::boxed("slow", 300 * MS, 10 * MS, &log));
    lru.set_schedule(Schedule::new(100 * MS).window("slow", 0, 10 * MS))
        .unwrap();
    let t = run(&mut lru, 0, 4 * S, 10 * MS, &good());
    log.borrow_mut().clear();
    run(&mut lru, t + 10 * MS, t + 1_000 * MS, 10 * MS, &good());
    let n = log.borrow().len();
    assert!((3..=4).contains(&n), "{n}");
}

#[test]
fn outputs_valid_when_running_and_invalid_after_reset() {
    let (mut lru, _, t) = booted();
    lru.set_input("a", "gain", 2.0, true, t);
    run(&mut lru, t + 10 * MS, t + 200 * MS, 10 * MS, &good());
    let s = lru.output("a", "count").unwrap();
    assert!(s.valid);
    assert!(s.value >= 2.0);
    assert!(s.timestamp_us > 0);
    lru.command_reset(t + 210 * MS);
    let frozen = lru.output("a", "count").unwrap();
    assert!(!frozen.valid);
    assert!((frozen.value - s.value).abs() < 1e-9 || frozen.value >= 2.0);
    let mid = run(&mut lru, t + 220 * MS, t + 2 * S, 10 * MS, &good());
    assert!(!lru.is_running());
    assert!(!lru.output("a", "count").unwrap().valid);
    run(&mut lru, mid + 10 * MS, mid + 3 * S, 10 * MS, &good());
    assert!(lru.is_running());
    assert!(lru.output("a", "count").unwrap().valid);
}

#[test]
fn outputs_invalid_after_power_loss() {
    let (mut lru, _, t) = booted();
    assert!(lru.output("b", "count").unwrap().valid);
    lru.step(t + 10 * MS, &dead(), 25.0);
    lru.step(t + 10 * S, &dead(), 25.0);
    assert_eq!(lru.mode(), Mode::Off);
    assert!(!lru.output("b", "count").unwrap().valid);
}

#[test]
fn input_validity_passes_to_app() {
    let (mut lru, _, t) = booted();
    lru.set_input("a", "gain", 5.0, false, t);
    run(&mut lru, t + 10 * MS, t + 200 * MS, 10 * MS, &good());
    let s = lru.output("a", "count").unwrap();
    assert!((s.value - s.value.floor()).abs() < 1e-9);
    let runs = lru.runs("a") as f64;
    assert!((s.value - runs).abs() < 1.5);
}

#[test]
fn overrun_detected_and_partition_restarted() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut lru = Lru::new(cfg());
    let mut app = Counter::boxed("a", 100 * MS, 10 * MS, &log);
    app.exec = 15 * MS;
    lru.add_app(app);
    lru.set_schedule(Schedule::new(100 * MS).window("a", 0, 10 * MS))
        .unwrap();
    run(&mut lru, 0, 5 * S, 10 * MS, &good());
    assert!(lru.overruns("a") >= 1);
    assert_eq!(lru.runs("a"), 0);
    assert!(lru.trace().iter().all(|r| r.overrun));
    assert!(lru.nvm().fault_count(FAULT_OVERRUN) >= 1);
    assert!(lru.output("a", "count").is_none());
}

#[test]
fn overrun_within_budget_is_fine() {
    let (mut lru, _, t) = booted();
    run(&mut lru, t, t + S, 10 * MS, &good());
    assert_eq!(lru.overruns("a"), 0);
    assert!(lru.trace().iter().all(|r| !r.overrun));
}

#[test]
fn overrun_can_reset_box() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut c = cfg();
    c.overrun_action = OverrunAction::ResetBox;
    let mut lru = Lru::new(c);
    let mut app = Counter::boxed("a", 100 * MS, 10 * MS, &log);
    app.exec = 11 * MS;
    lru.add_app(app);
    lru.set_schedule(Schedule::new(100 * MS).window("a", 0, 10 * MS))
        .unwrap();
    run(&mut lru, 0, 4 * S, 10 * MS, &good());
    assert!(lru.reset_count() >= 1);
    assert_eq!(lru.last_reset_cause(), Some(ResetCause::Overrun));
}

#[test]
fn overrun_log_only_keeps_outputs() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut c = cfg();
    c.overrun_action = OverrunAction::LogOnly;
    let mut lru = Lru::new(c);
    let mut app = Counter::boxed("a", 100 * MS, 10 * MS, &log);
    app.exec = 11 * MS;
    lru.add_app(app);
    lru.set_schedule(Schedule::new(100 * MS).window("a", 0, 10 * MS))
        .unwrap();
    run(&mut lru, 0, 5 * S, 10 * MS, &good());
    assert!(lru.overruns("a") >= 1);
    assert!(lru.output("a", "count").unwrap().valid);
}

#[test]
fn nvm_persists_across_power_cycles() {
    let (mut lru, _, t) = booted();
    lru.nvm_mut()
        .write("cfg", NvmValue::Text("x".to_string()))
        .unwrap();
    lru.step(t + 10 * MS, &dead(), 25.0);
    lru.step(t + 10 * S, &dead(), 25.0);
    assert_eq!(lru.mode(), Mode::Off);
    assert_eq!(lru.nvm().read("cfg"), Some(&NvmValue::Text("x".to_string())));
    run(&mut lru, t + 11 * S, t + 16 * S, 10 * MS, &good());
    assert_eq!(lru.mode(), Mode::Operational);
    assert_eq!(lru.nvm().read_u64("boot_count"), Some(2));
    assert!(lru.nvm().fault_count(FAULT_POWER_LOSS) >= 1);
    assert!(lru.nvm().faults()[0].time_us > 0);
}

#[test]
fn nvm_write_counting_and_wear() {
    let mut nvm = Nvm::new(3, 4);
    for i in 0..3 {
        nvm.write("k", NvmValue::U64(i)).unwrap();
    }
    assert_eq!(nvm.write_count("k"), 3);
    assert_eq!(nvm.write("k", NvmValue::U64(9)), Err(NvmError::WornOut));
    assert_eq!(nvm.read_u64("k"), Some(2));
    assert!((nvm.wear_fraction("k") - 1.0).abs() < 1e-12);
    nvm.write("other", NvmValue::F64(1.5)).unwrap();
    assert_eq!(nvm.total_writes(), 4);
    assert_eq!(nvm.keys(), vec!["k", "other"]);
}

#[test]
fn nvm_corruption_and_fault_log_capacity() {
    let mut nvm = Nvm::new(10, 2);
    nvm.write("a", NvmValue::I64(-4)).unwrap();
    nvm.log_fault(1, 10, Severity::Minor, "x");
    nvm.log_fault(2, 11, Severity::Major, "y");
    nvm.log_fault(3, 12, Severity::Info, "z");
    assert_eq!(nvm.faults().len(), 2);
    assert_eq!(nvm.faults()[0].code, 11);
    nvm.set_corrupt(true);
    assert!(nvm.read("a").is_none());
    assert_eq!(nvm.write("a", NvmValue::I64(1)), Err(NvmError::Corrupt));
    nvm.set_corrupt(false);
    assert_eq!(nvm.read("a"), Some(&NvmValue::I64(-4)));
    nvm.clear_faults();
    assert!(nvm.faults().is_empty());
}

#[test]
fn counter_overflow_248_days() {
    let mut c = PoweredCounter::new(31, 10 * MS);
    let day = 86_400 * S;
    let to_overflow = c.time_to_overflow_us();
    let days = to_overflow as f64 / day as f64;
    assert!((days - 248.55).abs() < 0.01);
    assert_eq!(c.advance(to_overflow - 1), 0);
    assert_eq!(c.value(), (1u64 << 31) - 1);
    assert_eq!(c.advance(10 * MS), 1);
    assert_eq!(c.value(), 0);
    assert_eq!(c.overflows(), 1);
}

#[test]
fn counter_width_configurable_and_remainder_kept() {
    let mut c = PoweredCounter::new(8, 1_000);
    c.advance(500);
    c.advance(500);
    assert_eq!(c.value(), 1);
    assert_eq!(c.advance(255_000), 1);
    assert_eq!(c.value(), 0);
    let mut d = PoweredCounter::new(64, 1);
    d.advance(u64::MAX);
    assert_eq!(d.overflows(), 0);
    d.set_ticks(5);
    d.reset();
    assert_eq!(d.value(), 0);
}

#[test]
fn counter_overflow_fails_box() {
    let mut c = cfg();
    c.counter_bits = 12;
    c.counter_tick_us = 10 * MS;
    c.counter_overflow = OverflowAction::FailBox;
    let (mut lru, _) = make(c);
    run(&mut lru, 0, 4 * S, 10 * MS, &good());
    assert!(lru.is_running());
    lru.step(42 * S, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::Failed);
    assert!(lru.is_no_go());
    assert!(lru.nvm().fault_count(FAULT_COUNTER_OVERFLOW) >= 1);
    assert!(!lru.output("a", "count").unwrap().valid);
}

#[test]
fn counter_overflow_resets_box_and_wraps() {
    let mut c = cfg();
    c.counter_bits = 12;
    c.counter_tick_us = 10 * MS;
    c.counter_overflow = OverflowAction::ResetBox;
    let (mut lru, _) = make(c);
    run(&mut lru, 0, 4 * S, 10 * MS, &good());
    run(&mut lru, 4 * S + 100 * MS, 45 * S, 100 * MS, &good());
    assert_eq!(lru.last_reset_cause(), Some(ResetCause::CounterOverflow));

    let mut w = cfg();
    w.counter_bits = 12;
    w.counter_tick_us = 10 * MS;
    let (mut lru, _) = make(w);
    run(&mut lru, 0, 45 * S, 100 * MS, &good());
    assert!(lru.is_running());
    assert_eq!(lru.counter().overflows(), 1);
}

#[test]
fn counter_resets_on_power_cycle_not_on_warm_reset() {
    let (mut lru, _, t) = booted();
    let v = lru.counter().value();
    assert!(v > 0);
    lru.command_reset(t + 10 * MS);
    lru.step(t + 20 * MS, &good(), 25.0);
    assert!(lru.counter().value() >= v);
    lru.step(t + 30 * MS, &dead(), 25.0);
    lru.step(t + 30 * S, &dead(), 25.0);
    assert_eq!(lru.counter().value(), 0);
    assert!(lru.nvm().read_u64("counter_ticks").unwrap() > 0);
}

#[test]
fn clock_drift_by_ppm_and_temperature() {
    let cc = ClockConfig {
        offset_ppm: 10.0,
        aging_ppm_per_year: 0.0,
        ..ClockConfig::default()
    };
    let mut clk = BoxClock::new(cc.clone());
    clk.power_up(0);
    clk.step(1_000 * S, 25.0);
    assert!((clk.error_us(1_000 * S) - 10_000.0).abs() < 1e-3);
    let mut hot = BoxClock::new(cc);
    hot.power_up(0);
    hot.step(1_000 * S, 85.0);
    assert!(hot.error_us(1_000 * S) < clk.error_us(1_000 * S));
    assert!((hot.last_rate_ppm() - (10.0 - 0.034 * 3600.0)).abs() < 1e-9);
}

#[test]
fn clock_aging_and_resync() {
    let cc = ClockConfig {
        aging_ppm_per_year: 3.0,
        ..ClockConfig::default()
    };
    let mut clk = BoxClock::new(cc);
    clk.set_age_hours(8766.0 * 5.0);
    assert!((clk.rate_error_ppm(25.0) - 15.0).abs() < 1e-9);
    clk.power_up(0);
    assert!(!clk.is_synced());
    clk.step(100 * S, 25.0);
    assert!(clk.error_us(100 * S) > 1_000.0);
    clk.resync(100 * S);
    assert!(clk.is_synced());
    assert_eq!(clk.local_time_us(), 100 * S);
    clk.set_stuck(true);
    clk.step(200 * S, 25.0);
    assert_eq!(clk.local_time_us(), 100 * S);
}

#[test]
fn box_clock_runs_and_resets_on_power_up() {
    let (mut lru, _, t) = booted();
    assert!(lru.clock().local_time_us() > 3 * S);
    lru.resync_clock(1_000_000_000);
    assert!(lru.clock().is_synced());
    lru.step(t + 10 * MS, &dead(), 25.0);
    lru.step(t + 10 * S, &dead(), 25.0);
    lru.step(t + 11 * S, &good(), 25.0);
    assert!(!lru.clock().is_synced());
    assert_eq!(lru.clock().local_time_us(), 0);
}

#[test]
fn identity_mismatch_is_no_go() {
    let mut c = cfg();
    c.expected.software_pn = "SW-2".to_string();
    let (mut lru, _) = make(c);
    run(&mut lru, 0, 5 * S, 10 * MS, &good());
    assert_eq!(lru.mode(), Mode::Failed);
    assert!(lru.is_no_go());
    assert_eq!(
        lru.mismatches(),
        &[IdentityMismatch::SoftwarePn {
            found: "SW-1".to_string(),
            expected: "SW-2".to_string()
        }]
    );
    assert!(lru.nvm().fault_count(FAULT_CONFIG_MISMATCH) >= 1);
    assert_eq!(lru.runs("a"), 0);
}

#[test]
fn data_load_fixes_mismatch() {
    let mut c = cfg();
    c.expected.software_pn = "SW-2".to_string();
    c.expected.config_pn = "CFG-9".to_string();
    let (mut lru, _) = make(c);
    run(&mut lru, 0, 5 * S, 10 * MS, &good());
    assert!(lru.is_no_go());
    assert!(lru.start_data_load(Identity::new("PN-100", "SN-1", "SW-2", "CFG-9"), 20 * S, 5 * S));
    assert_eq!(lru.mode(), Mode::DataLoad);
    run(&mut lru, 5 * S + 10 * MS, 24 * S, 10 * MS, &good());
    assert_eq!(lru.mode(), Mode::DataLoad);
    run(&mut lru, 24 * S + 10 * MS, 35 * S, 10 * MS, &good());
    assert_eq!(lru.mode(), Mode::Operational);
    assert!(!lru.is_no_go());
    assert_eq!(lru.identity().software_pn, "SW-2");
}

#[test]
fn wrong_part_number_detected() {
    let e = Expected {
        part_numbers: vec!["A".to_string(), "B".to_string()],
        software_pn: "S".to_string(),
        config_pn: "C".to_string(),
    };
    let id = Identity::new("Z", "1", "S", "C");
    assert_eq!(
        e.check(&id),
        vec![IdentityMismatch::PartNumber {
            found: "Z".to_string()
        }]
    );
    assert!(e.check(&Identity::new("B", "9", "S", "C")).is_empty());
}

#[test]
fn degraded_and_bite() {
    let (mut lru, _, t) = booted();
    lru.report_fault(100, Severity::Minor, "sensor", t);
    assert_eq!(lru.mode(), Mode::Degraded);
    assert!(lru.is_running());
    lru.clear_fault(100, t + MS);
    assert_eq!(lru.mode(), Mode::Operational);
    assert!(lru.start_bite(t + 10 * MS));
    assert_eq!(lru.mode(), Mode::Bite);
    assert!(!lru.output("a", "count").unwrap().valid);
    lru.step(t + 4 * S, &good(), 25.0);
    assert_eq!(lru.mode(), Mode::Operational);
    assert_eq!(lru.nvm().read_u64("fault_count_100"), Some(1));
}

#[test]
fn critical_fault_fails_box() {
    let (mut lru, _, t) = booted();
    lru.report_fault(200, Severity::Critical, "cpu", t);
    assert_eq!(lru.mode(), Mode::Failed);
    assert!(!lru.start_bite(t + MS));
    lru.command_reset(t + 10 * MS);
    assert_eq!(lru.mode(), Mode::PowerOnReset);
}

#[test]
fn post_failure_fails_box() {
    let (mut lru, _) = make(cfg());
    lru.set_post_fault(true);
    run(&mut lru, 0, 5 * S, 10 * MS, &good());
    assert_eq!(lru.mode(), Mode::Failed);
    assert!(lru.nvm().fault_count(FAULT_POST) >= 1);
}

#[test]
fn cold_soak_blocks_start() {
    let mut c = cfg();
    c.min_start_temp_c = -20.0;
    let (mut lru, _) = make(c);
    lru.step(0, &good(), -30.0);
    assert_eq!(lru.mode(), Mode::Off);
    lru.step(S, &good(), -10.0);
    assert_eq!(lru.mode(), Mode::PowerOnReset);
}

#[test]
fn deterministic_replay() {
    let go = || {
        let (mut lru, _) = make(cfg());
        run(&mut lru, 0, 4 * S, 10 * MS, &good());
        lru.step(4 * S + 10 * MS, &dead(), 25.0);
        lru.step(4 * S + 60 * MS, &good(), 25.0);
        run(&mut lru, 4 * S + 70 * MS, 6 * S, 10 * MS, &good());
        (
            lru.trace().to_vec(),
            lru.output("a", "count"),
            lru.clock().local_time_us(),
            lru.nvm().faults().to_vec(),
        )
    };
    assert_eq!(go(), go());
}
