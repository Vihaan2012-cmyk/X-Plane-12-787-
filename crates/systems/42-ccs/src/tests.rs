use crate::cabinet::{FULL_PBIT_MAX_US, FULL_PBIT_MIN_US, INHIBITED_HOSTED_MAX_US, INHIBITED_HOSTED_MIN_US};
use crate::ccs::{Ccs, CcsInputs, CcsValue};
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/42-ccs")
}

fn ccs() -> Ccs {
    Ccs::new(&data_dir(), 7).expect("ccs data loads")
}

fn buses(pairs: &[(&str, f64)]) -> CcsInputs {
    let mut i = CcsInputs::default();
    for (b, v) in pairs {
        i.bus_v.insert(b.to_string(), *v);
    }
    i
}

fn battery_only() -> CcsInputs {
    buses(&[("CIB", 27.0), ("FIB", 27.0), ("HBB_MAIN", 27.5)])
}

fn all_power() -> CcsInputs {
    buses(&[
        ("CIB", 28.0),
        ("FIB", 28.0),
        ("DC28_L", 28.0),
        ("DC28_R", 28.0),
        ("HBB_MAIN", 28.0),
    ])
}

fn run(c: &mut Ccs, seconds: f64, i: &CcsInputs) {
    for _ in 0..(seconds * 10.0) as u64 {
        c.step(100_000, i);
    }
}

fn text(c: &Ccs, s: &str) -> String {
    match c.value(s) {
        Some(CcsValue::Text(t)) => t,
        other => panic!("{s}: {other:?}"),
    }
}

fn flag(c: &Ccs, s: &str) -> bool {
    match c.value(s) {
        Some(CcsValue::Bool(b)) => b,
        other => panic!("{s}: {other:?}"),
    }
}

fn num(c: &Ccs, s: &str) -> f64 {
    match c.value(s) {
        Some(CcsValue::Num(x)) => x,
        other => panic!("{s}: {other:?}"),
    }
}

#[test]
fn battery_powers_only_the_left_cabinet_and_it_takes_two_to_three_minutes() {
    let mut c = ccs();
    let i = battery_only();
    run(&mut c, FULL_PBIT_MIN_US as f64 / 1e6 - 1.0, &i);
    assert_eq!(text(&c, "ccs.ccr.l.state"), "booting");
    assert_eq!(text(&c, "ccs.ccr.l.mode"), "uninhibited");
    assert!(!flag(&c, "ccs.ccr.l.displays_ready"));
    assert_eq!(text(&c, "ccs.ccr.r.state"), "off");
    run(
        &mut c,
        (FULL_PBIT_MAX_US - FULL_PBIT_MIN_US) as f64 / 1e6 + 2.0,
        &i,
    );
    assert_eq!(text(&c, "ccs.ccr.l.state"), "operational");
    assert!(flag(&c, "ccs.ccr.l.startup_complete"));
    assert_eq!(text(&c, "ccs.ccr.r.state"), "off");
}

#[test]
fn right_cabinet_starts_when_main_dc_is_powered() {
    let mut c = ccs();
    run(&mut c, 200.0, &battery_only());
    run(&mut c, 1.0, &all_power());
    assert_eq!(text(&c, "ccs.ccr.r.state"), "booting");
    run(&mut c, 185.0, &all_power());
    assert_eq!(text(&c, "ccs.ccr.r.state"), "operational");
    assert!(flag(&c, "ccs.gpm.r6.running"));
}

#[test]
fn hot_battery_holds_up_eleven_seconds_then_saves_and_shuts_down() {
    let mut c = ccs();
    run(&mut c, 200.0, &battery_only());
    let dead = buses(&[("HBB_MAIN", 27.0)]);
    run(&mut c, 5.0, &dead);
    assert!(flag(&c, "ccs.ccr.l.on_battery"));
    assert_eq!(text(&c, "ccs.ccr.l.state"), "operational");
    let w: f64 = c
        .loads_w()
        .iter()
        .filter(|(n, _)| *n == "CABINET CCR L BAT")
        .map(|(_, w)| *w)
        .sum();
    assert!(w > 0.0);
    run(&mut c, 1.0, &battery_only());
    assert_eq!(num(&c, "ccs.ccr.l.boots"), 1.0);
    run(&mut c, 12.0, &dead);
    assert_eq!(text(&c, "ccs.ccr.l.state"), "off");
    assert_eq!(num(&c, "ccs.ccr.l.nvm_saves"), 1.0);
    run(&mut c, 1.0, &battery_only());
    assert_eq!(num(&c, "ccs.ccr.l.boots"), 2.0);
}

#[test]
fn one_pcm_input_lost_keeps_the_cabinet_up() {
    let mut c = ccs();
    run(&mut c, 200.0, &battery_only());
    run(&mut c, 30.0, &buses(&[("FIB", 27.0), ("HBB_MAIN", 27.5)]));
    assert_eq!(text(&c, "ccs.ccr.l.state"), "operational");
    assert!(!flag(&c, "ccs.ccr.l.on_battery"));
}

#[test]
fn reset_in_the_air_is_an_inhibited_restart_without_nvm_save() {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    let mut air = all_power();
    air.on_ground = false;
    c.press_reset(0);
    run(&mut c, 1.0, &air);
    assert_eq!(num(&c, "ccs.ccr.l.nvm_saves"), 0.0);
    assert_eq!(text(&c, "ccs.ccr.l.mode"), "inhibited");
    run(&mut c, INHIBITED_HOSTED_MIN_US as f64 / 1e6 - 2.0, &air);
    assert!(!flag(&c, "ccs.ccr.l.hosted_ready"));
    run(
        &mut c,
        (INHIBITED_HOSTED_MAX_US - INHIBITED_HOSTED_MIN_US) as f64 / 1e6 + 8.0,
        &air,
    );
    assert!(flag(&c, "ccs.ccr.l.hosted_ready"));
    assert!(flag(&c, "ccs.ccr.l.displays_ready"));
    assert_eq!(text(&c, "ccs.ccr.r.state"), "operational");
}

#[test]
fn losing_a_gpm_drops_exactly_its_single_instance_apps() {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    assert!(flag(&c, "ccs.app.nitrogen_generation_system.available"));
    c.set_gpm_failed("R4", true);
    run(&mut c, 0.1, &all_power());
    assert!(!flag(&c, "ccs.app.nitrogen_generation_system.available"));
    assert!(flag(&c, "ccs.app.hydraulic_interface_function.available"));
    c.set_gpm_failed("L4", true);
    c.set_gpm_failed("R6", true);
    run(&mut c, 0.1, &all_power());
    assert!(!flag(&c, "ccs.app.low_pressure_system.available"));
    assert!(!flag(
        &c,
        "ccs.app.cabin_air_temperature_control_system.available"
    ));
}

#[test]
fn low_cooling_supply_runs_the_fan_and_closes_the_duct_valve() {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    let mut i = all_power();
    i.cooling_supply_ok = false;
    run(&mut c, 1.0, &i);
    assert!(flag(&c, "ccs.ccr.l.fan_on"));
    assert!(flag(&c, "ccs.ccr.l.valve_closed"));
}

#[test]
fn ground_overtemp_with_a_dead_fan_shuts_the_cabinet_down() {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    c.cabinets[0].fan_failed = true;
    let mut i = all_power();
    i.cooling_supply_ok = false;
    let mut went_off = false;
    for _ in 0..20_000 {
        c.step(5_000_000, &i);
        if text(&c, "ccs.ccr.l.state") == "off" {
            went_off = true;
            break;
        }
    }
    assert!(went_off);
    assert!(c.cabinets[0].temp_c >= crate::cabinet::APPROX_GROUND_SHUTDOWN_C - 1.0);
    assert!(flag(&c, "ccs.status.ccr_cabinet_cooling_l"));
}

#[test]
fn rdcs_run_from_either_supply_and_boot_inside_45_s() {
    let mut c = ccs();
    assert_eq!(c.rdcs.len(), 21);
    run(&mut c, 44.0, &all_power());
    assert!(flag(&c, "ccs.rdc.11.running"));
    let sec_only = buses(&[("DC28_L", 28.0), ("DC28_R", 28.0), ("HBB_MAIN", 28.0)]);
    run(&mut c, 1.0, &sec_only);
    assert!(flag(&c, "ccs.rdc.11.on_secondary"));
    assert!(flag(&c, "ccs.rdc.11.running"));
    c.set_rdc_failed("RDC 19", true);
    run(&mut c, 0.1, &all_power());
    assert!(flag(&c, "ccs.status.ccs_rdc"));
}

#[test]
fn steady_network_runs_on_the_fast_path_and_keeps_data_fresh() {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    assert!(c.cdn.fast_steps > 100);
    assert!(flag(&c, "ccs.data.rdc_11_data.gpm_l1.valid"));
    assert!(num(&c, "ccs.data.rdc_11_data.gpm_l1.age_ms") <= 32.0);
    assert!(flag(&c, "ccs.data.bpcu_p300.gpm_l2.valid"));
    assert!(flag(&c, "ccs.data.time_l1.rdc_19.valid"));
}

#[test]
fn losing_both_channels_to_an_rdc_makes_its_data_stale() {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    assert!(c.set_device_fault("ARS 5", "failed", true));
    run(&mut c, 2.0, &all_power());
    assert!(flag(&c, "ccs.data.rdc_11_data.gpm_r1.valid"));
    assert!(c.set_device_fault("ARS 6", "failed", true));
    run(&mut c, 2.0, &all_power());
    assert!(!flag(&c, "ccs.data.rdc_11_data.gpm_r1.valid"));
    assert!(flag(&c, "ccs.data.rdc_9_data.gpm_r1.valid"));
}

#[test]
fn after_51_days_replayed_stale_data_is_shown_as_valid() {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    for s in ["ARS 5", "ARS 6"] {
        c.set_device_fault(s, "replay", true);
    }
    run(&mut c, 3.0, &all_power());
    assert!(!flag(&c, "ccs.data.rdc_11_data.gpm_l1.valid"));
    c.set_powered_days(51.5);
    run(&mut c, 0.5, &all_power());
    assert!(flag(&c, "ccs.monitor_lost"));
    assert!(flag(&c, "ccs.data.rdc_11_data.gpm_l1.valid"));
    assert!(num(&c, "ccs.data.rdc_11_data.gpm_l1.age_ms") > 3000.0);
}

#[test]
fn power_cycle_resets_the_51_day_counter() {
    let mut c = ccs();
    run(&mut c, 10.0, &all_power());
    c.set_powered_days(50.0);
    run(&mut c, 20.0, &buses(&[]));
    assert_eq!(num(&c, "ccs.powered_days"), 0.0);
}

fn ready() -> Ccs {
    let mut c = ccs();
    run(&mut c, 400.0, &all_power());
    c
}

#[test]
fn every_gpm_schedule_fits_its_major_frame() {
    let c = ccs();
    for sw in &c.sw {
        assert!(sw.schedule_fits(), "{}", sw.name);
        assert!(sw.partitions.iter().all(|p| p.window_us > 0), "{}", sw.name);
        assert_eq!(sw.partitions.iter().filter(|p| !p.hosted).count(), 6);
    }
}

#[test]
fn a_crashing_partition_restarts_then_is_held_by_the_health_monitor() {
    let mut c = ready();
    let app = "NITROGEN GENERATION SYSTEM";
    c.part_faults.insert(("R4".into(), app.into()));
    run(&mut c, 0.1, &all_power());
    assert!(!c.instance_live(app, "R4"));
    c.part_faults.clear();
    run(&mut c, 2.5, &all_power());
    assert!(c.instance_live(app, "R4"));
    c.part_faults.insert(("R4".into(), app.into()));
    run(&mut c, 10.0, &all_power());
    c.part_faults.clear();
    run(&mut c, 10.0, &all_power());
    assert!(!c.instance_live(app, "R4"));
    assert!(!c.app_available(app));
    assert!(c.app_available("HYDRAULIC INTERFACE FUNCTION"));
}

#[test]
fn one_lane_keeps_the_gpm_two_lanes_stop_it_and_ccshm_reports_it() {
    let mut c = ready();
    c.sw.iter_mut().find(|s| s.name == "L4").unwrap().lane_failed[0] = true;
    run(&mut c, 0.1, &all_power());
    assert!(c.gpm_running("L4"));
    assert!(flag(&c, "ccs.gpm.l4.single_lane"));
    c.sw.iter_mut().find(|s| s.name == "L4").unwrap().lane_failed[1] = true;
    run(&mut c, 0.1, &all_power());
    assert!(!c.gpm_running("L4"));
    assert!(flag(&c, "ccs.crew_status.ccs_gpm_l4"));
    assert_eq!(text(&c, "ccs.maint.page2.gpm_4"), "SEE CMCF");
}

#[test]
fn losing_every_ccshm_instance_raises_ccs_health_management_after_a_delay() {
    let mut c = ready();
    for g in crate::network::GPMS {
        c.part_faults.insert((g.into(), "CCSHM".into()));
    }
    run(&mut c, 15.0, &all_power());
    assert!(!flag(&c, "ccs.crew_status.ccs_health_management"));
    run(&mut c, 30.0, &all_power());
    assert!(flag(&c, "ccs.crew_status.ccs_health_management"));
}

#[test]
fn electrical_alerts_reach_crew_alerting_over_the_cdn() {
    let mut c = ccs();
    c.set_elec_icd(&["ELEC AC BUS L1".to_string(), "ELEC GEN OFF L1".to_string()]);
    let payload = c.encode_elec(&["ELEC GEN OFF L1".to_string()]).unwrap();
    c.bpcu_payload = [Some(payload.clone()), Some(payload)];
    run(&mut c, 100.0, &all_power());
    assert!(!flag(&c, "ccs.cas.available"));
    run(&mut c, 300.0, &all_power());
    assert!(flag(&c, "ccs.cas.available"));
    assert!(flag(&c, "ccs.cas.elec_data_valid"));
    assert!(flag(&c, "ccs.cas.msg.elec_gen_off_l1"));
    assert!(!flag(&c, "ccs.cas.msg.elec_ac_bus_l1"));
    c.set_device_fault("P300", "failed", true);
    run(&mut c, 1.0, &all_power());
    assert!(flag(&c, "ccs.cas.elec_data_valid"));
    c.set_device_fault("P400", "failed", true);
    run(&mut c, 1.0, &all_power());
    assert!(!flag(&c, "ccs.cas.elec_data_valid"));
    assert_eq!(num(&c, "ccs.cas.count"), 0.0);
}

#[test]
fn wrong_icd_version_on_the_bpcu_is_rejected() {
    let mut c = ccs();
    c.set_elec_icd(&["ELEC GEN OFF L1".to_string()]);
    let mut payload = c.encode_elec(&["ELEC GEN OFF L1".to_string()]).unwrap();
    payload[1] = 9;
    c.bpcu_payload = [Some(payload.clone()), Some(payload)];
    run(&mut c, 400.0, &all_power());
    assert!(!flag(&c, "ccs.cas.elec_data_valid"));
}

#[test]
fn data_load_mode_runs_only_the_data_load_partition() {
    let mut c = ready();
    c.sw.iter_mut().find(|s| s.name == "R4").unwrap().data_load = true;
    run(&mut c, 0.1, &all_power());
    assert!(!flag(&c, "ccs.gpm.r4.data_load"));
    c.gnd_test_sel = 2;
    c.sw.iter_mut().find(|s| s.name == "R4").unwrap().data_load = true;
    run(&mut c, 0.1, &all_power());
    assert!(!c.instance_live("NITROGEN GENERATION SYSTEM", "R4"));
    assert!(c.instance_live("DATA LOAD", "R4"));
    assert!(!c.instance_live("DATA LOAD", "R5"));
}

#[test]
fn maintenance_pages_show_switch_rdc_and_terminal_faults() {
    let mut c = ready();
    assert_eq!(text(&c, "ccs.maint.page1.ccs_switches"), "NORMAL");
    c.set_device_fault("ACS L A", "failed", true);
    c.set_rdc_failed("RDC 7", true);
    run(&mut c, 1.0, &all_power());
    assert_eq!(text(&c, "ccs.maint.page1.ccs_switches"), "FAULT");
    assert_eq!(text(&c, "ccs.maint.page2.acs_a"), "FAULT");
    assert_eq!(text(&c, "ccs.maint.page7.rdc_7"), "FAULT");
    assert_eq!(text(&c, "ccs.maint.page1.ccs_rdc_s"), "FAULT");
    assert_eq!(text(&c, "ccs.maint.page8.ars_3"), "NORMAL");
    assert!(flag(&c, "ccs.crew_status.ccs_acs_l_a"));
}

#[test]
fn leaving_ground_test_takes_two_and_a_half_seconds() {
    let mut c = ready();
    c.gnd_test_sel = 1;
    run(&mut c, 0.2, &all_power());
    assert_eq!(text(&c, "ccs.gnd_test"), "ENABLE");
    c.gnd_test_sel = 0;
    run(&mut c, 2.0, &all_power());
    assert_eq!(text(&c, "ccs.gnd_test"), "ENABLE");
    run(&mut c, 0.7, &all_power());
    assert_eq!(text(&c, "ccs.gnd_test"), "NORM");
}
