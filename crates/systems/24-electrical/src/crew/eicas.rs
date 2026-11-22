use super::inputs::{CrewInputs, FlightPhase};

const GEN_NAMES: [&str; 4] = ["L1", "L2", "R1", "R2"];
const SIDE: [&str; 2] = ["L", "R"];
const MC_NAMES: [&str; 4] = ["C1", "L", "C2", "R"];
const TRU_NAMES: [&str; 4] = ["C1", "C2", "L", "R"];
const CAC_NAMES: [&str; 4] = [
    "CAC L1/START L1",
    "CAC L2/START L2",
    "CAC R1/START R1",
    "CAC R2/START R2",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Warning,
    Caution,
    Advisory,
    Status,
    Memo,
}

impl Level {
    fn rank(self) -> u8 {
        match self {
            Level::Warning => 0,
            Level::Caution => 1,
            Level::Advisory => 2,
            Level::Status => 3,
            Level::Memo => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgId {
    AcBus(u8),
    BatteryOff,
    CabinUtilOff,
    IfeSeatsOff,
    GenDrive(u8),
    GenOffApu(u8),
    GenOff(u8),
    StandbySys,
    MainBattDisch,
    MainBattLow,
    MainBatt,
    ApuBattery,
    EngStarters(u8),
    ApuRunning,
    ApuShutdown,
    MainAcBusIsln(u8),
    Ac115BusIsln(u8),
    Ac115Bus(u8),
    Dc28Bus(u8),
    BackupBus,
    HotBatteryBus,
    FltInstBusCapt,
    FltInstBusFo,
    DcBusIsln,
    GenSys(u8),
    GenSysApu(u8),
    GenCtrlUnit(u8),
    GenDebris(u8),
    GenDebrisSensor(u8),
    GenAirOilValve(u8),
    ApuGenCtrlUnit(u8),
    ApuBatteryCharger,
    ApuBatteryOverheat,
    ApuStarter(u8),
    ApuStartPwrUnit,
    ApuStartSys,
    EngineStarter(u8),
    MainBatteryCharger,
    MainBatteryOverheat,
    BusPwrCtrlUnit(u8),
    Atu(u8),
    Tru(u8),
    McCac(u8),
    McHyd(u8),
    DistrChannel,
    SysCtrlEnable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageDef {
    pub id: MsgId,
    pub name: String,
    pub level: Level,
    pub on_delay_us: u64,
    pub off_delay_us: u64,
    pub latched: bool,
    pub inhibit_takeoff: bool,
    pub inhibit_landing: bool,
    pub delay_approx: bool,
    pub beeper: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EicasLine {
    pub id: MsgId,
    pub name: String,
    pub level: Level,
    pub since_us: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MsgState {
    cond_since: Option<u64>,
    clear_since: Option<u64>,
    active: bool,
    since_us: u64,
    seq: u64,
    acked: bool,
}

#[derive(Clone, Debug)]
pub struct EicasElec {
    defs: Vec<MessageDef>,
    state: Vec<MsgState>,
    counter: u64,
    phase: FlightPhase,
}

impl EicasElec {
    pub fn same_state(&self, o: &EicasElec) -> bool {
        self.counter == o.counter && self.state == o.state
    }
}

const MS: u64 = 1_000;
pub(crate) const DC28_BUS_MSG_MS: u64 = 1380;
pub(crate) const DC28_BUS_MSG_V: f64 = 18.0;
pub(crate) const APU_BATT_OVERHEAT_MS: u64 = 1_000;
pub(crate) const TRU_LOW_CURRENT_A: f64 = 11.0;
pub(crate) const DISCH_HIGH_MS: u64 = 20_000;
pub(crate) const DISCH_LOW_MS: u64 = 600_000;
pub(crate) const HBB_LOW_V: f64 = 22.0;
pub(crate) const HBB_LOW_RESET_V: f64 = 24.0;
pub(crate) const BATTERY_OFF_MSG_MS: u64 = 1_000;
pub(crate) const HBB_LOW_MSG_MS: u64 = 1_500;
pub(crate) const AC_BUS_MSG_V: f64 = 130.5;
pub(crate) const AC_BUS_MSG_MS: u64 = 500;
pub(crate) const APU_CHARGER_MSG_MS: u64 = 1_000;
pub(crate) const GCB_DISAGREE_MS: u64 = 150;
pub(crate) const GCU_28V_FAIL_MS: u64 = 1_500;

fn def(id: MsgId, name: String, level: Level) -> MessageDef {
    let advisory = level == Level::Advisory;
    MessageDef {
        id,
        name,
        level,
        on_delay_us: 0,
        off_delay_us: 0,
        latched: false,
        inhibit_takeoff: advisory,
        inhibit_landing: advisory,
        delay_approx: true,
        beeper: matches!(level, Level::Warning | Level::Caution),
    }
}

fn timed(mut d: MessageDef, on_ms: u64, off_ms: u64) -> MessageDef {
    d.on_delay_us = on_ms * MS;
    d.off_delay_us = off_ms * MS;
    d.delay_approx = false;
    d
}

fn latched(mut d: MessageDef) -> MessageDef {
    d.latched = true;
    d
}

pub fn message_catalogue() -> Vec<MessageDef> {
    let mut v = Vec::new();
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(timed(
            def(MsgId::AcBus(i as u8), format!("ELEC AC BUS {n}"), Level::Caution),
            AC_BUS_MSG_MS,
            AC_BUS_MSG_MS,
        ));
    }
    v.push(timed(
        def(MsgId::BatteryOff, "ELEC BATTERY OFF".to_string(), Level::Advisory),
        BATTERY_OFF_MSG_MS,
        BATTERY_OFF_MSG_MS,
    ));
    v.push(timed(
        def(
            MsgId::CabinUtilOff,
            "ELEC CABIN/UTIL OFF".to_string(),
            Level::Advisory,
        ),
        500,
        0,
    ));
    v.push(timed(
        def(
            MsgId::IfeSeatsOff,
            "ELEC IFE/SEATS OFF".to_string(),
            Level::Advisory,
        ),
        500,
        0,
    ));
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(latched(def(
            MsgId::GenDrive(i as u8),
            format!("ELEC GEN DRIVE {n}"),
            Level::Advisory,
        )));
    }
    for (s, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::GenOffApu(s as u8),
            format!("ELEC GEN OFF APU {n}"),
            Level::Advisory,
        ));
    }
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(def(
            MsgId::GenOff(i as u8),
            format!("ELEC GEN OFF {n}"),
            Level::Advisory,
        ));
    }
    v.push(def(
        MsgId::StandbySys,
        "ELEC STANDBY SYS".to_string(),
        Level::Advisory,
    ));
    v.push(timed(
        def(
            MsgId::MainBattDisch,
            "MAIN BATTERY DISCH".to_string(),
            Level::Advisory,
        ),
        HBB_LOW_MSG_MS,
        0,
    ));
    v.push(def(
        MsgId::MainBattLow,
        "MAIN BATTERY LOW".to_string(),
        Level::Advisory,
    ));
    v.push(def(MsgId::MainBatt, "MAIN BATTERY".to_string(), Level::Advisory));
    v.push(def(MsgId::ApuBattery, "APU BATTERY".to_string(), Level::Advisory));
    for (e, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::EngStarters(e as u8),
            format!("ENG STARTERS {n}"),
            Level::Advisory,
        ));
    }
    v.push(def(MsgId::ApuRunning, "APU RUNNING".to_string(), Level::Memo));
    v.push(def(
        MsgId::ApuShutdown,
        "APU SHUTDOWN".to_string(),
        Level::Advisory,
    ));
    for (s, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::MainAcBusIsln(s as u8),
            format!("ELEC MAIN AC BUS ISLN {n}"),
            Level::Status,
        ));
    }
    for (s, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::Ac115BusIsln(s as u8),
            format!("ELEC 115V AC BUS ISLN {n}"),
            Level::Status,
        ));
    }
    for (s, n) in SIDE.iter().enumerate() {
        v.push(timed(
            def(
                MsgId::Ac115Bus(s as u8),
                format!("ELEC 115V AC BUS {n}"),
                Level::Status,
            ),
            500,
            500,
        ));
    }
    for (s, n) in SIDE.iter().enumerate() {
        v.push(timed(
            def(
                MsgId::Dc28Bus(s as u8),
                format!("ELEC 28V DC BUS {n}"),
                Level::Status,
            ),
            DC28_BUS_MSG_MS,
            DC28_BUS_MSG_MS,
        ));
    }
    v.push(timed(
        def(MsgId::BackupBus, "ELEC BACKUP BUS".to_string(), Level::Status),
        500,
        500,
    ));
    v.push(timed(
        def(
            MsgId::HotBatteryBus,
            "ELEC HOT BATTERY BUS".to_string(),
            Level::Status,
        ),
        1500,
        0,
    ));
    v.push(timed(
        def(
            MsgId::FltInstBusCapt,
            "ELEC FLT INST BUS CAPT".to_string(),
            Level::Status,
        ),
        1500,
        0,
    ));
    v.push(timed(
        def(
            MsgId::FltInstBusFo,
            "ELEC FLT INST BUS F/O".to_string(),
            Level::Status,
        ),
        1500,
        0,
    ));
    v.push(def(
        MsgId::DcBusIsln,
        "ELEC DC BUS ISLN".to_string(),
        Level::Status,
    ));
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(def(
            MsgId::GenSys(i as u8),
            format!("ELEC GEN SYS {n}"),
            Level::Status,
        ));
    }
    for (s, n) in SIDE.iter().enumerate() {
        v.push(latched(def(
            MsgId::GenSysApu(s as u8),
            format!("ELEC GEN SYS APU {n}"),
            Level::Status,
        )));
    }
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(def(
            MsgId::GenCtrlUnit(i as u8),
            format!("ELEC GEN CTRL UNIT {n}"),
            Level::Status,
        ));
    }
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(latched(def(
            MsgId::GenDebris(i as u8),
            format!("ELEC GEN DEBRIS {n}"),
            Level::Status,
        )));
    }
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(def(
            MsgId::GenDebrisSensor(i as u8),
            format!("ELEC GEN DEBRIS SENSOR {n}"),
            Level::Status,
        ));
    }
    for (e, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::GenAirOilValve(e as u8),
            format!("ELEC GEN {} AIR/OIL VALVE {n}", e + 1),
            Level::Status,
        ));
    }
    for (s, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::ApuGenCtrlUnit(s as u8),
            format!("ELEC APU GEN CTRL UNIT {n}"),
            Level::Status,
        ));
    }
    v.push(timed(
        def(
            MsgId::ApuBatteryCharger,
            "ELEC APU BATTERY CHARGER".to_string(),
            Level::Status,
        ),
        APU_CHARGER_MSG_MS,
        0,
    ));
    v.push(timed(
        def(
            MsgId::ApuBatteryOverheat,
            "ELEC APU BATTERY OVERHEAT".to_string(),
            Level::Status,
        ),
        1000,
        0,
    ));
    for (s, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::ApuStarter(s as u8),
            format!("ELEC APU STARTER {n}"),
            Level::Status,
        ));
    }
    v.push(def(
        MsgId::ApuStartPwrUnit,
        "ELEC APU START PWR UNIT".to_string(),
        Level::Status,
    ));
    v.push(def(
        MsgId::ApuStartSys,
        "APU START SYS".to_string(),
        Level::Status,
    ));
    for (i, n) in GEN_NAMES.iter().enumerate() {
        v.push(def(
            MsgId::EngineStarter(i as u8),
            format!("ELEC ENGINE STARTER {n}"),
            Level::Status,
        ));
    }
    v.push(def(
        MsgId::MainBatteryCharger,
        "ELEC MAIN BATTERY CHARGER".to_string(),
        Level::Status,
    ));
    v.push(def(
        MsgId::MainBatteryOverheat,
        "ELEC MAIN BATTERY OVERHEAT".to_string(),
        Level::Status,
    ));
    for (s, n) in SIDE.iter().enumerate() {
        v.push(def(
            MsgId::BusPwrCtrlUnit(s as u8),
            format!("ELEC BUS PWR CTRL UNIT {n}"),
            Level::Status,
        ));
    }
    for (s, n) in SIDE.iter().enumerate() {
        v.push(def(MsgId::Atu(s as u8), format!("ELEC ATU {n}"), Level::Status));
    }
    for (i, n) in TRU_NAMES.iter().enumerate() {
        v.push(def(MsgId::Tru(i as u8), format!("ELEC TRU {n}"), Level::Status));
    }
    for (i, n) in CAC_NAMES.iter().enumerate() {
        v.push(def(MsgId::McCac(i as u8), format!("ELEC MC {n}"), Level::Status));
    }
    for (i, n) in MC_NAMES.iter().enumerate() {
        v.push(def(
            MsgId::McHyd(i as u8),
            format!("ELEC MC HYD {n}"),
            Level::Status,
        ));
    }
    v.push(def(
        MsgId::DistrChannel,
        "ELEC DISTR CHANNEL".to_string(),
        Level::Status,
    ));
    v.push(def(
        MsgId::SysCtrlEnable,
        "ELEC SYS CTRL ENABLE".to_string(),
        Level::Status,
    ));
    v
}

fn volts_low(volts: f64, low: f64, reset: f64, active: bool) -> bool {
    if active { volts <= reset } else { volts < low }
}

fn condition(id: MsgId, i: &CrewInputs, active: bool) -> bool {
    match id {
        MsgId::AcBus(k) => volts_low(i.ac_bus_volts[k as usize], AC_BUS_MSG_V, 165.0, active),
        MsgId::BatteryOff => !i.battery_switch_on,
        MsgId::CabinUtilOff => !i.cabin_util_on,
        MsgId::IfeSeatsOff => !i.ife_on,
        MsgId::GenDrive(k) => {
            let k = k as usize;
            i.engine_running[k / 2] && (i.vfsg_lop[k] || i.pmg_lost[k] || i.vfsg_oil_high[k])
        }
        MsgId::GenOffApu(s) => {
            let s = s as usize;
            (i.apb_cmd_closed[s] && !i.apb_closed[s]) || (!i.apu_gen_switch_on[s] && i.apu_running)
        }
        MsgId::GenOff(k) => {
            let k = k as usize;
            !i.gcb_closed[k] && !i.gcb_prevented[k]
        }
        MsgId::StandbySys => i.standby_sys_fault,
        MsgId::MainBattDisch => {
            i.main_batt_disch_msg
                || i.main_batt_disch_high_us >= DISCH_HIGH_MS * MS
                || i.main_batt_disch_low_us >= DISCH_LOW_MS * MS
                || i.hot_batt_volts < HBB_LOW_V
        }
        MsgId::MainBattLow => i.main_batt_charge_low,
        MsgId::MainBatt => i.main_batt_failed,
        MsgId::ApuBattery => i.apu_batt_failed,
        MsgId::EngStarters(e) => {
            let e = e as usize;
            i.both_starters_failed[e] || i.mes_unavailable_after_start[e]
        }
        MsgId::ApuRunning => i.apu_running,
        MsgId::ApuShutdown => i.apu_start_failed,
        MsgId::MainAcBusIsln(s) => i.main_ac_bus_isln[s as usize],
        MsgId::Ac115BusIsln(s) => i.ac115_bus_isolated[s as usize],
        MsgId::Ac115Bus(s) => {
            let s = s as usize;
            i.ac_source_online && volts_low(i.ac115_volts[s], 50.0, 85.0, active)
        }
        MsgId::Dc28Bus(s) => volts_low(i.dc28_volts[s as usize], DC28_BUS_MSG_V, 22.0, active),
        MsgId::BackupBus => i.ac_source_online && volts_low(i.backup_volts, 130.0, 165.0, active),
        MsgId::HotBatteryBus => volts_low(i.hot_batt_volts, HBB_LOW_V, HBB_LOW_RESET_V, active),
        MsgId::FltInstBusCapt => volts_low(i.flt_inst_capt_volts, 22.0, 25.0, active),
        MsgId::FltInstBusFo => !i.battery_refuel_mode && volts_low(i.flt_inst_fo_volts, 22.0, 25.0, active),
        MsgId::DcBusIsln => i.dc_bus_isln,
        MsgId::GenSys(k) => {
            let k = k as usize;
            i.gcb_disagree_us[k] > GCB_DISAGREE_MS * MS
                || i.pmg_shorted[k]
                || i.protective_trip[k]
                || i.gcu_28v_fail_us[k] >= GCU_28V_FAIL_MS * MS
                || i.oil_no_lop_after_shutdown[k]
        }
        MsgId::GenSysApu(s) => {
            let s = s as usize;
            i.apb_disagree_us[s] > GCB_DISAGREE_MS * MS
                || i.asg_pmg_shorted[s]
                || i.apb_trip[s]
                || i.apu_gcu_comm_fail[s]
                || i.asg_not_commanded_start[s]
        }
        MsgId::GenCtrlUnit(k) => {
            let k = k as usize;
            i.gcu_inop[k] || i.gcu_comm_loss[k] || i.gcu_fan_inop[k] || i.gcu_data_invalid[k]
        }
        MsgId::GenDebris(k) => i.ecd_debris[k as usize],
        MsgId::GenDebrisSensor(k) => i.ecd_inop[k as usize],
        MsgId::GenAirOilValve(e) => i.oil_cooler_valve_fault[e as usize],
        MsgId::ApuGenCtrlUnit(s) => i.apu_gcu_internal_fault[s as usize],
        MsgId::ApuBatteryCharger => i.apu_batt_charger_fail,
        MsgId::ApuBatteryOverheat => i.apu_batt_overtemp_us >= APU_BATT_OVERHEAT_MS * MS,
        MsgId::ApuStarter(s) => {
            let s = s as usize;
            i.apu_starter_inop[s] || (i.apu_start_config_bad && i.apu_start_signal)
        }
        MsgId::ApuStartPwrUnit => i.spu_main_fail || i.spu_housekeeping_fail,
        MsgId::ApuStartSys => i.apu_start_sys_inop,
        MsgId::EngineStarter(k) => i.engine_starter_inop[k as usize],
        MsgId::MainBatteryCharger => i.main_batt_charger_fail,
        MsgId::MainBatteryOverheat => i.main_batt_overheat,
        MsgId::BusPwrCtrlUnit(s) => i.bpcu_inop[s as usize],
        MsgId::Atu(s) => i.atu_low_current[s as usize],
        MsgId::Tru(k) => {
            let k = k as usize;
            i.tru_relay_failed_open[k] || (i.tru_ac_supplied[k] && i.tru_current_a[k] < TRU_LOW_CURRENT_A)
        }
        MsgId::McCac(k) => i.mc_cac_inop[k as usize],
        MsgId::McHyd(k) => i.emp_mc_inop[k as usize],
        MsgId::DistrChannel => i.distr_channel_fault,
        MsgId::SysCtrlEnable => i.sys_ctrl_enable_fault,
    }
}

fn reset(id: MsgId, i: &CrewInputs) -> bool {
    match id {
        MsgId::GenDrive(k) => {
            let k = k as usize;
            i.engine_running[k / 2]
                && !i.vfsg_lop[k]
                && !i.pmg_lost[k]
                && !i.vfsg_oil_high[k]
                && !i.drive_disconnected[k]
        }
        MsgId::GenDebris(k) => i.gen_ctrl_cycled[k as usize],
        MsgId::GenSysApu(s) => i.apu_gen_cycled[s as usize],
        _ => true,
    }
}

impl Default for EicasElec {
    fn default() -> Self {
        Self::new()
    }
}

impl EicasElec {
    pub fn new() -> Self {
        let defs = message_catalogue();
        let state = vec![MsgState::default(); defs.len()];
        Self {
            defs,
            state,
            counter: 0,
            phase: FlightPhase::Ground,
        }
    }

    pub fn message_count(&self) -> usize {
        self.defs.len()
    }

    pub fn defs(&self) -> &[MessageDef] {
        &self.defs
    }

    pub fn update(&mut self, inp: &CrewInputs) {
        self.phase = inp.phase;
        let now = inp.now_us;
        for (d, st) in self.defs.iter().zip(self.state.iter_mut()) {
            let c = condition(d.id, inp, st.active);
            if st.active {
                let clearing = if d.latched { !c && reset(d.id, inp) } else { !c };
                if clearing {
                    let since = *st.clear_since.get_or_insert(now);
                    if now.saturating_sub(since) >= d.off_delay_us {
                        st.active = false;
                        st.clear_since = None;
                        st.cond_since = None;
                        st.acked = false;
                    }
                } else {
                    st.clear_since = None;
                }
            } else if c {
                let since = *st.cond_since.get_or_insert(now);
                if now.saturating_sub(since) >= d.on_delay_us {
                    st.active = true;
                    st.since_us = now;
                    st.seq = self.counter;
                    self.counter += 1;
                    st.clear_since = None;
                    st.acked = false;
                }
            } else {
                st.cond_since = None;
            }
        }
    }

    fn inhibited(&self, d: &MessageDef) -> bool {
        match self.phase {
            FlightPhase::TakeoffInhibit => d.inhibit_takeoff,
            FlightPhase::LandingInhibit => d.inhibit_landing,
            _ => false,
        }
    }

    pub fn is_active(&self, id: MsgId) -> bool {
        self.defs
            .iter()
            .zip(self.state.iter())
            .any(|(d, s)| d.id == id && s.active)
    }

    pub fn is_shown(&self, id: MsgId) -> bool {
        self.defs
            .iter()
            .zip(self.state.iter())
            .any(|(d, s)| d.id == id && s.active && !self.inhibited(d))
    }

    pub fn list(&self) -> Vec<EicasLine> {
        let mut rows: Vec<(u8, u64, u64, EicasLine)> = self
            .defs
            .iter()
            .zip(self.state.iter())
            .filter(|(d, s)| s.active && !self.inhibited(d))
            .map(|(d, s)| {
                (
                    d.level.rank(),
                    s.since_us,
                    s.seq,
                    EicasLine {
                        id: d.id,
                        name: d.name.clone(),
                        level: d.level,
                        since_us: s.since_us,
                    },
                )
            })
            .collect();
        rows.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| b.1.cmp(&a.1))
                .then_with(|| b.2.cmp(&a.2))
        });
        rows.into_iter().map(|r| r.3).collect()
    }

    pub fn names(&self) -> Vec<String> {
        self.list().into_iter().map(|l| l.name).collect()
    }

    pub fn beeper(&self) -> bool {
        self.defs
            .iter()
            .zip(self.state.iter())
            .any(|(d, s)| d.beeper && s.active && !s.acked && !self.inhibited(d))
    }

    pub fn cancel_aural(&mut self) {
        for s in &mut self.state {
            if s.active {
                s.acked = true;
            }
        }
    }

    pub fn master_caution(&self) -> bool {
        self.beeper()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crew::inputs::classify_phase;

    fn healthy() -> CrewInputs {
        CrewInputs {
            battery_switch_on: true,
            apu_gen_switch_on: [true; 2],
            gen_ctrl_on: [true; 4],
            cabin_util_on: true,
            ife_on: true,
            gcb_closed: [true; 4],
            ac_bus_energized: [true; 4],
            ac_bus_volts: [235.0; 4],
            ac115_volts: [115.0; 2],
            dc28_volts: [28.0; 2],
            backup_volts: 235.0,
            flt_inst_capt_volts: 28.0,
            flt_inst_fo_volts: 28.0,
            hot_batt_volts: 28.0,
            ac_source_online: true,
            main_batt_current_a: 2.0,
            engine_running: [true; 2],
            ..CrewInputs::default()
        }
    }

    fn step(e: &mut EicasElec, i: &mut CrewInputs, dt_us: u64) {
        i.now_us += dt_us;
        e.update(i);
    }

    fn settle(e: &mut EicasElec, i: &mut CrewInputs) {
        step(e, i, 1_000);
        step(e, i, 30_000_000);
    }

    #[test]
    fn healthy_shows_nothing() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        settle(&mut e, &mut i);
        assert!(e.list().is_empty());
    }

    #[test]
    fn catalogue_has_expected_levels() {
        let e = EicasElec::new();
        assert!(e.message_count() >= 80);
        let cautions = e.defs().iter().filter(|d| d.level == Level::Caution).count();
        assert_eq!(cautions, 4);
        assert!(
            e.defs()
                .iter()
                .filter(|d| d.level == Level::Caution)
                .all(|d| d.beeper)
        );
        assert!(e.defs().iter().all(|d| d.level != Level::Warning));
    }

    #[test]
    fn ac_bus_threshold_delay_and_hysteresis() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.ac_bus_volts[0] = 129.0;
        step(&mut e, &mut i, 100_000);
        step(&mut e, &mut i, 300_000);
        assert!(!e.is_active(MsgId::AcBus(0)));
        step(&mut e, &mut i, 200_000);
        assert!(e.is_shown(MsgId::AcBus(0)));
        assert!(e.beeper());
        e.cancel_aural();
        assert!(!e.beeper());
        i.ac_bus_volts[0] = 150.0;
        step(&mut e, &mut i, 1_000_000);
        assert!(e.is_active(MsgId::AcBus(0)));
        i.ac_bus_volts[0] = 166.0;
        step(&mut e, &mut i, 100_000);
        assert!(e.is_active(MsgId::AcBus(0)));
        step(&mut e, &mut i, 500_000);
        assert!(!e.is_active(MsgId::AcBus(0)));
    }

    #[test]
    fn ac_bus_trips_below_130_5_only() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.ac_bus_volts[1] = AC_BUS_MSG_V;
        settle(&mut e, &mut i);
        assert!(!e.is_active(MsgId::AcBus(1)));
        i.ac_bus_volts[1] = AC_BUS_MSG_V - 0.2;
        settle(&mut e, &mut i);
        assert!(e.is_active(MsgId::AcBus(1)));
    }

    #[test]
    fn short_glitch_does_not_show() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.ac_bus_volts[3] = 0.0;
        step(&mut e, &mut i, 300_000);
        i.ac_bus_volts[3] = 235.0;
        step(&mut e, &mut i, 100_000);
        i.ac_bus_volts[3] = 0.0;
        step(&mut e, &mut i, 300_000);
        assert!(!e.is_active(MsgId::AcBus(3)));
    }

    #[test]
    fn gen_off_follows_gcb_unless_prevented() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.gcb_closed[1] = false;
        step(&mut e, &mut i, 10);
        assert!(e.is_shown(MsgId::GenOff(1)));
        i.gcb_prevented[1] = true;
        step(&mut e, &mut i, 10);
        assert!(!e.is_active(MsgId::GenOff(1)));
        i.gcb_prevented[1] = false;
        i.gcb_closed[1] = true;
        step(&mut e, &mut i, 10);
        assert!(!e.is_active(MsgId::GenOff(1)));
    }

    #[test]
    fn gen_sys_conditions() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.gcb_disagree_us[0] = 150_000;
        i.gcu_28v_fail_us[1] = 1_400_000;
        step(&mut e, &mut i, 10);
        assert!(!e.is_active(MsgId::GenSys(0)));
        assert!(!e.is_active(MsgId::GenSys(1)));
        i.gcb_disagree_us[0] = 151_000;
        i.gcu_28v_fail_us[1] = 1_500_000;
        i.pmg_shorted[2] = true;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenSys(0)));
        assert!(e.is_active(MsgId::GenSys(1)));
        assert!(e.is_active(MsgId::GenSys(2)));
        assert!(!e.is_active(MsgId::GenSys(3)));
    }

    #[test]
    fn gen_drive_latches_until_reset() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.vfsg_lop[0] = true;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenDrive(0)));
        i.vfsg_lop[0] = false;
        i.drive_disconnected[0] = true;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenDrive(0)));
        i.drive_disconnected[0] = false;
        step(&mut e, &mut i, 10);
        assert!(!e.is_active(MsgId::GenDrive(0)));
    }

    #[test]
    fn gen_drive_pmg_loss_without_engine_is_silent() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.engine_running = [false; 2];
        i.pmg_lost[2] = true;
        step(&mut e, &mut i, 10);
        assert!(!e.is_active(MsgId::GenDrive(2)));
        i.engine_running = [true; 2];
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenDrive(2)));
    }

    #[test]
    fn gen_debris_latches_until_switch_cycled() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.ecd_debris[2] = true;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenDebris(2)));
        i.ecd_debris[2] = false;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenDebris(2)));
        i.gen_ctrl_cycled[2] = true;
        step(&mut e, &mut i, 10);
        assert!(!e.is_active(MsgId::GenDebris(2)));
    }

    #[test]
    fn gen_sys_apu_latches_until_apu_gen_cycled() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.apb_trip[1] = true;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenSysApu(1)));
        i.apb_trip[1] = false;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenSysApu(1)));
        i.apu_gen_cycled[1] = true;
        step(&mut e, &mut i, 10);
        assert!(!e.is_active(MsgId::GenSysApu(1)));
    }

    #[test]
    fn advisories_inhibited_in_takeoff_and_landing_but_not_cautions_or_status() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.phase = FlightPhase::TakeoffInhibit;
        i.main_batt_failed = true;
        i.ecd_inop[0] = true;
        i.ac_bus_volts[1] = 0.0;
        settle(&mut e, &mut i);
        assert!(e.is_active(MsgId::MainBatt));
        assert!(!e.is_shown(MsgId::MainBatt));
        assert!(e.is_shown(MsgId::AcBus(1)));
        assert!(e.is_shown(MsgId::GenDebrisSensor(0)));
        i.phase = FlightPhase::LandingInhibit;
        step(&mut e, &mut i, 1_000);
        assert!(!e.is_shown(MsgId::MainBatt));
        i.phase = FlightPhase::Flight;
        step(&mut e, &mut i, 1_000);
        assert!(e.is_shown(MsgId::MainBatt));
    }

    #[test]
    fn newest_first_within_level_and_level_order() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 1_000);
        i.ecd_inop[0] = true;
        step(&mut e, &mut i, 1_000);
        i.main_batt_failed = true;
        step(&mut e, &mut i, 1_000);
        i.apu_batt_failed = true;
        step(&mut e, &mut i, 1_000);
        i.ac_bus_volts[2] = 0.0;
        step(&mut e, &mut i, 1_000);
        step(&mut e, &mut i, 1_000_000);
        let n = e.names();
        assert_eq!(n[0], "ELEC AC BUS R1");
        assert_eq!(n[1], "APU BATTERY");
        assert_eq!(n[2], "MAIN BATTERY");
        assert_eq!(n[3], "ELEC GEN DEBRIS SENSOR L1");
    }

    #[test]
    fn battery_off_one_second_and_reset_delay() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.battery_switch_on = false;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 900_000);
        assert!(!e.is_active(MsgId::BatteryOff));
        step(&mut e, &mut i, 100_000);
        assert!(e.is_shown(MsgId::BatteryOff));
        i.battery_switch_on = true;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 900_000);
        assert!(e.is_active(MsgId::BatteryOff));
        step(&mut e, &mut i, 200_000);
        assert!(!e.is_active(MsgId::BatteryOff));
    }

    #[test]
    fn cabin_and_ife_500ms() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.cabin_util_on = false;
        i.ife_on = false;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 400_000);
        assert!(!e.is_active(MsgId::CabinUtilOff));
        step(&mut e, &mut i, 100_000);
        assert!(e.is_shown(MsgId::CabinUtilOff));
        assert!(e.is_shown(MsgId::IfeSeatsOff));
        i.cabin_util_on = true;
        step(&mut e, &mut i, 1);
        assert!(!e.is_active(MsgId::CabinUtilOff));
    }

    #[test]
    fn main_batt_disch_conditions() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.main_batt_disch_high_us = 19_000_000;
        step(&mut e, &mut i, 2_000_000);
        assert!(!e.is_active(MsgId::MainBattDisch));
        i.main_batt_disch_high_us = 20_000_000;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 1_600_000);
        assert!(e.is_active(MsgId::MainBattDisch));
        i.main_batt_disch_high_us = 0;
        step(&mut e, &mut i, 1);
        assert!(!e.is_active(MsgId::MainBattDisch));
        i.main_batt_disch_low_us = 600_000_000;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 1_600_000);
        assert!(e.is_active(MsgId::MainBattDisch));
        i.main_batt_disch_low_us = 0;
        i.hot_batt_volts = 21.0;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 1_600_000);
        assert!(e.is_active(MsgId::MainBattDisch));
        assert!(e.is_active(MsgId::HotBatteryBus));
    }

    #[test]
    fn hot_battery_bus_hysteresis() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.hot_batt_volts = 21.9;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 1_400_000);
        assert!(!e.is_active(MsgId::HotBatteryBus));
        step(&mut e, &mut i, 200_000);
        assert!(e.is_active(MsgId::HotBatteryBus));
        i.hot_batt_volts = 23.0;
        step(&mut e, &mut i, 1_000_000);
        assert!(e.is_active(MsgId::HotBatteryBus));
        i.hot_batt_volts = 24.5;
        step(&mut e, &mut i, 1);
        assert!(!e.is_active(MsgId::HotBatteryBus));
    }

    #[test]
    fn apu_battery_overheat_needs_one_second() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.apu_batt_overtemp_us = 900_000;
        step(&mut e, &mut i, 1_100_000);
        assert!(!e.is_active(MsgId::ApuBatteryOverheat));
        i.apu_batt_overtemp_us = 1_000_000;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 1_000_000);
        assert!(e.is_active(MsgId::ApuBatteryOverheat));
    }

    #[test]
    fn apu_gen_off_and_running_memo() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.apu_running = true;
        i.apu_gen_switch_on = [false, true];
        i.apb_cmd_closed = [false, true];
        i.apb_closed = [false, false];
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::GenOffApu(0)));
        assert!(e.is_active(MsgId::GenOffApu(1)));
        assert!(e.is_shown(MsgId::ApuRunning));
        i.phase = FlightPhase::TakeoffInhibit;
        step(&mut e, &mut i, 10);
        assert!(e.is_shown(MsgId::ApuRunning));
        assert!(!e.is_shown(MsgId::GenOffApu(0)));
    }

    #[test]
    fn apu_shutdown_and_starters() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.apu_start_failed = true;
        i.apu_start_config_bad = true;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::ApuShutdown));
        assert!(!e.is_active(MsgId::ApuStarter(0)));
        i.apu_start_signal = true;
        step(&mut e, &mut i, 10);
        assert!(e.is_active(MsgId::ApuStarter(0)));
        assert!(e.is_active(MsgId::ApuStarter(1)));
    }

    #[test]
    fn bus_voltage_status_messages() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        step(&mut e, &mut i, 0);
        i.ac115_volts[0] = 40.0;
        i.dc28_volts[1] = 17.0;
        i.backup_volts = 100.0;
        i.flt_inst_capt_volts = 20.0;
        i.flt_inst_fo_volts = 20.0;
        i.battery_refuel_mode = true;
        step(&mut e, &mut i, 1);
        step(&mut e, &mut i, 1_300_000);
        assert!(!e.is_active(MsgId::Dc28Bus(1)));
        step(&mut e, &mut i, 300_000);
        assert!(e.is_active(MsgId::Ac115Bus(0)));
        assert!(!e.is_active(MsgId::Ac115Bus(1)));
        assert!(e.is_active(MsgId::Dc28Bus(1)));
        assert!(e.is_active(MsgId::BackupBus));
        assert!(e.is_active(MsgId::FltInstBusCapt));
        assert!(!e.is_active(MsgId::FltInstBusFo));
    }

    #[test]
    fn ac115_needs_source_online() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.ac_source_online = false;
        i.ac115_volts[0] = 0.0;
        settle(&mut e, &mut i);
        assert!(!e.is_active(MsgId::Ac115Bus(0)));
        i.ac_source_online = true;
        settle(&mut e, &mut i);
        assert!(e.is_active(MsgId::Ac115Bus(0)));
    }

    #[test]
    fn misc_status_conditions() {
        let mut e = EicasElec::new();
        let mut i = healthy();
        i.tru_ac_supplied = [true; 4];
        i.tru_current_a = [30.0, 30.0, 30.0, 5.0];
        i.emp_mc_inop[1] = true;
        i.mc_cac_inop[2] = true;
        i.ac115_bus_isolated[0] = true;
        i.main_ac_bus_isln[1] = true;
        i.standby_sys_fault = true;
        i.oil_cooler_valve_fault[1] = true;
        i.ecd_inop[3] = true;
        i.gcu_inop[0] = true;
        i.protective_trip[2] = true;
        i.spu_main_fail = true;
        i.apu_start_sys_inop = true;
        i.apu_gcu_internal_fault[1] = true;
        i.apu_gcu_comm_fail[1] = true;
        i.engine_starter_inop[2] = true;
        i.both_starters_failed[1] = true;
        i.main_batt_charge_low = true;
        i.dc_bus_isln = true;
        i.apu_batt_charger_fail = true;
        i.main_batt_charger_fail = true;
        i.main_batt_overheat = true;
        i.bpcu_inop[0] = true;
        i.atu_low_current[1] = true;
        i.distr_channel_fault = true;
        i.sys_ctrl_enable_fault = true;
        settle(&mut e, &mut i);
        for id in [
            MsgId::Tru(3),
            MsgId::McHyd(1),
            MsgId::McCac(2),
            MsgId::Ac115BusIsln(0),
            MsgId::MainAcBusIsln(1),
            MsgId::StandbySys,
            MsgId::GenAirOilValve(1),
            MsgId::GenDebrisSensor(3),
            MsgId::GenCtrlUnit(0),
            MsgId::GenSys(2),
            MsgId::ApuStartPwrUnit,
            MsgId::ApuStartSys,
            MsgId::ApuGenCtrlUnit(1),
            MsgId::GenSysApu(1),
            MsgId::EngineStarter(2),
            MsgId::EngStarters(1),
            MsgId::MainBattLow,
            MsgId::DcBusIsln,
            MsgId::ApuBatteryCharger,
            MsgId::MainBatteryCharger,
            MsgId::MainBatteryOverheat,
            MsgId::BusPwrCtrlUnit(0),
            MsgId::Atu(1),
            MsgId::DistrChannel,
            MsgId::SysCtrlEnable,
        ] {
            assert!(e.is_active(id), "{id:?}");
        }
        assert!(!e.is_active(MsgId::Tru(0)));
        assert!(!e.is_active(MsgId::GenSys(0)));
    }

    #[test]
    fn classify_phase_rules() {
        assert_eq!(classify_phase(10.0, 0.0, true, true), FlightPhase::Ground);
        assert_eq!(
            classify_phase(100.0, 0.0, true, true),
            FlightPhase::TakeoffInhibit
        );
        assert_eq!(
            classify_phase(100.0, 0.0, true, false),
            FlightPhase::LandingInhibit
        );
        assert_eq!(
            classify_phase(150.0, 300.0, false, true),
            FlightPhase::TakeoffInhibit
        );
        assert_eq!(classify_phase(150.0, 3000.0, false, true), FlightPhase::Flight);
        assert_eq!(
            classify_phase(150.0, 500.0, false, false),
            FlightPhase::LandingInhibit
        );
    }
}
