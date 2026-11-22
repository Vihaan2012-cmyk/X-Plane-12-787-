#![allow(clippy::needless_range_loop, clippy::collapsible_if)]

use super::bpcu::*;
use super::cbic::*;
use super::load_mgmt::*;

struct Sim {
    pair: BpcuPair,
    inp: BpcuInput,
    out: BpcuOutput,
}

fn live(inp: &BpcuInput) -> [f64; BUS_COUNT] {
    let mut up = [false; NODE_COUNT];
    for b in 0..BUS_COUNT {
        if inp.gcb_aux_closed[b] && inp.gen_ok[b] {
            up[b] = true;
        }
    }
    for k in 0..2 {
        if inp.asg_cb_aux_closed[k] && inp.asg_ok[k] {
            up[APB_NODE[k]] = true;
        }
        if inp.epc_aux_closed[k] && inp.ext_plugged[k] && inp.ext_quality_ok[k] {
            up[EPC_BUS[k]] = true;
        }
    }
    for _ in 0..8 {
        for (e, &(x, y)) in BTB_NODES.iter().enumerate() {
            if inp.btb_aux_closed[e] && (up[x] || up[y]) {
                up[x] = true;
                up[y] = true;
            }
        }
    }
    [0, 1, 2, 3].map(|b| if up[b] { 235.0 } else { 0.0 })
}

fn overlap(i: &BpcuInput) -> bool {
    let mut comp = [0usize, 1, 2, 3, 4, 5];
    for _ in 0..8 {
        for (e, &(x, y)) in BTB_NODES.iter().enumerate() {
            if i.btb_aux_closed[e] {
                let m = comp[x].min(comp[y]);
                comp[x] = m;
                comp[y] = m;
            }
        }
    }
    let mut count = [0u8; NODE_COUNT];
    for b in 0..BUS_COUNT {
        if i.gcb_aux_closed[b] && i.gen_ok[b] {
            count[comp[b]] += 1;
        }
    }
    for k in 0..2 {
        if i.asg_cb_aux_closed[k] && i.asg_ok[k] {
            count[comp[APB_NODE[k]]] += 1;
        }
        if i.epc_aux_closed[k] && i.ext_plugged[k] && i.ext_quality_ok[k] {
            count[comp[EPC_BUS[k]]] += 1;
        }
    }
    count.iter().any(|&c| c > 1)
}

impl Sim {
    fn new() -> Self {
        let inp = BpcuInput::cold_and_dark(0);
        let mut pair = BpcuPair::new();
        let out = pair.step(&inp);
        Self { pair, inp, out }
    }

    fn tick(&mut self) {
        self.out = self.pair.step(&self.inp);
        loop_plant(&mut self.inp, &self.out, 10_000);
        self.inp.bus_volts = live(&self.inp);
        self.inp.atu_ok = [self.inp.bus_volts[0] >= 200.0, self.inp.bus_volts[2] >= 200.0];
        self.inp.dc_main_ok = self.inp.atu_ok;
        self.inp.backup_bus_ok = self.inp.bus_volts[3] >= 200.0 || self.inp.bus_volts[0] >= 200.0;
    }

    fn run_ms(&mut self, ms: u64) {
        for _ in 0..ms / 10 {
            self.tick();
            assert!(!overlap(&self.inp), "paralleled sources");
        }
    }

    fn all_powered(&self) -> bool {
        self.out.bus_powered.iter().all(|&p| p)
    }
}

fn flight_sim() -> Sim {
    let mut s = Sim::new();
    s.inp.on_ground = false;
    s.inp.gen_ok = [true; 4];
    s.inp.asg_ok = [true, true];
    s.run_ms(300);
    s
}

fn vf(k: u8) -> Feed {
    Feed::Src(Src::Vfsg(k))
}

#[test]
fn battery_only_power_up_limits_loads() {
    let mut s = Sim::new();
    s.inp.batt_switch_on = true;
    s.run_ms(100);
    assert_eq!(s.out.ground_config, GroundConfig::BatteryOnly);
    assert!(s.out.only_apu_start_loads);
    assert!(!s.out.sspc_elcc_loads_enabled);
    assert!(s.out.main_batt_relay_close);
    assert!(!s.out.apu_batt_relay_close);
    assert_eq!(s.out.capt_inst, InstFeed::MainBattery);
    s.inp.apu_start_requested = true;
    s.run_ms(100);
    assert!(s.out.apu_batt_relay_close);
}

#[test]
fn cold_dark_to_engines_full_sequence() {
    let mut s = Sim::new();
    s.inp.batt_switch_on = true;
    s.run_ms(100);
    s.inp.ext_plugged = [true, false, false];
    s.inp.ext_quality_ok = [true, false, false];
    s.run_ms(100);
    assert!(!s.out.ext_avail[0]);
    assert!(s.out.bus_feed.iter().all(|&f| f == Feed::None));
    s.run_ms(100);
    assert!(s.out.ext_avail[0]);
    assert!(!s.out.epc_close[0]);
    s.run_ms(200);
    assert!(s.out.epc_close[0]);
    assert_eq!(s.out.ground_config, GroundConfig::GroundHandling);
    s.run_ms(500);
    assert_eq!(s.out.bus_feed[0], Feed::Src(Src::Ext(0)));
    assert!(s.all_powered());
    assert!(!s.out.ext_full_power);
    assert!(s.out.sspc_elcc_loads_enabled);
    s.inp.ext_switch_on[0] = true;
    s.run_ms(100);
    assert_eq!(s.out.ground_config, GroundConfig::ExtPowerOn);
    assert!(s.out.ext_full_power);
    s.inp.asg_ok = [true, true];
    s.run_ms(500);
    assert_eq!(s.out.bus_feed[0], Feed::Src(Src::Asg(1)));
    assert_eq!(s.out.bus_feed[1], Feed::Src(Src::Asg(0)));
    assert_eq!(s.out.bus_feed[2], Feed::Src(Src::Asg(0)));
    assert_eq!(s.out.bus_feed[3], Feed::Src(Src::Asg(1)));
    assert!(!s.out.epc_close[0]);
    assert!(s.all_powered());
    s.inp.gen_ok = [true; 4];
    s.run_ms(500);
    assert_eq!(s.out.bus_feed, [vf(0), vf(1), vf(2), vf(3)]);
    assert!(s.all_powered());
    assert!(s.out.btb_close.iter().all(|&c| !c));
    assert!(s.out.asg_cb_close.iter().all(|&c| !c));
}

#[test]
fn external_power_quality_drop_then_apu() {
    let mut s = Sim::new();
    s.inp.ext_plugged = [true, false, false];
    s.inp.ext_quality_ok = [true, false, false];
    s.inp.ext_switch_on[0] = true;
    s.run_ms(2000);
    assert_eq!(s.out.bus_feed[0], Feed::Src(Src::Ext(0)));
    assert!(s.all_powered());
    s.inp.ext_quality_ok[0] = false;
    s.run_ms(20);
    assert!(!s.out.epc_close[0]);
    s.run_ms(100);
    assert!(s.out.bus_powered.iter().all(|&p| !p));
    s.inp.asg_ok = [true, true];
    s.run_ms(300);
    assert_eq!(s.out.bus_feed[0], Feed::Src(Src::Asg(1)));
    assert!(s.all_powered());
    s.inp.ext_quality_ok[0] = true;
    s.run_ms(200);
    assert!(s.out.ext_avail[0]);
    assert!(!s.out.epc_close[0]);
    assert_eq!(s.out.bus_feed[0], Feed::Src(Src::Asg(1)));
}

#[test]
fn power_transfer_parallels_instrument_buses_for_15_s() {
    let mut s = flight_sim();
    s.run_ms(130_000);
    assert!(!s.out.instr_parallel);
    s.inp.gen_ok[1] = false;
    s.run_ms(20);
    assert!(s.out.instr_parallel);
    s.run_ms(300);
    assert!(s.out.instr_parallel);
    s.run_ms(14_000);
    assert!(s.out.instr_parallel);
    s.run_ms(2_000);
    assert!(!s.out.instr_parallel);
}

#[test]
fn flight_loss_of_one_vfsg_uses_bus_tie() {
    let mut s = flight_sim();
    s.inp.asg_ok = [false, false];
    s.run_ms(100);
    assert_eq!(s.out.bus_feed, [vf(0), vf(1), vf(2), vf(3)]);
    s.inp.gen_ok[1] = false;
    s.run_ms(300);
    assert_eq!(s.out.bus_feed[1], vf(0));
    assert!(s.out.btb_close[4]);
    assert!(s.all_powered());
    s.inp.gen_ok[1] = true;
    s.run_ms(300);
    assert_eq!(s.out.bus_feed[1], vf(1));
    assert!(s.out.btb_close.iter().all(|&c| !c));
}

#[test]
fn flight_loss_of_one_vfsg_prefers_other_apu_on_l1() {
    let mut s = flight_sim();
    s.inp.gen_ok[0] = false;
    s.run_ms(300);
    assert_eq!(s.out.bus_feed[0], Feed::Src(Src::Asg(1)));
    assert!(s.out.btb_close[0]);
    assert!(s.all_powered());
}

#[test]
fn flight_loss_of_both_generators_one_side() {
    let mut s = flight_sim();
    s.inp.asg_ok = [false, false];
    s.inp.gen_ok = [false, false, true, true];
    s.run_ms(500);
    assert!(matches!(s.out.bus_feed[0], Feed::Src(Src::Vfsg(2 | 3))));
    assert!(matches!(s.out.bus_feed[1], Feed::Src(Src::Vfsg(2 | 3))));
    assert!(s.all_powered());
    assert!(s.out.source_kva <= 2.0 * VFSG_KVA);
}

#[test]
fn flight_ext_power_is_never_used() {
    let mut s = flight_sim();
    s.inp.gen_ok = [false; 4];
    s.inp.asg_ok = [false; 2];
    s.inp.ext_plugged = [true; 3];
    s.inp.ext_quality_ok = [true; 3];
    s.inp.ext_switch_on = [true; 3];
    s.run_ms(3000);
    assert!(!s.out.epc_close.iter().any(|&c| c));
    assert!(s.out.all_ac_lost);
}

#[test]
fn aft_external_power_never_feeds_a_main_bus() {
    let mut s = Sim::new();
    s.inp.ext_plugged = [false, false, true];
    s.inp.ext_quality_ok = [false, false, true];
    s.inp.ext_switch_on[2] = true;
    s.run_ms(1000);
    assert!(s.out.epc_close[2]);
    assert!(s.out.bus_feed.iter().all(|&f| f == Feed::None));
}

#[test]
fn rank_tables_match_documented_priorities() {
    assert_eq!(RANK_GROUND[0][1], Src::Asg(1));
    assert_eq!(RANK_AIR[0][3], Src::Vfsg(3));
    assert_eq!(RANK_GROUND[0][3], Src::Asg(0));
    assert_eq!(RANK_AIR[1][1], Src::Asg(0));
    assert_eq!(RANK_AIR[3][1], Src::Asg(1));
    assert_eq!(RANK_GROUND[2][1], Src::Asg(0));
    for b in 0..4 {
        assert_eq!(RANK_AIR[b][6], Src::Ext(0));
        assert_eq!(RANK_AIR[b][7], Src::Ext(1));
        assert_eq!(RANK_GROUND[b][0], Src::Vfsg(b as u8));
    }
}

#[test]
fn bus_115_ties_close_when_one_side_only() {
    let mut s = flight_sim();
    assert_eq!(s.out.bsb_close, [true, true]);
    assert_eq!(s.out.btb115_close, [false, false]);
    s.inp.atu_ok = [true, false];
    s.out = s.pair.step(&s.inp);
    assert_eq!(s.out.bsb_close, [true, false]);
    assert_eq!(s.out.btb115_close, [true, true]);
    s.inp.atu_ok = [false, false];
    s.out = s.pair.step(&s.inp);
    assert_eq!(s.out.btb115_close, [false, false]);
}

#[test]
fn rat_requested_after_15s_loss_of_l1_and_backup() {
    let mut s = flight_sim();
    s.inp.gen_ok = [false; 4];
    s.inp.asg_ok = [false; 2];
    s.run_ms(10_000);
    assert!(!s.out.rat_deploy);
    assert_eq!(s.out.capt_inst, InstFeed::MainBattery);
    s.run_ms(6_000);
    assert!(s.out.rat_deploy);
    s.inp.rat_gen_ok = true;
    s.run_ms(100);
    assert!(s.out.rat_cb_close && s.out.bb_iso_open && s.out.c1_tru_iso_open);
    assert_eq!(s.out.capt_inst, InstFeed::BackupBus);
    assert_eq!(s.out.fo_inst, InstFeed::BackupBus);
    s.inp.gen_ok = [true; 4];
    s.run_ms(500);
    assert!(!s.out.rat_deploy);
    assert!(!s.out.rat_cb_close);
}

#[test]
fn refuel_on_battery_expires_after_15_minutes() {
    let mut s = Sim::new();
    s.inp.batt_switch_on = true;
    s.inp.refuel_panel_active = true;
    s.run_ms(100);
    assert!(s.out.refuel_on_battery);
    s.inp.now_us += 14 * 60 * 1_000_000;
    s.run_ms(100);
    assert!(s.out.refuel_on_battery);
    s.inp.now_us += 2 * 60 * 1_000_000;
    s.run_ms(100);
    assert!(!s.out.refuel_on_battery);
}

#[test]
fn bpcu_peer_takes_over_when_one_unpowered() {
    let mut s = flight_sim();
    s.pair.left_powered = false;
    s.run_ms(50);
    assert_eq!(s.out.controlling, [false, true]);
    assert!(s.out.gcb_close.iter().all(|&c| c));
    s.pair.right_powered = false;
    s.run_ms(50);
    assert!(s.out.gcb_close.iter().all(|&c| !c));
}

fn lm_input(now: u64, kw: f64) -> LmInput {
    LmInput {
        now_us: now,
        on_ground: false,
        available_kw: kw,
        demand_on: [true; 14],
        engine_start: false,
        source_change: false,
        ext_in_use: false,
        ext_kva: 0.0,
        overload_hint_kw: 0.0,
        hpx_kw: [0.0; 2],
        hpx_limit_kw: [0.0; 2],
        pecs_coolant_c: 20.0,
        ext_phase_a: 0.0,
        ext_single_90: false,
    }
}

fn cac_relief_kw() -> f64 {
    DEFAULT_LOADS[CAC_LOAD_INDEX].kw * (1.0 - APPROX_CAC_MIN_SPEED)
}

fn index_of(name: &str) -> usize {
    DEFAULT_LOADS.iter().position(|d| d.name == name).unwrap()
}

#[test]
fn load_shed_order_lowest_priority_first_and_restore_reverse() {
    let mut lm = LoadManager::new();
    let total: f64 = DEFAULT_LOADS.iter().map(|d| d.kw).sum();
    let mut now = 0;
    let mut order = Vec::new();
    let mut out = lm.step(&lm_input(now, total));
    assert_eq!(out.shed_count, 0);
    let cap = total - cac_relief_kw() - 200.0;
    let mut prev = out.shed;
    for _ in 0..700 {
        now += 500_000;
        out = lm.step(&lm_input(now, cap));
        for k in 0..14 {
            if out.shed[k] && !prev[k] {
                order.push(k);
            }
        }
        prev = out.shed;
    }
    assert!(order.len() >= 2);
    assert!(out.total_kw <= cap);
    let pr: Vec<u8> = order.iter().map(|&k| LoadManager::priority(k, false)).collect();
    assert!(pr.windows(2).all(|w| w[0] >= w[1]));
    assert!(!out.shed[index_of("FLIGHT_DISPLAYS_ESSENTIAL")]);
    assert!(!out.shed[index_of("COMM_NAV")]);
    let mut restored = Vec::new();
    let mut prev = out.shed;
    for _ in 0..600 {
        now += 500_000;
        out = lm.step(&lm_input(now, total + 10.0));
        for k in 0..14 {
            if !out.shed[k] && prev[k] {
                restored.push(k);
            }
        }
        prev = out.shed;
    }
    assert_eq!(out.shed_count, 0);
    let rp: Vec<u8> = restored
        .iter()
        .map(|&k| LoadManager::priority(k, false))
        .collect();
    assert!(rp.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn shed_load_shows_override_with_breaker_closed() {
    let mut lm = LoadManager::new();
    let mut now = 0;
    let mut out = lm.step(&lm_input(now, 60.0));
    for _ in 0..40 {
        now += 500_000;
        out = lm.step(&lm_input(now, 60.0));
    }
    let g = index_of("GALLEY_FWD");
    assert!(out.shed[g] && out.override_status[g] && out.breaker_closed[g]);
}

#[test]
fn generator_overload_first_reduces_cac_speed() {
    let mut lm = LoadManager::new();
    let total: f64 = DEFAULT_LOADS.iter().map(|d| d.kw).sum();
    let out = lm.step(&lm_input(0, total - 50.0));
    assert!(out.cac_speed < 1.0 && out.cac_speed >= APPROX_CAC_MIN_SPEED);
    assert!(out.total_kw <= total - 50.0 + 1e-9);
    assert_eq!(out.shed_count, 0);
}

#[test]
fn restore_needs_delay_and_hysteresis() {
    let mut lm = LoadManager::new();
    let total: f64 = DEFAULT_LOADS.iter().map(|d| d.kw).sum();
    let mut now = 0;
    let short = (total - cac_relief_kw()) / 1.1;
    lm.step(&lm_input(now, short));
    now += 500_000;
    let early = lm.step(&lm_input(now, short));
    assert_eq!(early.shed_count, 0);
    let mut out = early;
    while out.shed_count == 0 && now < 400_000_000 {
        now += 500_000;
        out = lm.step(&lm_input(now, short));
    }
    assert!(now >= 300_000_000);
    let shed_before = out.shed_count;
    assert!(shed_before > 0);
    now += 1_000_000;
    let out = lm.step(&lm_input(now, total + 10.0));
    assert_eq!(out.shed_count, shed_before);
    now += 20_000_000;
    let out = lm.step(&lm_input(now, total + 10.0));
    assert!(out.shed_count < shed_before);
}

#[test]
fn engine_start_always_sheds_and_holds() {
    let mut lm = LoadManager::new();
    let mut i = lm_input(0, 10_000.0);
    i.on_ground = true;
    i.engine_start = true;
    let out = lm.step(&i);
    assert!(out.shed[index_of("IFE")] && out.shed[index_of("ECS_PACKS")]);
    assert!(!out.shed[index_of("COMM_NAV")]);
    i.engine_start = false;
    i.now_us = 30_000_000;
    let out = lm.step(&i);
    assert!(out.shed[index_of("IFE")]);
    let held = out.shed_count;
    i.now_us = 200_000_000;
    lm.step(&i);
    i.now_us = 211_000_000;
    let out = lm.step(&i);
    assert!(out.shed_count < held);
}

#[test]
fn external_power_overload_sheds_extra_load() {
    let mut lm = LoadManager::new();
    let mut i = lm_input(0, 10_000.0);
    i.on_ground = true;
    i.ext_in_use = true;
    i.ext_kva = 120.0;
    let first = lm.step(&i);
    assert!(first.ext_overload_5s);
    assert_eq!(first.shed_count, 1);
    i.now_us = 6_000_000;
    let out = lm.step(&i);
    assert!(out.ext_overload_5s);
    assert_eq!(out.shed_count, 2);
}

#[test]
fn bpcu_power_up_reset_sheds_non_essential() {
    let mut lm = LoadManager::new();
    lm.power_up_reset(0);
    let out = lm.step(&lm_input(0, 10_000.0));
    assert!(out.shed_count >= 13);
    assert!(!out.shed[index_of("FLIGHT_DISPLAYS_ESSENTIAL")]);
}

fn cbic_input(now_us: u64, on_ground: bool, doors: bool, gs: f64, gnd_test: bool) -> CbicInput {
    CbicInput {
        now_us,
        on_ground,
        ground_speed_kt: gs,
        doors_open: doors,
        ext_power_avail: false,
        gnd_test_enable: gnd_test,
    }
}

fn cbic_ready(on_ground: bool, doors: bool, gs: f64, gnd_test: bool) -> Cbic {
    let mut c = Cbic::new();
    c.add_breaker(1, "FUEL PUMP L", BreakerKind::Ecb, CbState::Closed);
    c.add_breaker(2, "IFE", BreakerKind::Ecb, CbState::Open);
    c.add_breaker(3, "P49 TCB", BreakerKind::Tcb, CbState::Closed);
    c.add_breaker(4, "FC SSR", BreakerKind::FcSsr, CbState::Closed);
    c.add_breaker(5, "ELCC X", BreakerKind::Elcc, CbState::Tripped);
    c.add_breaker(6, "GALLEY", BreakerKind::Ecb, CbState::Tripped);
    c.update(&cbic_input(0, on_ground, doors, gs, gnd_test));
    c
}

#[test]
fn cbic_modes() {
    assert_eq!(cbic_ready(true, true, 0.0, false).mode(), Mode::Ground);
    assert_eq!(cbic_ready(true, true, 0.0, true).mode(), Mode::Maintenance);
    assert_eq!(cbic_ready(true, true, 3.0, true).mode(), Mode::Limited);
    assert_eq!(cbic_ready(false, true, 0.0, true).mode(), Mode::Limited);
}

#[test]
fn cbic_ground_open_needs_ctrl_and_flight_needs_enter() {
    let mut c = cbic_ready(true, true, 0.0, false);
    let u = c.connect(UserKind::Mfd).unwrap();
    assert_eq!(
        c.command(u, 1, CbAction::Open, false),
        CmdOutcome::Refused(Refusal::CtrlRequired)
    );
    assert_eq!(c.command(u, 1, CbAction::Open, true), CmdOutcome::Executed);
    assert_eq!(
        c.take_commands(),
        vec![BreakerCommand {
            id: 1,
            action: CbAction::Open
        }]
    );
    let mut f = cbic_ready(false, false, 250.0, false);
    let u = f.connect(UserKind::Mfd).unwrap();
    assert_eq!(f.command(u, 1, CbAction::Open, false), CmdOutcome::PendingEnter);
    assert!(f.take_commands().is_empty());
    assert_eq!(f.press_enter(u), CmdOutcome::Executed);
    assert_eq!(f.take_commands().len(), 1);
    assert_eq!(f.press_enter(u), CmdOutcome::Refused(Refusal::NoPendingCommand));
}

#[test]
fn cbic_flight_confirmation_expires() {
    let mut f = cbic_ready(false, false, 250.0, false);
    let u = f.connect(UserKind::Mfd).unwrap();
    assert_eq!(f.command(u, 1, CbAction::Open, false), CmdOutcome::PendingEnter);
    f.update(&cbic_input(11_000_000, false, false, 250.0, false));
    assert_eq!(f.press_enter(u), CmdOutcome::Refused(Refusal::NoPendingCommand));
}

#[test]
fn cbic_limited_mode_refuses_locked_actions() {
    let mut g = cbic_ready(true, true, 0.0, false);
    let u = g.connect(UserKind::Mfd).unwrap();
    assert_eq!(
        g.command(u, 2, CbAction::Lock(LockTag::DoNotClose), true),
        CmdOutcome::Executed
    );
    assert_eq!(g.breaker(2).unwrap().state, CbState::Locked(LockTag::DoNotClose));
    assert_eq!(
        g.command(u, 2, CbAction::Close, true),
        CmdOutcome::Refused(Refusal::LockedUnlockFirst)
    );
    let nvm = g.export_nvm(7);
    let mut f = cbic_ready(false, false, 250.0, false);
    f.restore_nvm(&nvm, 7);
    let u = f.connect(UserKind::Mfd).unwrap();
    assert_eq!(
        f.command(u, 2, CbAction::Unlock, false),
        CmdOutcome::Refused(Refusal::LimitedModeNoLockControl)
    );
    assert_eq!(
        f.command(u, 2, CbAction::Close, false),
        CmdOutcome::Refused(Refusal::LockedUnlockFirst)
    );
    assert_eq!(
        f.command(u, 1, CbAction::Lock(LockTag::Inop), false),
        CmdOutcome::Refused(Refusal::LimitedModeNoLockControl)
    );
    assert_eq!(
        f.command(u, 5, CbAction::Open, false),
        CmdOutcome::Refused(Refusal::LimitedModeResetOnly)
    );
    assert_eq!(
        f.command(u, 5, CbAction::RequestReset, false),
        CmdOutcome::PendingEnter
    );
}

#[test]
fn cbic_fc_ssr_blocked_in_flight_and_tcb_indication_only() {
    let mut f = cbic_ready(false, false, 250.0, false);
    let u = f.connect(UserKind::Mfd).unwrap();
    assert_eq!(
        f.command(u, 4, CbAction::Open, false),
        CmdOutcome::Refused(Refusal::FlightControlBlockedInFlight)
    );
    assert_eq!(
        f.command(u, 3, CbAction::Open, false),
        CmdOutcome::Refused(Refusal::IndicationOnly)
    );
    let mut g = cbic_ready(true, true, 0.0, false);
    let u = g.connect(UserKind::Mfd).unwrap();
    assert_eq!(
        g.command(u, 4, CbAction::Lock(LockTag::Inop), true),
        CmdOutcome::Refused(Refusal::NoLockOnFcSsr)
    );
    assert_eq!(g.command(u, 4, CbAction::Open, true), CmdOutcome::Executed);
}

#[test]
fn cbic_lock_rules() {
    let mut g = cbic_ready(true, true, 0.0, false);
    let u = g.connect(UserKind::Mfd).unwrap();
    assert_eq!(
        g.command(u, 1, CbAction::Lock(LockTag::Inop), true),
        CmdOutcome::Refused(Refusal::MustBeOpenToLock)
    );
    assert_eq!(
        g.command(u, 5, CbAction::Lock(LockTag::Inop), true),
        CmdOutcome::Refused(Refusal::InopLockEcbOnly)
    );
    assert_eq!(
        g.command(u, 6, CbAction::Close, true),
        CmdOutcome::Refused(Refusal::TrippedResetFirst)
    );
    assert_eq!(
        g.command(u, 6, CbAction::RequestReset, true),
        CmdOutcome::Executed
    );
    assert_eq!(
        g.command(u, 2, CbAction::Lock(LockTag::Inop), true),
        CmdOutcome::Executed
    );
    assert_eq!(g.command(u, 2, CbAction::Unlock, true), CmdOutcome::Executed);
    assert_eq!(g.breaker(2).unwrap().state, CbState::Open);
}

#[test]
fn cbic_laptop_control_ground_only() {
    let mut f = cbic_ready(false, false, 250.0, false);
    let l = f.connect(UserKind::Laptop).unwrap();
    assert_eq!(
        f.command(l, 1, CbAction::Open, false),
        CmdOutcome::Refused(Refusal::LaptopControlOnGroundOnly)
    );
}

#[test]
fn cbic_fifth_user_refused() {
    let mut c = cbic_ready(true, true, 0.0, false);
    let a = c.connect(UserKind::Mfd).unwrap();
    c.connect(UserKind::Mfd).unwrap();
    c.connect(UserKind::Mfd).unwrap();
    assert_eq!(c.connect(UserKind::Mfd), Err(Refusal::MaxUsers));
    c.connect(UserKind::Laptop).unwrap();
    assert_eq!(c.user_count(), 4);
    assert_eq!(c.connect(UserKind::Laptop), Err(Refusal::MaxUsers));
    assert_eq!(c.connect(UserKind::Mfd), Err(Refusal::MaxUsers));
    c.disconnect(a);
    assert!(c.connect(UserKind::Mfd).is_ok());
}

#[test]
fn cbic_pages_and_state_ordering() {
    let mut c = cbic_ready(true, true, 0.0, true);
    c.add_breaker(7, "X", BreakerKind::Ecb, CbState::Unknown);
    c.add_breaker(8, "Y", BreakerKind::Ecb, CbState::Locked(LockTag::Inop));
    assert_eq!(c.open_trip_page(), vec![2, 5, 6, 8]);
    assert_eq!(c.by_state(), vec![5, 6, 2, 8, 7]);
    assert_eq!(c.search("NOPE"), Err("CB WAS NOT FOUND"));
    assert_eq!(c.search("IFE"), Ok(vec![2]));
    assert!(c.set_custom_list(&[1, 2, 99]).is_ok());
    assert_eq!(c.custom_list(), &[1, 2]);
    let f = cbic_ready(false, false, 0.0, false);
    let mut f2 = f.clone();
    assert_eq!(f2.set_custom_list(&[1]), Err(Refusal::CustomListMaintenanceOnly));
    let ids: Vec<u32> = (0..40).collect();
    assert_eq!(f.status_pages(&ids).len(), 3);
}

#[test]
fn cbic_recent_used_capped_and_override_status() {
    let mut c = Cbic::new();
    for id in 0..40 {
        c.add_breaker(id, "B", BreakerKind::Ecb, CbState::Closed);
    }
    c.update(&cbic_input(0, true, true, 0.0, false));
    let u = c.connect(UserKind::Mfd).unwrap();
    for id in 0..40 {
        c.command(u, id, CbAction::Open, true);
    }
    assert_eq!(c.recent_used().len(), MAX_RECENT);
    assert_eq!(c.recent_used()[0], 39);
    c.report_status(5, CbState::Closed, 1.5);
    c.report_override(5, true);
    assert_eq!(c.output_status(5), Some("OVERRIDE"));
    assert_eq!(c.breaker(5).unwrap().state, CbState::Closed);
    c.report_comm_loss(5);
    assert_eq!(c.breaker(5).unwrap().state, CbState::Unknown);
}

#[test]
fn cbic_locks_persist_across_power_down() {
    let mut c = cbic_ready(true, true, 0.0, false);
    let u = c.connect(UserKind::Mfd).unwrap();
    c.command(u, 2, CbAction::Lock(LockTag::Inop), true);
    let mut cl = CbicCluster::new(c, 11);
    cl.set_host_power(0, false);
    assert!(cl.is_powered());
    assert_eq!(cl.active_host(), Some(1));
    assert_eq!(cl.app.breaker(2).unwrap().state, CbState::Locked(LockTag::Inop));
    cl.set_host_power(1, false);
    cl.set_host_power(2, false);
    assert!(!cl.is_powered());
    assert_eq!(cl.app.breaker(2).unwrap().state, CbState::Unknown);
    cl.set_host_power(2, true);
    assert_eq!(cl.app.breaker(2).unwrap().state, CbState::Locked(LockTag::Inop));
    assert_eq!(cl.app.breaker(1).unwrap().state, CbState::Unknown);
    assert_eq!(cl.app.user_count(), 0);
}

#[test]
fn cbic_nvm_discarded_on_airplane_sequence_mismatch() {
    let mut c = cbic_ready(true, true, 0.0, false);
    let u = c.connect(UserKind::Mfd).unwrap();
    c.command(u, 2, CbAction::Lock(LockTag::DoNotClose), true);
    let mut cl = CbicCluster::new(c, 11);
    for h in 0..3 {
        cl.set_host_power(h, false);
    }
    cl.set_airplane_seq(12);
    cl.set_host_power(0, true);
    assert!(cl.app.nvm_discarded);
    assert_eq!(cl.app.breaker(2).unwrap().state, CbState::Unknown);
}

#[test]
fn hpx_exceedance_sheds_a_group_and_repeat_within_20_min_adds_a_group() {
    let mut lm = LoadManager::new();
    let mut i = lm_input(0, 0.0);
    i.available_kw = 1000.0;
    i.demand_on = [true; 14];
    i.hpx_limit_kw = [300.0; 2];
    let mut t = 0u64;
    let mut step = |lm: &mut LoadManager, i: &mut LmInput, hpx: f64, secs: u64| {
        let mut o = None;
        for _ in 0..secs * 10 {
            t += 100_000;
            i.now_us = t;
            i.hpx_kw = [hpx, 100.0];
            o = Some(lm.step(i));
        }
        o.unwrap()
    };
    let o = step(&mut lm, &mut i, 200.0, 30);
    assert_eq!(o.hpx_groups_shed, 0);
    let o = step(&mut lm, &mut i, 350.0, 1);
    assert_eq!(o.hpx_groups_shed, 1);
    let cabin: Vec<usize> = (0..14)
        .filter(|k| DEFAULT_LOADS[*k].group == LoadGroup::Cabin)
        .collect();
    assert!(cabin.iter().all(|k| o.shed[*k] || !DEFAULT_LOADS[*k].sheddable));
    let o = step(&mut lm, &mut i, 200.0, 61);
    assert_eq!(o.hpx_groups_shed, 0);
    let o = step(&mut lm, &mut i, 350.0, 1);
    assert_eq!(
        o.hpx_groups_shed, 2,
        "second exceedance within 20 min adds a group"
    );
    let o = step(&mut lm, &mut i, 200.0, 61 + 20 * 60);
    assert_eq!(o.hpx_groups_shed, 0);
    let o = step(&mut lm, &mut i, 350.0, 1);
    assert_eq!(o.hpx_groups_shed, 1, "after 20 min the sequence starts again");
}

#[test]
fn pecs_coolant_above_64c_sheds_cabin_lighting_then_ife_and_restores_below() {
    let mut lm = LoadManager::new();
    let mut i = lm_input(0, 0.0);
    i.available_kw = 1000.0;
    i.demand_on = [true; 14];
    let mut t = 0u64;
    let mut run = |lm: &mut LoadManager, i: &mut LmInput, c: f64, secs: u64| {
        let mut o = None;
        for _ in 0..secs * 10 {
            t += 100_000;
            i.now_us = t;
            i.pecs_coolant_c = c;
            o = Some(lm.step(i));
        }
        o.unwrap()
    };
    let o = run(&mut lm, &mut i, 50.0, 30);
    assert_eq!(o.pecs_thermal_shed, 0);
    let o = run(&mut lm, &mut i, 66.0, 1);
    assert_eq!(o.pecs_thermal_shed, 1);
    assert!(o.shed[3], "cabin lighting first");
    let o = run(&mut lm, &mut i, 66.0, 11);
    assert_eq!(o.pecs_thermal_shed, 2);
    assert!(o.shed[2], "then IFE");
    let o = run(&mut lm, &mut i, 62.0, 30);
    assert_eq!(o.pecs_thermal_shed, 2, "no restore inside the hysteresis band");
    let o = run(&mut lm, &mut i, 55.0, 25);
    assert_eq!(o.pecs_thermal_shed, 0);
}

#[test]
fn ep_current_stops_emp_cmscs_with_thresholds_by_ep_configuration() {
    for (single, slow, fast) in [
        (false, EP_CMSC_STOP_A, EP_CMSC_STOP_FAST_A),
        (true, EP_90_CMSC_STOP_A, EP_90_CMSC_STOP_FAST_A),
    ] {
        let mut lm = LoadManager::new();
        let mut i = lm_input(0, 0.0);
        i.available_kw = 1000.0;
        i.demand_on = [true; 14];
        i.ext_in_use = true;
        i.ext_single_90 = single;
        let mut t = 0u64;
        let mut run = |lm: &mut LoadManager, i: &mut LmInput, a: f64, secs: f64| {
            let mut o = None;
            for _ in 0..(secs * 10.0) as u64 {
                t += 100_000;
                i.now_us = t;
                i.ext_phase_a = a;
                o = Some(lm.step(i));
            }
            o.unwrap()
        };
        assert!(!run(&mut lm, &mut i, fast + 5.0, 4.9).emp_cmsc_stopped);
        assert!(run(&mut lm, &mut i, fast + 5.0, 0.2).emp_cmsc_stopped);
        let mut lm = LoadManager::new();
        assert!(!run(&mut lm, &mut i, slow + 5.0, 239.0).emp_cmsc_stopped);
        let o = run(&mut lm, &mut i, slow + 5.0, 1.5);
        assert!(o.emp_cmsc_stopped && o.shed[HYD_LOAD_INDEX]);
    }
}
