use super::{Electrical, ElectricalInputs, Faults, Switchgear, new_cbic, plant::Plant, receptacle};
use crate::control::{BpcuInput, BpcuPair, LmInput, LoadManager};
use crate::crew::{CrewInputs, EicasElec, OverheadPanel};
use crate::protection::contactor::{Contactor, ContactorConfig, ContactorOutput};
use crate::sources::{ExtPowerMonitor, Gcu, GcuParams, MachineParams};
use crate::storage::{BatteryCharger, BatteryMonitoringUnit, ChargerMode, ChargerOutput, ChargerParams};

fn parse_contactors(data_dir: &std::path::Path) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(data_dir.join("contactors.csv")).unwrap_or_default();
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 4 || f[1].is_empty() || f[1].starts_with('"') {
            continue;
        }
        out.push((f[1].to_string(), f[2].to_string()));
    }
    out
}

fn charger_idle() -> ChargerOutput {
    ChargerOutput {
        mode: ChargerMode::Off,
        charge_current_a: 0.0,
        output_v: 0.0,
        input_power_w: 0.0,
        failed: false,
    }
}

pub(super) fn build(data_dir: &std::path::Path) -> Result<Electrical, String> {
    let plant = Plant::new(data_dir)?;
    let pdu =
        super::pdu_link::PduLink::new(data_dir, plant.net().sspc_ids(), &plant.net().sspc_load_names())?;
    let mut cbic = new_cbic();
    pdu.register(&mut cbic.app);
    let list = parse_contactors(data_dir);
    if list.is_empty() {
        return Err("contactors.csv missing or empty".to_string());
    }
    let idle = ContactorOutput {
        contacts_closed: false,
        armature_closed: false,
        aux_no: false,
        aux_nc: true,
        resistance_ohm: 0.0,
        welded: false,
        bouncing: false,
    };
    let gear: Vec<Switchgear> = list
        .into_iter()
        .map(|(name, upstream)| Switchgear {
            name,
            upstream,
            unit: Contactor::new(ContactorConfig::default()),
            cmd: false,
            out: idle,
            forced_open: false,
        })
        .collect();
    let mut panel = OverheadPanel::new();
    panel.set_normal();
    panel.battery.set_latched(false);
    let gcus = (0..6)
        .map(|m| {
            Gcu::new(if m < 4 {
                GcuParams::vfsg()
            } else {
                GcuParams::asg()
            })
        })
        .collect();
    let machine_params = (0..6)
        .map(|m| {
            if m < 4 {
                MachineParams::vfsg()
            } else {
                MachineParams::asg()
            }
        })
        .collect();
    let crew_in = CrewInputs::default();
    let (panel_out, lamps) = panel.update(&crew_in);
    let bpcu_idle = BpcuPair::new().step(&BpcuInput::cold_and_dark(0));
    let lm_idle = LoadManager::new().step(&LmInput {
        now_us: 0,
        on_ground: true,
        available_kw: 0.0,
        demand_on: [false; 14],
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
    });
    let thermal = Some(super::thermal_link::ThermalLink::build(
        data_dir,
        &gear,
        plant.net(),
    ));
    Ok(Electrical {
        parallel_blocks: Vec::new(),
        tru_oc: [crate::protection::tru_oc::TruOc::default(); 4],
        exciter_feedback_a: [0.0; 6],
        atru_prot: Default::default(),
        epc_sup: Default::default(),
        pecs_c: [f64::NAN; 2],
        managed_w: [[0.0; 4]; 14],
        circuit_prot: super::breaker_link::build(plant.net(), &pdu),
        batt_mon: Default::default(),
        batt_mon_out: Default::default(),
        batt_switch_prev: false,
        tie_diff: [crate::protection::zone_diff::ZoneDiff::default(); 2],
        atu_diff: [crate::protection::zone_diff::ZoneDiff::default(); 2],
        plant,
        now_us: 0,
        panel,
        lamps,
        panel_out,
        eicas: EicasElec::new(),
        synoptic: crate::crew::build_synoptic(&crew_in, [true; 4], [true; 4]),
        crew_in,
        bpcu: BpcuPair::new(),
        bpcu_out: bpcu_idle,
        rat: super::rat::RatUnit::new(),
        gcus,
        gcu_out: vec![Default::default(); 6],
        ext_mon: (0..3).map(|k| ExtPowerMonitor::new(receptacle(k))).collect(),
        ext_out: (0..3)
            .map(|k| ExtPowerMonitor::new(receptacle(k)).step(&Default::default()))
            .collect(),
        ext_switch_on: [false; 3],
        gear,
        bmu: [BatteryMonitoringUnit::new(), BatteryMonitoringUnit::new()],
        bmu_out: [Default::default(); 2],
        chargers: [
            BatteryCharger::new(ChargerParams::default()),
            BatteryCharger::new(ChargerParams::default()),
        ],
        charger_out: [charger_idle(); 2],
        diode: Default::default(),
        lm: LoadManager::new(),
        lm_out: lm_idle,
        cbic,
        inputs: ElectricalInputs::default(),
        faults: Faults::default(),
        apu_batt_switch_on: false,
        ground_service_switch: false,
        apu_start_latch_us: None,
        drive_disconnected: [false; 4],
        drive_sol: Default::default(),
        machine_params,
        last_dt_us: 0,
        pace: super::stepping::Pace::new(),
        rig: super::rig::Rig::new(),
        fault_link: super::fault_link::FaultLink::new(),
        thermal,
        pdu,
    })
}
