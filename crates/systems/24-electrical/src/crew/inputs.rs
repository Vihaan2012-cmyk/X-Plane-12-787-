#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlightPhase {
    #[default]
    Ground,
    TakeoffInhibit,
    Flight,
    LandingInhibit,
}

pub fn classify_phase(
    ground_speed_kt: f64,
    radio_alt_ft: f64,
    on_ground: bool,
    takeoff_config: bool,
) -> FlightPhase {
    if on_ground {
        if ground_speed_kt >= 80.0 {
            if takeoff_config {
                FlightPhase::TakeoffInhibit
            } else {
                FlightPhase::LandingInhibit
            }
        } else {
            FlightPhase::Ground
        }
    } else if takeoff_config && radio_alt_ft < 400.0 {
        FlightPhase::TakeoffInhibit
    } else if !takeoff_config && radio_alt_ft < 800.0 {
        FlightPhase::LandingInhibit
    } else {
        FlightPhase::Flight
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CrewInputs {
    pub now_us: u64,
    pub phase: FlightPhase,
    pub on_ground: bool,
    pub lamp_test: bool,
    pub dim: bool,

    pub engine_running: [bool; 2],
    pub fuel_run: [bool; 2],

    pub battery_switch_on: bool,
    pub apu_gen_switch_on: [bool; 2],
    pub gen_ctrl_on: [bool; 4],
    pub gen_ctrl_cycled: [bool; 4],
    pub cabin_util_on: bool,
    pub ife_on: bool,

    pub gcb_closed: [bool; 4],
    pub gcb_prevented: [bool; 4],
    pub gcb_disagree_us: [u64; 4],
    pub pmg_shorted: [bool; 4],
    pub gcu_28v_fail_us: [u64; 4],
    pub gcu_inop: [bool; 4],
    pub gcu_comm_loss: [bool; 4],
    pub gcu_fan_inop: [bool; 4],
    pub gcu_data_invalid: [bool; 4],
    pub ecd_debris: [bool; 4],
    pub ecd_inop: [bool; 4],
    pub vfsg_lop: [bool; 4],
    pub vfsg_lop_us: [u64; 4],
    pub vfsg_oil_high: [bool; 4],
    pub pmg_lost: [bool; 4],
    pub drive_disconnected: [bool; 4],
    pub protective_trip: [bool; 4],
    pub oil_no_lop_after_shutdown: [bool; 4],
    pub oil_cooler_valve_fault: [bool; 2],
    pub gen_volts: [f64; 4],
    pub gen_hz: [f64; 4],
    pub gen_load_frac: [f64; 4],

    pub ac_bus_energized: [bool; 4],
    pub ac_bus_fault: [bool; 4],
    pub ac_bus_volts: [f64; 4],
    pub ac115_bus_isolated: [bool; 2],
    pub tru_relay_failed_open: [bool; 4],
    pub tru_ac_supplied: [bool; 4],
    pub tru_current_a: [f64; 4],
    pub emp_mc_inop: [bool; 4],

    pub apu_running: bool,
    pub apu_gen_available: [bool; 2],
    pub apb_closed: [bool; 2],
    pub apb_cmd_closed: [bool; 2],
    pub apu_gcu_internal_fault: [bool; 2],
    pub apu_gcu_comm_fail: [bool; 2],
    pub asg_not_commanded_start: [bool; 2],
    pub spu_main_fail: bool,
    pub spu_housekeeping_fail: bool,
    pub apu_start_sys_inop: bool,
    pub apu_start_config_bad: bool,
    pub apu_start_signal: bool,
    pub apu_gen_volts: [f64; 2],
    pub apu_gen_hz: [f64; 2],
    pub apu_gen_load_frac: [f64; 2],

    pub one_starter_inop: [bool; 2],
    pub both_starters_failed: [bool; 2],
    pub mes_unavailable_after_start: [bool; 2],

    pub ext_fwd_inserted: [bool; 2],
    pub ext_fwd_quality_ok: [bool; 2],
    pub ext_fwd_epc_closed: [bool; 2],
    pub ext_aft_inserted: bool,
    pub ext_aft_quality_ok: bool,
    pub ext_aft_epc_closed: bool,
    pub engine_start_in_progress: [bool; 2],
    pub apu_start_in_progress: bool,
    pub apu_batt_current_a: f64,
    pub main_batt_data_invalid: bool,
    pub apu_batt_data_invalid: bool,
    pub load_shed: Vec<String>,
    pub apu_selector_start: bool,
    pub ext_trip: bool,
    pub ground_handling_mode: bool,
    pub ground_service_on: bool,

    pub main_batt_volts: f64,
    pub main_batt_current_a: f64,
    pub main_batt_charge_low: bool,
    pub main_batt_failed: bool,
    pub hot_batt_bus_energized: bool,
    pub apu_batt_volts: f64,
    pub apu_batt_failed: bool,
    pub apu_batt_overtemp_us: u64,
    pub standby_sys_fault: bool,
    pub rat_deployed: bool,
    pub main_batt_disch_high_us: u64,
    pub main_batt_disch_low_us: u64,
    pub main_batt_disch_msg: bool,
    pub hot_batt_volts: f64,
    pub apu_start_failed: bool,
    pub main_ac_bus_isln: [bool; 2],
    pub ac_source_online: bool,
    pub ac115_volts: [f64; 2],
    pub dc28_volts: [f64; 2],
    pub backup_volts: f64,
    pub flt_inst_capt_volts: f64,
    pub flt_inst_fo_volts: f64,
    pub battery_refuel_mode: bool,
    pub dc_bus_isln: bool,
    pub apb_disagree_us: [u64; 2],
    pub asg_pmg_shorted: [bool; 2],
    pub apb_trip: [bool; 2],
    pub apu_gen_cycled: [bool; 2],
    pub apu_batt_charger_fail: bool,
    pub apu_starter_inop: [bool; 2],
    pub engine_starter_inop: [bool; 4],
    pub main_batt_charger_fail: bool,
    pub main_batt_overheat: bool,
    pub bpcu_inop: [bool; 2],
    pub atu_low_current: [bool; 2],
    pub mc_cac_inop: [bool; 4],
    pub distr_channel_fault: bool,
    pub sys_ctrl_enable_fault: bool,
}
