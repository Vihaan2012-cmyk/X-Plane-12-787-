#![allow(clippy::needless_range_loop, clippy::collapsible_if)]

pub const BUS_COUNT: usize = 4;
pub const NODE_COUNT: usize = 6;
pub const SRC_COUNT: usize = 8;
pub const BTB_COUNT: usize = 6;
pub const BUS_NAMES: [&str; BUS_COUNT] = ["L1", "L2", "R1", "R2"];
pub const NODE_NAMES: [&str; NODE_COUNT] = ["L1", "L2", "R1", "R2", "LTB", "RTB"];
pub const BTB_NAMES: [&str; BTB_COUNT] = ["L1 BTB", "L2 BTB", "R1 BTB", "R2 BTB", "L3 BTB", "R3 BTB"];
pub const BTB_NODES: [(usize, usize); BTB_COUNT] = [(0, 5), (1, 4), (2, 4), (3, 5), (0, 1), (2, 3)];
pub const APB_NODE: [usize; 2] = [4, 5];
pub const EPC_BUS: [usize; 2] = [0, 2];
pub const EXT_QUALITY_US: u64 = 150_000;
pub const EXT_CONTACTOR_HOLD_US: u64 = 300_000;
pub const INSTR_PARALLEL_HOLD_US: u64 = 15_000_000;
pub const MBR_OPEN_DELAY_US: u64 = 15_000_000;
pub const MBR_SHUTDOWN_HOLD_US: u64 = 20_000_000;
pub const CCS_HOLD_US: u64 = 11_000_000;
pub const CIT_GROUND_TO_AIR_US: u64 = 120_000_000;
pub const BUS_LIVE_VOLTS: f64 = 200.0;
pub const VFSG_KVA: f64 = 250.0;
pub const ASG_KVA: f64 = 225.0;
pub const APU_CONTINUOUS_KVA_SEA_LEVEL: f64 = 450.0;
pub const APU_CONTINUOUS_KVA_CEILING: f64 = 100.0;
pub const APU_CEILING_FT: f64 = 43_100.0;

pub fn apu_continuous_kva(alt_ft: f64) -> f64 {
    let f = (alt_ft / APU_CEILING_FT).clamp(0.0, 1.0);
    APU_CONTINUOUS_KVA_SEA_LEVEL + (APU_CONTINUOUS_KVA_CEILING - APU_CONTINUOUS_KVA_SEA_LEVEL) * f
}
pub const EXT_KVA: f64 = 90.0;
pub const REFUEL_BATTERY_LIMIT_US: u64 = 15 * 60 * 1_000_000;
pub const RAT_REQUEST_DELAY_US: u64 = 15_000_000;
pub const FAIL_TO_CLOSE_BTB_US: u64 = 150_000;
pub const GROUND_BATTERY_ONLY_OFF_US: u64 = 30_000_000;
pub const BATTERY_OFF_MAX_US: u64 = 120_000_000;
pub const STANDBY_PARALLEL_INSTR_V: f64 = 24.0;
pub const STANDBY_PARALLEL_TRU_A: f64 = 360.0;
pub const STANDBY_PARALLEL_US: u64 = 50_000;
pub const FAIL_TO_CLOSE_BSB_US: u64 = 190_000;
pub const GROUND_HANDLING_MAX_HZ: f64 = 440.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Src {
    Vfsg(u8),
    Asg(u8),
    Ext(u8),
}

const V0: Src = Src::Vfsg(0);
const V1: Src = Src::Vfsg(1);
const V2: Src = Src::Vfsg(2);
const V3: Src = Src::Vfsg(3);
const AL: Src = Src::Asg(0);
const AR: Src = Src::Asg(1);
const EL: Src = Src::Ext(0);
const ER: Src = Src::Ext(1);

pub const RANK_GROUND: [[Src; SRC_COUNT]; BUS_COUNT] = [
    [V0, AR, V1, AL, V3, V2, EL, ER],
    [V1, AL, V0, V2, V3, AR, EL, ER],
    [V2, AL, V3, AR, V1, V0, ER, EL],
    [V3, AR, V2, V0, V1, AL, ER, EL],
];

pub const RANK_AIR: [[Src; SRC_COUNT]; BUS_COUNT] = [
    [V0, AR, V1, V3, AL, V2, EL, ER],
    [V1, AL, V0, V2, V3, AR, EL, ER],
    [V2, AL, V3, V1, AR, V0, EL, ER],
    [V3, AR, V2, V0, V1, AL, EL, ER],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feed {
    None,
    Src(Src),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BpcuInput {
    pub now_us: u64,
    pub on_ground: bool,
    pub gen_ok: [bool; BUS_COUNT],
    pub gen_ctrl_on: [bool; BUS_COUNT],
    pub gcb_aux_closed: [bool; BUS_COUNT],
    pub asg_ok: [bool; 2],
    pub asg_ctrl_on: [bool; 2],
    pub asg_cb_aux_closed: [bool; 2],
    pub ext_plugged: [bool; 3],
    pub ext_quality_ok: [bool; 3],
    pub ext_switch_on: [bool; 3],
    pub ext_hz: [f64; 3],
    pub alt_ft: f64,
    pub epc_aux_closed: [bool; 3],
    pub btb_aux_closed: [bool; BTB_COUNT],
    pub bus_volts: [f64; BUS_COUNT],
    pub atu_ok: [bool; 2],
    pub tie_fault: [bool; 2],
    pub instr_bus_v: [f64; 2],
    pub instr_tru_a: [f64; 2],
    pub bsb_aux_closed: [bool; 2],
    pub btb115_aux_closed: [bool; 2],
    pub dc_main_ok: [bool; 2],
    pub batt_switch_on: bool,
    pub apu_batt_switch_on: bool,
    pub apu_start_requested: bool,
    pub refuel_panel_active: bool,
    pub rat_gen_ok: bool,
    pub backup_bus_ok: bool,
    pub ground_service_switch: bool,
    pub engines_running: u8,
}

impl BpcuInput {
    pub fn cold_and_dark(now_us: u64) -> Self {
        Self {
            now_us,
            on_ground: true,
            gen_ok: [false; BUS_COUNT],
            gen_ctrl_on: [true; BUS_COUNT],
            gcb_aux_closed: [false; BUS_COUNT],
            asg_ok: [false; 2],
            asg_ctrl_on: [true; 2],
            asg_cb_aux_closed: [false; 2],
            ext_plugged: [false; 3],
            ext_quality_ok: [false; 3],
            ext_switch_on: [false; 3],
            ext_hz: [0.0; 3],
            alt_ft: 0.0,
            epc_aux_closed: [false; 3],
            btb_aux_closed: [false; BTB_COUNT],
            bus_volts: [0.0; BUS_COUNT],
            atu_ok: [false; 2],
            tie_fault: [false; 2],
            instr_bus_v: [28.0; 2],
            instr_tru_a: [0.0; 2],
            bsb_aux_closed: [false; 2],
            btb115_aux_closed: [false; 2],
            dc_main_ok: [false; 2],
            batt_switch_on: false,
            apu_batt_switch_on: false,
            apu_start_requested: false,
            refuel_panel_active: false,
            rat_gen_ok: false,
            backup_bus_ok: false,
            ground_service_switch: false,
            engines_running: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstFeed {
    None,
    MainDc,
    MainBattery,
    BackupBus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundConfig {
    FlightOrNormal,
    BatteryOnly,
    GroundHandling,
    ExtPowerOn,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BpcuOutput {
    pub gcb_close: [bool; BUS_COUNT],
    pub asg_cb_close: [bool; 2],
    pub epc_close: [bool; 3],
    pub btb_close: [bool; BTB_COUNT],
    pub bsb_close: [bool; 2],
    pub btb115_close: [bool; 2],
    pub bus_feed: [Feed; BUS_COUNT],
    pub bus_powered: [bool; BUS_COUNT],
    pub ext_avail: [bool; 3],
    pub ext_on: [bool; 3],
    pub source_kva: f64,
    pub source_change: bool,
    pub transfer_pending: bool,
    pub instr_parallel: bool,
    pub main_batt_relay_close: bool,
    pub apu_batt_relay_close: bool,
    pub ground_config: GroundConfig,
    pub only_apu_start_loads: bool,
    pub sspc_elcc_loads_enabled: bool,
    pub refuel_on_battery: bool,
    pub ground_service_mode: bool,
    pub ext_full_power: bool,
    pub capt_inst: InstFeed,
    pub fo_inst: InstFeed,
    pub standby: InstFeed,
    pub rat_deploy: bool,
    pub rat_cb_close: bool,
    pub bb_iso_open: bool,
    pub c1_tru_iso_open: bool,
    pub controlling: [bool; 2],
    pub all_ac_lost: bool,
    pub ccs_powered: bool,
    pub btb_fail_to_close: [bool; BTB_COUNT],
    pub bsb_fail_to_close: [bool; 2],
}

#[derive(Clone, Debug)]
pub struct BpcuPair {
    ext_ok_since: [Option<u64>; 3],
    ext_avail: [bool; 3],
    ext_ready: [bool; 3],
    last_feed: [Feed; BUS_COUNT],
    refuel_since: Option<u64>,
    no_ac_since: Option<u64>,
    rat_deployed: bool,
    last_transfer_us: Option<u64>,
    mbr_hold_until: u64,
    mbr_cmd_prev: bool,
    ccs_hold_until: u64,
    rcb_latched: bool,
    was_on_ground: bool,
    air_since: Option<u64>,
    btb_wait_since: [Option<u64>; BTB_COUNT],
    btb_failed: [bool; BTB_COUNT],
    bsb_wait_since: [Option<u64>; 2],
    bsb_failed: [bool; 2],
    batt_only_hold_until: u64,
    batt_off_since: Option<u64>,
    standby_low_since: Option<u64>,
    pub left_powered: bool,
    pub right_powered: bool,
}

impl Default for BpcuPair {
    fn default() -> Self {
        Self::new()
    }
}

fn src_index(s: Src) -> usize {
    match s {
        Src::Vfsg(k) => k as usize,
        Src::Asg(k) => 4 + k as usize,
        Src::Ext(k) => 6 + k as usize,
    }
}

fn attach_node(s: Src) -> usize {
    match s {
        Src::Vfsg(k) => k as usize,
        Src::Asg(k) => APB_NODE[k as usize],
        Src::Ext(k) => EPC_BUS[k as usize],
    }
}

fn find_path(
    start: usize,
    target: usize,
    owner: &[Option<Src>; NODE_COUNT],
    src: Src,
    failed: &[bool; BTB_COUNT],
    tie_fault: &[bool; 2],
) -> Option<(Vec<usize>, Vec<usize>)> {
    let ok = |n: usize| {
        let faulted = n >= BUS_COUNT && tie_fault[n - BUS_COUNT];
        !faulted && (owner[n].is_none() || owner[n] == Some(src))
    };
    if !ok(start) || !ok(target) {
        return None;
    }
    let mut prev: [Option<(usize, usize)>; NODE_COUNT] = [None; NODE_COUNT];
    let mut seen = [false; NODE_COUNT];
    let mut queue = vec![start];
    seen[start] = true;
    let mut head = 0;
    while head < queue.len() {
        let n = queue[head];
        head += 1;
        if n == target {
            break;
        }
        for (e, &(a, b)) in BTB_NODES.iter().enumerate() {
            if failed[e] {
                continue;
            }
            let m = if a == n {
                b
            } else if b == n {
                a
            } else {
                continue;
            };
            if !seen[m] && ok(m) {
                seen[m] = true;
                prev[m] = Some((n, e));
                queue.push(m);
            }
        }
    }
    if !seen[target] {
        return None;
    }
    let mut nodes = vec![target];
    let mut edges = Vec::new();
    let mut cur = target;
    while let Some((p, e)) = prev[cur] {
        nodes.push(p);
        edges.push(e);
        cur = p;
    }
    Some((nodes, edges))
}

struct Plan {
    feed: [Feed; BUS_COUNT],
    gcb: [bool; BUS_COUNT],
    apb: [bool; 2],
    epc: [bool; 2],
    btb: [bool; BTB_COUNT],
}

impl BpcuPair {
    pub fn new() -> Self {
        Self {
            ext_ok_since: [None; 3],
            ext_avail: [false; 3],
            ext_ready: [false; 3],
            last_feed: [Feed::None; BUS_COUNT],
            refuel_since: None,
            no_ac_since: None,
            rat_deployed: false,
            last_transfer_us: None,
            mbr_hold_until: 0,
            mbr_cmd_prev: false,
            ccs_hold_until: 0,
            rcb_latched: false,
            was_on_ground: true,
            air_since: None,
            btb_wait_since: [None; BTB_COUNT],
            btb_failed: [false; BTB_COUNT],
            bsb_wait_since: [None; 2],
            bsb_failed: [false; 2],
            batt_only_hold_until: 0,
            batt_off_since: None,
            standby_low_since: None,
            left_powered: true,
            right_powered: true,
        }
    }

    pub fn rat_deployed(&self) -> bool {
        self.rat_deployed
    }

    fn update_ext(&mut self, i: &BpcuInput) {
        for k in 0..3 {
            let good = i.ext_plugged[k] && i.ext_quality_ok[k];
            if good {
                let since = *self.ext_ok_since[k].get_or_insert(i.now_us);
                let held = i.now_us.saturating_sub(since);
                self.ext_avail[k] = held >= EXT_QUALITY_US;
                self.ext_ready[k] = held >= EXT_CONTACTOR_HOLD_US;
            } else {
                self.ext_ok_since[k] = None;
                self.ext_avail[k] = false;
                self.ext_ready[k] = false;
            }
        }
    }

    fn available(&self, s: Src, i: &BpcuInput) -> bool {
        match s {
            Src::Vfsg(k) => i.gen_ok[k as usize] && i.gen_ctrl_on[k as usize],
            Src::Asg(k) => i.asg_ok[k as usize] && i.asg_ctrl_on[k as usize] && !i.tie_fault[k as usize],
            Src::Ext(k) => i.on_ground && self.ext_ready[k as usize],
        }
    }

    fn plan(&self, i: &BpcuInput) -> Plan {
        let mut p = Plan {
            feed: [Feed::None; BUS_COUNT],
            gcb: [false; BUS_COUNT],
            apb: [false; 2],
            epc: [false; 2],
            btb: [false; BTB_COUNT],
        };
        let mut owner: [Option<Src>; NODE_COUNT] = [None; NODE_COUNT];
        for b in 0..BUS_COUNT {
            let s = Src::Vfsg(b as u8);
            if self.available(s, i) {
                owner[b] = Some(s);
                p.feed[b] = Feed::Src(s);
                p.gcb[b] = true;
            }
        }
        for b in 0..BUS_COUNT {
            if p.feed[b] != Feed::None {
                continue;
            }
            let ranks = if i.on_ground {
                &RANK_GROUND[b]
            } else {
                &RANK_AIR[b]
            };
            for &s in ranks {
                if !self.available(s, i) {
                    continue;
                }
                let Some((nodes, edges)) =
                    find_path(attach_node(s), b, &owner, s, &self.btb_failed, &i.tie_fault)
                else {
                    continue;
                };
                for n in nodes {
                    owner[n] = Some(s);
                }
                for e in edges {
                    p.btb[e] = true;
                }
                match s {
                    Src::Asg(k) => p.apb[k as usize] = true,
                    Src::Ext(k) => p.epc[k as usize] = true,
                    Src::Vfsg(_) => {}
                }
                p.feed[b] = Feed::Src(s);
                break;
            }
        }
        let degraded = (0..BUS_COUNT).any(|b| p.feed[b] != Feed::Src(Src::Vfsg(b as u8)));
        for k in 0..2 {
            let node = APB_NODE[k];
            if degraded && owner[node].is_none() && self.available(Src::Asg(k as u8), i) {
                owner[node] = Some(Src::Asg(k as u8));
                p.apb[k] = true;
            }
        }
        p
    }

    pub fn step(&mut self, i: &BpcuInput) -> BpcuOutput {
        self.update_ext(i);
        let plan = self.plan(i);
        let mut want = [false; 14];
        let mut aux = [false; 14];
        let mut nodes: [[Option<usize>; 2]; 14] = [[None; 2]; 14];
        for b in 0..4 {
            want[b] = plan.gcb[b];
            aux[b] = i.gcb_aux_closed[b];
            nodes[b] = [Some(b), None];
        }
        for k in 0..2 {
            want[4 + k] = plan.apb[k];
            aux[4 + k] = i.asg_cb_aux_closed[k];
            nodes[4 + k] = [Some(APB_NODE[k]), None];
            want[6 + k] = plan.epc[k];
            aux[6 + k] = i.epc_aux_closed[k];
            nodes[6 + k] = [Some(EPC_BUS[k]), None];
        }
        for e in 0..BTB_COUNT {
            want[8 + e] = plan.btb[e];
            aux[8 + e] = i.btb_aux_closed[e];
            nodes[8 + e] = [Some(BTB_NODES[e].0), Some(BTB_NODES[e].1)];
        }
        let mut cmd = [false; 14];
        let mut pending = false;
        for c in 0..14 {
            if !want[c] {
                if aux[c] {
                    pending = true;
                }
                continue;
            }
            let mut blocked = false;
            for d in 0..14 {
                if d != c && aux[d] && !want[d] {
                    let shares = nodes[c]
                        .iter()
                        .flatten()
                        .any(|n| nodes[d].iter().flatten().any(|m| m == n));
                    if shares {
                        blocked = true;
                    }
                }
            }
            cmd[c] = !blocked;
            if blocked || !aux[c] {
                pending = true;
            }
        }
        let mut gcb = [false; 4];
        gcb.copy_from_slice(&cmd[0..4]);
        let asg = [cmd[4], cmd[5]];
        let aft_ok = i.on_ground && self.ext_ready[2] && i.ext_switch_on[2];
        let epc = [cmd[6], cmd[7], aft_ok];
        let mut btb = [false; BTB_COUNT];
        btb.copy_from_slice(&cmd[8..14]);
        for e in 0..BTB_COUNT {
            if self.btb_failed[e] {
                btb[e] = false;
            }
            latch_fail_to_close(
                &mut self.btb_wait_since[e],
                &mut self.btb_failed[e],
                btb[e] && !i.btb_aux_closed[e],
                i.now_us,
                FAIL_TO_CLOSE_BTB_US,
            );
        }
        let plan = plan.feed;
        let bus_powered = [0, 1, 2, 3].map(|b| i.bus_volts[b] >= BUS_LIVE_VOLTS);
        let any_ac = bus_powered.iter().any(|&p| p);
        let mut used = [false; SRC_COUNT];
        for f in plan {
            if let Feed::Src(s) = f {
                used[src_index(s)] = true;
            }
        }
        let mut kva = 0.0;
        for k in 0..SRC_COUNT {
            if used[k] {
                kva += if k < 4 {
                    VFSG_KVA
                } else if k < 6 {
                    ASG_KVA.min(apu_continuous_kva(i.alt_ft) / 2.0)
                } else {
                    EXT_KVA
                };
            }
        }
        let source_change = plan != self.last_feed;
        self.last_feed = plan;
        if pending || source_change {
            self.last_transfer_us = Some(i.now_us);
        }
        if self.was_on_ground && !i.on_ground {
            self.air_since = Some(i.now_us);
        }
        if i.on_ground {
            self.air_since = None;
        }
        self.was_on_ground = i.on_ground;
        let standby_low = (0..2).any(|k| {
            i.instr_bus_v[k] <= STANDBY_PARALLEL_INSTR_V
                && i.instr_tru_a[k] < STANDBY_PARALLEL_TRU_A
                && (any_ac || i.backup_bus_ok)
        });
        let standby_parallel = if standby_low {
            let since = *self.standby_low_since.get_or_insert(i.now_us);
            i.now_us.saturating_sub(since) >= STANDBY_PARALLEL_US
        } else {
            self.standby_low_since = None;
            false
        };
        let instr_parallel = pending
            || standby_parallel
            || self
                .last_transfer_us
                .is_some_and(|t| i.now_us.saturating_sub(t) < INSTR_PARALLEL_HOLD_US)
            || self
                .air_since
                .is_some_and(|t| i.now_us.saturating_sub(t) < CIT_GROUND_TO_AIR_US);

        let (l_src, r_src) = (i.atu_ok[0], i.atu_ok[1]);
        let btb115 = l_src != r_src;
        let bsb = [l_src && !self.bsb_failed[0], r_src && !self.bsb_failed[1]];
        for k in 0..2 {
            latch_fail_to_close(
                &mut self.bsb_wait_since[k],
                &mut self.bsb_failed[k],
                bsb[k] && !i.bsb_aux_closed[k],
                i.now_us,
                FAIL_TO_CLOSE_BSB_US,
            );
        }

        let ext_selected_on = (0..3).any(|k| self.ext_avail[k] && i.ext_switch_on[k]);
        let fwd_avail = (0..2).any(|k| self.ext_avail[k] && i.ext_hz[k] <= GROUND_HANDLING_MAX_HZ);
        let other_source = (0..BUS_COUNT).any(|b| i.gen_ok[b]) || i.asg_ok[0] || i.asg_ok[1];
        let ground_handling = i.on_ground && fwd_avail && !ext_selected_on && !other_source;
        let ground_config = if ground_handling {
            GroundConfig::GroundHandling
        } else if i.on_ground && ext_selected_on {
            GroundConfig::ExtPowerOn
        } else if i.on_ground && !any_ac && (i.batt_switch_on || i.now_us < self.batt_only_hold_until) {
            GroundConfig::BatteryOnly
        } else {
            GroundConfig::FlightOrNormal
        };
        let batt_only = ground_config == GroundConfig::BatteryOnly;
        if batt_only && i.batt_switch_on {
            self.batt_only_hold_until = i.now_us + GROUND_BATTERY_ONLY_OFF_US;
        } else if !batt_only {
            self.batt_only_hold_until = 0;
        }
        let refuel_req = i.refuel_panel_active && i.batt_switch_on && i.on_ground && !any_ac;
        let refuel_on_battery = if refuel_req {
            let since = *self.refuel_since.get_or_insert(i.now_us);
            i.now_us.saturating_sub(since) < REFUEL_BATTERY_LIMIT_US
        } else {
            self.refuel_since = None;
            false
        };

        if !bus_powered[0] && !i.backup_bus_ok && !i.on_ground {
            self.no_ac_since.get_or_insert(i.now_us);
        } else {
            self.no_ac_since = None;
        }
        let rat_demand = self
            .no_ac_since
            .is_some_and(|t| i.now_us.saturating_sub(t) >= RAT_REQUEST_DELAY_US);
        if rat_demand {
            self.rat_deployed = true;
        }
        if any_ac || (i.backup_bus_ok && !self.rcb_latched) {
            self.rat_deployed = false;
        }
        let rat_only = self.rat_deployed && i.rat_gen_ok && !any_ac && (!i.backup_bus_ok || self.rcb_latched);
        self.rcb_latched = rat_only;

        let dc_any = i.dc_main_ok[0] || i.dc_main_ok[1];
        let normal_dc = i.backup_bus_ok || any_ac;
        let capt_inst = if rat_only {
            InstFeed::BackupBus
        } else if normal_dc && i.dc_main_ok[0] {
            InstFeed::MainDc
        } else if i.batt_switch_on || (!i.on_ground && !any_ac) {
            InstFeed::MainBattery
        } else {
            InstFeed::None
        };
        let fo_inst = if rat_only {
            InstFeed::BackupBus
        } else if normal_dc && (i.dc_main_ok[1] || dc_any) {
            InstFeed::MainDc
        } else if capt_inst == InstFeed::MainBattery {
            InstFeed::MainBattery
        } else {
            InstFeed::None
        };
        let standby = if dc_any {
            InstFeed::MainDc
        } else if i.batt_switch_on {
            InstFeed::MainBattery
        } else {
            InstFeed::None
        };

        let mbr_cmd = i.batt_switch_on || pending || self.rat_deployed;
        if mbr_cmd {
            self.batt_off_since = None;
        } else if self.batt_off_since.is_none() {
            self.batt_off_since = Some(i.now_us);
        }
        let off_expired = i.on_ground
            && self
                .batt_off_since
                .is_some_and(|t| i.now_us.saturating_sub(t) >= BATTERY_OFF_MAX_US);
        if mbr_cmd {
            self.mbr_hold_until = i.now_us;
        } else if self.mbr_cmd_prev {
            let shutdown = i.on_ground && !any_ac && i.engines_running == 0;
            let hold = if shutdown {
                MBR_SHUTDOWN_HOLD_US
            } else {
                MBR_OPEN_DELAY_US
            };
            self.mbr_hold_until = i.now_us + hold;
        }
        self.mbr_cmd_prev = mbr_cmd;
        let mbr = mbr_cmd || (i.now_us < self.mbr_hold_until && !off_expired);
        if mbr || dc_any || any_ac {
            self.ccs_hold_until = i.now_us + CCS_HOLD_US;
        }
        let ccs_powered = i.now_us < self.ccs_hold_until;
        let both_ok = self.left_powered && self.right_powered;
        let controlling = [
            self.left_powered && (both_ok || !self.right_powered),
            self.right_powered && (both_ok || !self.left_powered),
        ];
        let live = self.left_powered || self.right_powered;
        if !live {
            self.btb_failed = [false; BTB_COUNT];
            self.bsb_failed = [false; 2];
            self.btb_wait_since = [None; BTB_COUNT];
            self.bsb_wait_since = [None; 2];
        }
        let gate = |v: bool| v && live;
        let gate4 = |a: [bool; 4]| a.map(gate);
        let gate6 = |a: [bool; BTB_COUNT]| a.map(gate);

        BpcuOutput {
            gcb_close: gate4(gcb),
            asg_cb_close: asg.map(gate),
            epc_close: epc.map(gate),
            btb_close: gate6(btb),
            bsb_close: bsb.map(gate),
            btb115_close: [gate(btb115), gate(btb115)],
            bus_feed: plan,
            bus_powered,
            ext_avail: self.ext_avail,
            ext_on: [0, 1, 2].map(|k| self.ext_ready[k] && i.ext_switch_on[k] && i.on_ground),
            source_kva: kva,
            source_change,
            main_batt_relay_close: mbr,
            transfer_pending: pending,
            instr_parallel,
            apu_batt_relay_close: i.apu_batt_switch_on || (batt_only && i.apu_start_requested),
            ground_config,
            only_apu_start_loads: batt_only,
            sspc_elcc_loads_enabled: !batt_only,
            refuel_on_battery,
            ground_service_mode: ground_config == GroundConfig::GroundHandling && i.ground_service_switch,
            ext_full_power: ext_selected_on && i.on_ground,
            capt_inst,
            fo_inst,
            standby,
            rat_deploy: self.rat_deployed,
            rat_cb_close: rat_only,
            bb_iso_open: rat_only,
            c1_tru_iso_open: rat_only,
            controlling,
            all_ac_lost: !any_ac,
            ccs_powered,
            btb_fail_to_close: self.btb_failed,
            bsb_fail_to_close: self.bsb_failed,
        }
    }
}

fn latch_fail_to_close(
    since: &mut Option<u64>,
    failed: &mut bool,
    waiting: bool,
    now_us: u64,
    limit_us: u64,
) {
    if waiting {
        let t0 = *since.get_or_insert(now_us);
        if now_us.saturating_sub(t0) >= limit_us {
            *failed = true;
            *since = None;
        }
    } else {
        *since = None;
    }
}

pub fn loop_plant(i: &mut BpcuInput, o: &BpcuOutput, dt_us: u64) {
    i.now_us += dt_us;
    i.gcb_aux_closed = o.gcb_close;
    i.asg_cb_aux_closed = o.asg_cb_close;
    i.epc_aux_closed = o.epc_close;
    i.btb_aux_closed = o.btb_close;
    i.bsb_aux_closed = o.bsb_close;
    i.btb115_aux_closed = o.btb115_close;
}

#[cfg(test)]
mod apu_capability_tests {
    use super::*;

    #[test]
    fn apu_continuous_capability_falls_with_altitude() {
        assert_eq!(apu_continuous_kva(0.0), 450.0);
        assert_eq!(apu_continuous_kva(43_100.0), 100.0);
        assert!((apu_continuous_kva(21_550.0) - 275.0).abs() < 1e-9);
        assert_eq!(apu_continuous_kva(-500.0), 450.0);
    }
}
