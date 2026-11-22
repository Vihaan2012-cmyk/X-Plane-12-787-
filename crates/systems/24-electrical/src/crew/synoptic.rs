use super::inputs::CrewInputs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementState {
    On,
    Available,
    Off,
    Failed,
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colour {
    Green,
    White,
    WhiteLow,
    Amber,
    AmberLow,
    Cyan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveState {
    Connected,
    NotRunning,
    Fault,
    Disconnected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryLabel {
    Chg,
    Disch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SynSource {
    pub state: ElementState,
    pub colour: Colour,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SynGen {
    pub source: SynSource,
    pub drive: DriveState,
    pub drive_colour: Colour,
    pub drive_text: bool,
    pub starter_on: bool,
    pub starters_failed_text: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SynBus {
    pub powered: Option<bool>,
    pub colour: Colour,
    pub fault_message_expected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SynBattery {
    pub volts: Option<i32>,
    pub amps: Option<i32>,
    pub label: Option<BatteryLabel>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LargeMotorPower {
    pub l_start: bool,
    pub r_start: bool,
    pub apu_start: bool,
    pub aft_feed_flowing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlowLine {
    pub from: &'static str,
    pub to: &'static str,
    pub flowing: bool,
    pub colour: Colour,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SynopticPage {
    pub gens: [SynGen; 4],
    pub apu_gens: [SynSource; 2],
    pub ext_fwd: [SynSource; 2],
    pub ext_aft: SynSource,
    pub buses: [SynBus; 4],
    pub large_motor: LargeMotorPower,
    pub main_battery: SynBattery,
    pub apu_battery: SynBattery,
    pub lines: Vec<FlowLine>,
    pub load_shed: Vec<String>,
}

fn colour_of(s: ElementState) -> Colour {
    match s {
        ElementState::On | ElementState::Available => Colour::Green,
        ElementState::Off => Colour::White,
        ElementState::Failed => Colour::Amber,
        ElementState::Invalid => Colour::WhiteLow,
    }
}

fn src(state: ElementState) -> SynSource {
    SynSource {
        state,
        colour: colour_of(state),
    }
}

fn ext_state(inserted: bool, quality: bool, closed: bool) -> ElementState {
    if closed {
        ElementState::On
    } else if inserted && quality {
        ElementState::Available
    } else {
        ElementState::Off
    }
}

fn battery(volts: f64, amps: f64, valid: bool) -> SynBattery {
    if !valid {
        return SynBattery {
            volts: None,
            amps: None,
            label: None,
        };
    }
    let label = if amps < -0.5 {
        Some(BatteryLabel::Disch)
    } else if amps > 0.5 {
        Some(BatteryLabel::Chg)
    } else {
        None
    };
    SynBattery {
        volts: Some(volts.round() as i32),
        amps: Some(amps.abs().round() as i32),
        label,
    }
}

fn line(from: &'static str, to: &'static str, flowing: bool) -> FlowLine {
    FlowLine {
        from,
        to,
        flowing,
        colour: if flowing { Colour::Green } else { Colour::White },
    }
}

const GEN_LABEL: [&str; 4] = ["GEN L1", "GEN L2", "GEN R1", "GEN R2"];
const BUS_LABEL: [&str; 4] = ["BUS L1", "BUS L2", "BUS R1", "BUS R2"];
const APU_LABEL: [&str; 2] = ["APU GEN L", "APU GEN R"];
const FWD_LABEL: [&str; 2] = ["FWD EXT PWR L", "FWD EXT PWR R"];

pub fn build_synoptic(i: &CrewInputs, gen_valid: [bool; 4], bus_valid: [bool; 4]) -> SynopticPage {
    let mut gens = [SynGen {
        source: src(ElementState::Off),
        drive: DriveState::NotRunning,
        drive_colour: Colour::White,
        drive_text: false,
        starter_on: false,
        starters_failed_text: false,
    }; 4];
    for k in 0..4 {
        let eng = k / 2;
        let running = i.engine_running[eng];
        let state = if !gen_valid[k] {
            ElementState::Invalid
        } else if running && i.gen_ctrl_on[k] && i.gcb_closed[k] {
            ElementState::On
        } else if running && i.gen_ctrl_on[k] && !i.gcb_closed[k] && !i.gcb_prevented[k] {
            ElementState::Failed
        } else {
            ElementState::Off
        };
        let fault = running && (i.vfsg_lop[k] || i.pmg_lost[k]);
        let (drive, dc, text) = if i.drive_disconnected[k] {
            (DriveState::Disconnected, Colour::AmberLow, true)
        } else if fault {
            (DriveState::Fault, Colour::Amber, true)
        } else if running {
            (DriveState::Connected, Colour::Green, false)
        } else {
            (DriveState::NotRunning, Colour::White, false)
        };
        gens[k] = SynGen {
            source: src(state),
            drive,
            drive_colour: dc,
            drive_text: text,
            starter_on: i.engine_start_in_progress[eng],
            starters_failed_text: i.both_starters_failed[eng],
        };
    }

    let mut apu_gens = [src(ElementState::Off); 2];
    let mut ext_fwd = [src(ElementState::Off); 2];
    for s in 0..2 {
        let st = if i.apu_running && i.apu_gen_switch_on[s] && i.apb_cmd_closed[s] && !i.apb_closed[s] {
            ElementState::Failed
        } else if i.apb_closed[s] {
            ElementState::On
        } else if i.apu_running && i.apu_gen_switch_on[s] {
            ElementState::Available
        } else {
            ElementState::Off
        };
        apu_gens[s] = src(st);
        ext_fwd[s] = src(ext_state(
            i.ext_fwd_inserted[s],
            i.ext_fwd_quality_ok[s],
            i.ext_fwd_epc_closed[s],
        ));
    }
    let ext_aft = src(ext_state(
        i.ext_aft_inserted,
        i.ext_aft_quality_ok,
        i.ext_aft_epc_closed,
    ));

    let mut buses = [SynBus {
        powered: Some(false),
        colour: Colour::Amber,
        fault_message_expected: false,
    }; 4];
    for k in 0..4 {
        buses[k] = if !bus_valid[k] {
            SynBus {
                powered: None,
                colour: Colour::WhiteLow,
                fault_message_expected: false,
            }
        } else if i.ac_bus_energized[k] {
            SynBus {
                powered: Some(true),
                colour: Colour::Green,
                fault_message_expected: false,
            }
        } else {
            SynBus {
                powered: Some(false),
                colour: Colour::Amber,
                fault_message_expected: i.ac_bus_fault[k],
            }
        };
    }

    let large_motor = LargeMotorPower {
        l_start: i.engine_start_in_progress[0],
        r_start: i.engine_start_in_progress[1],
        apu_start: i.apu_start_in_progress,
        aft_feed_flowing: ext_aft.state == ElementState::On,
    };

    let mut lines = Vec::new();
    for k in 0..4 {
        lines.push(line(
            GEN_LABEL[k],
            BUS_LABEL[k],
            gens[k].source.state == ElementState::On && i.ac_bus_energized[k],
        ));
    }
    for s in 0..2 {
        lines.push(line(
            APU_LABEL[s],
            BUS_LABEL[s * 2],
            apu_gens[s].state == ElementState::On && i.ac_bus_energized[s * 2],
        ));
        lines.push(line(
            FWD_LABEL[s],
            BUS_LABEL[s * 2 + 1],
            ext_fwd[s].state == ElementState::On && i.ac_bus_energized[s * 2 + 1],
        ));
    }
    lines.push(line(
        "AFT EXT PWR",
        "LARGE MOTOR POWER",
        large_motor.aft_feed_flowing,
    ));

    SynopticPage {
        gens,
        apu_gens,
        ext_fwd,
        ext_aft,
        buses,
        large_motor,
        main_battery: battery(
            i.main_batt_volts,
            i.main_batt_current_a,
            !i.main_batt_data_invalid,
        ),
        apu_battery: battery(i.apu_batt_volts, i.apu_batt_current_a, !i.apu_batt_data_invalid),
        lines,
        load_shed: i.load_shed.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cruise() -> CrewInputs {
        CrewInputs {
            engine_running: [true; 2],
            gen_ctrl_on: [true; 4],
            gcb_closed: [true; 4],
            ac_bus_energized: [true; 4],
            main_batt_volts: 28.4,
            main_batt_current_a: 2.6,
            apu_batt_volts: 27.6,
            ..CrewInputs::default()
        }
    }

    #[test]
    fn all_gens_on_green() {
        let p = build_synoptic(&cruise(), [true; 4], [true; 4]);
        for k in 0..4 {
            assert_eq!(p.gens[k].source.state, ElementState::On);
            assert_eq!(p.gens[k].source.colour, Colour::Green);
            assert_eq!(p.gens[k].drive, DriveState::Connected);
            assert_eq!(p.buses[k].colour, Colour::Green);
        }
        assert!(
            p.lines
                .iter()
                .take(4)
                .all(|l| l.flowing && l.colour == Colour::Green)
        );
        assert_eq!(p.main_battery.volts, Some(28));
        assert_eq!(p.main_battery.label, Some(BatteryLabel::Chg));
    }

    #[test]
    fn gen_failed_amber_and_bus_amber() {
        let mut i = cruise();
        i.gcb_closed[0] = false;
        i.ac_bus_energized[0] = false;
        i.ac_bus_fault[0] = true;
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.gens[0].source.state, ElementState::Failed);
        assert_eq!(p.gens[0].source.colour, Colour::Amber);
        assert_eq!(p.buses[0].colour, Colour::Amber);
        assert!(p.buses[0].fault_message_expected);
        assert!(!p.lines[0].flowing);
        assert_eq!(p.lines[0].colour, Colour::White);
    }

    #[test]
    fn gen_ctrl_off_running_is_white() {
        let mut i = cruise();
        i.gen_ctrl_on[1] = false;
        i.gcb_closed[1] = false;
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.gens[1].source.state, ElementState::Off);
        assert_eq!(p.gens[1].source.colour, Colour::White);
    }

    #[test]
    fn drive_states() {
        let mut i = cruise();
        i.vfsg_lop[2] = true;
        i.drive_disconnected[3] = true;
        i.engine_running[0] = false;
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.gens[2].drive, DriveState::Fault);
        assert!(p.gens[2].drive_text);
        assert_eq!(p.gens[3].drive, DriveState::Disconnected);
        assert_eq!(p.gens[3].drive_colour, Colour::AmberLow);
        assert_eq!(p.gens[0].drive, DriveState::NotRunning);
        assert_eq!(p.gens[0].drive_colour, Colour::White);
    }

    #[test]
    fn invalid_data_low_intensity() {
        let p = build_synoptic(&cruise(), [false, true, true, true], [true, true, false, true]);
        assert_eq!(p.gens[0].source.colour, Colour::WhiteLow);
        assert_eq!(p.buses[2].colour, Colour::WhiteLow);
        assert_eq!(p.buses[2].powered, None);
    }

    #[test]
    fn ground_ext_power_and_apu() {
        let mut i = CrewInputs {
            ext_fwd_inserted: [true, false],
            ext_fwd_quality_ok: [true, false],
            ac_bus_energized: [false, true, false, false],
            ..CrewInputs::default()
        };
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.ext_fwd[0].state, ElementState::Available);
        assert_eq!(p.ext_fwd[1].state, ElementState::Off);
        assert_eq!(p.ext_fwd[1].colour, Colour::White);
        i.ext_fwd_epc_closed[0] = true;
        i.ac_bus_energized = [true; 4];
        i.apu_running = true;
        i.apu_gen_switch_on = [true; 2];
        i.apb_closed = [false, true];
        i.apb_cmd_closed = [false, true];
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.ext_fwd[0].state, ElementState::On);
        assert_eq!(p.apu_gens[0].state, ElementState::Available);
        assert_eq!(p.apu_gens[1].state, ElementState::On);
        assert!(p.lines.iter().any(|l| l.from == "FWD EXT PWR L" && l.flowing));
    }

    #[test]
    fn apu_gen_failed_and_start_arrows() {
        let mut i = cruise();
        i.apu_running = true;
        i.apu_gen_switch_on = [true; 2];
        i.apb_cmd_closed = [true, false];
        i.engine_start_in_progress = [false, true];
        i.apu_start_in_progress = true;
        i.ext_aft_inserted = true;
        i.ext_aft_quality_ok = true;
        i.ext_aft_epc_closed = true;
        i.both_starters_failed = [true, false];
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.apu_gens[0].state, ElementState::Failed);
        assert_eq!(p.apu_gens[0].colour, Colour::Amber);
        assert!(p.large_motor.r_start && !p.large_motor.l_start && p.large_motor.apu_start);
        assert!(p.large_motor.aft_feed_flowing);
        assert!(p.gens[0].starters_failed_text);
        assert!(p.gens[3].starter_on);
    }

    #[test]
    fn battery_discharge_and_invalid() {
        let mut i = cruise();
        i.apu_batt_data_invalid = true;
        i.main_batt_current_a = -12.4;
        i.main_batt_volts = 25.6;
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.main_battery.amps, Some(12));
        assert_eq!(p.main_battery.volts, Some(26));
        assert_eq!(p.main_battery.label, Some(BatteryLabel::Disch));
        assert_eq!(p.apu_battery.volts, None);
        i.load_shed = vec!["CABIN LOADS".to_string()];
        let p = build_synoptic(&i, [true; 4], [true; 4]);
        assert_eq!(p.load_shed.len(), 1);
    }
}
