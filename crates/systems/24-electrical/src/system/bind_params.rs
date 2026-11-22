mod rated;

use super::Electrical;
use super::SignalValue;
use crate::control::bpcu::{
    BTB_COUNT, BUS_COUNT, CIT_GROUND_TO_AIR_US, EPC_BUS, EXT_CONTACTOR_HOLD_US, EXT_KVA,
    INSTR_PARALLEL_HOLD_US, MBR_OPEN_DELAY_US, NODE_COUNT, RAT_REQUEST_DELAY_US,
};
use crate::control::load_mgmt::{EP_5MIN_KVA, EP_5S_KVA, EP_SHED_INTERVAL_US};
use crate::crew::eicas::{
    APU_BATT_OVERHEAT_MS, DC28_BUS_MSG_MS, DC28_BUS_MSG_V, DISCH_HIGH_MS, DISCH_LOW_MS, GCB_DISAGREE_MS,
    GCU_28V_FAIL_MS, HBB_LOW_RESET_V, HBB_LOW_V, TRU_LOW_CURRENT_A,
};
use crate::faults::{COUNTER_LIMIT_TICKS, TICK_US};
use crate::net::{EXT_FREQ_HZ, V_EXT_PHASE};
use crate::pdu::rpdu::{AC12_RATINGS_A, DC19_RATINGS_A};
use crate::pdu::spdu::SpduCardKind;
use crate::protection::sspc::RPDU_RATINGS_A;
use crate::protection::tru_oc::TRU_OC_TIERS;
use crate::sources::extpower::{
    EP_OC1_A, EP_OC1_DELAY_S, EP_OC2_A, EP_OC2_DELAY_S, EP_OF_DELAY_S, EP_OF_HZ, EP_OF_RESET_HZ, EP_OV_V,
    EP_POWER_QUALITY_DELAY_S, EP_RECOMMENDED_SOURCE_VA, EP_UF_HZ, EP_UF_RESET_HZ, EP_UV_DELAY_S, EP_UV_V,
};
use crate::sources::gcu::{
    DIFF_DELAY_S, DIFF_THRESH_LOW_A, GcuParams, OC_ASG_A, OC_ASG_RESET_A, OC_MAX_DELAY_S, OC_MIN_DELAY_S,
    OC_VFSG_A, OC_VFSG_RESET_A, OF_DELAY_ASG_S, OF_DELAY_S, OF_PMG_HZ_ASG, OF_PMG_HZ_VFSG, OF2_PMG_HZ_ASG,
    OPEN_PHASE_DELAY_S, OPEN_PHASE_LOW_A, OPEN_PHASE_SPREAD_A, OV_TIERS, PHASE_SEQ_DELAY_S,
    RAT_BREAKER_CLOSE_HZ, UF_DELAY_ASG_S, UF_PMG_HZ, UF_PMG_HZ_ASG, UF2_DELAY_ASG_S, UF2_PMG_HZ_ASG,
    UV_INHIBIT_OC_ASG_A, UV_INHIBIT_OC_VFSG_A,
};
use crate::sources::machine::{MachineKind, MachineParams};
use crate::storage::charger::{CHARGER_CURRENT_LIMIT_A, CHARGER_OUTPUT_V};

fn ms(seconds: f64) -> f64 {
    (seconds * 1000.0).round()
}

fn ms_u(us: u64) -> f64 {
    us as f64 / 1000.0
}

fn secs(us: u64) -> f64 {
    us as f64 / 1_000_000.0
}

fn count_kind(list: &[MachineParams], kind: MachineKind) -> usize {
    list.iter().filter(|m| m.kind == kind).count()
}

fn first_kind(list: &[MachineParams], kind: MachineKind) -> Option<&MachineParams> {
    list.iter().find(|m| m.kind == kind)
}

fn max_of(list: &[f64]) -> f64 {
    list.iter().copied().fold(0.0, f64::max)
}

fn pmg_ratio(m: &MachineParams) -> f64 {
    m.pmg_pole_pairs / m.pole_pairs
}

impl Electrical {
    pub(crate) fn bound_params(&self, name: &str) -> Option<SignalValue> {
        self.model_param(name).map(SignalValue::Num)
    }

    pub fn param_is_unmodelled(&self, name: &str) -> bool {
        self.model_param(name).is_none() && rated::lookup(name).is_some()
    }

    fn model_param(&self, name: &str) -> Option<f64> {
        let net = self.plant.net();
        let vfsg = first_kind(&self.machine_params, MachineKind::Vfsg);
        let asg = first_kind(&self.machine_params, MachineKind::Asg);
        let rat = MachineParams::rat();
        let vfsg_count = count_kind(&self.machine_params, MachineKind::Vfsg);
        Some(match name {
            "vfsg.count_total" => vfsg_count as f64,
            "vfsg.freq_min" => crate::sources::machine::VFSG_FREQ_RANGE_HZ.0,
            "vfsg.freq_max" => crate::sources::machine::VFSG_FREQ_RANGE_HZ.1,
            "vfsg.mode_count" => super::rig::VFSG_MODES.len() as f64,
            "vfsg.motor_torque_ftlb" => super::rig::VFSG_MOTOR_TORQUE_FTLB,
            "vfsg.duty_cycle_cooldown_min" => super::rig::VFSG_DUTY_COOLDOWN_S / 60.0,
            "vfsg.cooldown_torque_limit_pct" => super::rig::VFSG_COOLDOWN_TORQUE_FRACTION * 100.0,
            "vfsg.thermal_disconnect_temp_f" => crate::sources::machine::c_to_f(vfsg?.thermal_disconnect_c?),
            "vfsg.thermal_disconnect_temp_c" => vfsg?.thermal_disconnect_c?,
            "vfsg.oil_normal_min_f" => crate::sources::machine::VFSG_OIL_NORMAL_F.0,
            "vfsg.oil_normal_max_f" => crate::sources::machine::VFSG_OIL_NORMAL_F.1,
            "vfsg.oil_normal_min_c" => {
                crate::sources::machine::f_to_c(crate::sources::machine::VFSG_OIL_NORMAL_F.0).round()
            }
            "vfsg.oil_normal_max_c" => {
                crate::sources::machine::f_to_c(crate::sources::machine::VFSG_OIL_NORMAL_F.1).round()
            }
            "vfsg.drive_disc_condition_time_ms" => ms_u(super::drive_disc::DRIVE_DISC_CONDITION_US),
            "vfsg.drive_switch_max_press_s" => secs(super::drive_disc::DRIVE_SWITCH_MAX_PRESS_US),
            "vfsg.drive_switch_wait_s" => secs(super::drive_disc::DRIVE_SWITCH_WAIT_US),
            "vfsg.lop_auto_disconnect_time_ms" => ms_u(super::drive_disc::LOP_AUTO_DISCONNECT_US),
            "vfsg.overload_125pct_kva" => crate::control::load_mgmt::VFSG_5MIN_KVA,
            "vfsg.overload_125pct_time_s" => secs(crate::control::load_mgmt::GEN_5MIN_US),
            "vfsg.overload_175pct_kva" => crate::control::load_mgmt::VFSG_5S_KVA,
            "vfsg.overload_175pct_time_s" => secs(crate::control::load_mgmt::GEN_5S_US),
            "start.ext_pwr_min_fwd_sources" => super::rig::START_EXT_MIN_FWD_SOURCES as f64,
            "apu.continuous_kva_sea_level" => crate::control::bpcu::apu_continuous_kva(0.0),
            "apu.continuous_kva_43100ft" => crate::control::bpcu::apu_continuous_kva(43_100.0),
            "ext_pwr.aft_system_limit_kva" => crate::protection::atru_prot::ATRU_115_INPUT_KVA,
            "ext_pwr.fwd_off_voltage_rms" => crate::protection::epc_supervisor::FWD_OFF_VOLTAGE_RMS,
            "load_mgmt.pecs_thermal_shed_f" => crate::control::load_mgmt::PECS_THERMAL_SHED_F,
            "load_mgmt.ground_handling_freq_limit_hz" => crate::control::bpcu::GROUND_HANDLING_MAX_HZ,
            "cmsc.rating_kw" => super::rig::CMSC_RATING_W / 1000.0,
            "batt.battery_off_delay_min_s" => secs(crate::control::bpcu::GROUND_BATTERY_ONLY_OFF_US),
            "batt.battery_off_delay_max_s" => secs(crate::control::bpcu::BATTERY_OFF_MAX_US),
            "batt.standby_parallel_ms" => ms_u(crate::control::bpcu::STANDBY_PARALLEL_US),
            "batt.full_charge_time_max_min" => {
                crate::storage::charger::FULL_CHARGE_TIME_MAX_US as f64 / 60.0e6
            }
            "batt.disch_inhibit_a" => crate::storage::monitor::DISCH_HIGH_A,
            "batt.disch_inhibit_low_a" => crate::storage::monitor::INHIBIT_LOW_A,
            "batt.disch_inhibit_hbb_v" => crate::storage::monitor::INHIBIT_HBB_V,
            "batt.disch_inhibit_hbb_time_s" => secs(crate::storage::monitor::INHIBIT_HBB_US),
            "batt.disch_clear_a" => crate::storage::monitor::CLEAR_A,
            "batt.disch_clear_hbb_v" => crate::storage::monitor::CLEAR_HBB_V,
            "batt.disch_msg_low_a" => crate::storage::monitor::MSG_LOW_A,
            "batt.fault_sense_s" => secs(crate::storage::monitor::FAULT_SENSE_US),
            "batt.hbb_uv_v" => crate::storage::monitor::HBB_UV_V,
            "batt.hbb_uv_time_s" => secs(crate::storage::monitor::HBB_UV_US),
            "batt.hbb_ov_v" => crate::storage::monitor::HBB_OV_V,
            "batt.hbb_ov_time_s" => secs(crate::storage::monitor::HBB_OV_US),
            "batt.full_charge_v" => crate::storage::monitor::FULL_CHARGE_V,
            "batt.full_charge_a" => crate::storage::monitor::FULL_CHARGE_A,
            "batt.charge_high_min" => crate::storage::monitor::TOW_HIGH_MIN,
            "batt.charge_high_v" => crate::storage::monitor::TOW_HIGH_V,
            "batt.charge_med_min" => crate::storage::monitor::TOW_MEDIUM_MIN,
            "batt.charge_low_min" => crate::storage::monitor::TOW_LOW_MIN,
            "batt.charge_low_v" => crate::storage::monitor::TOW_LOW_V,
            "batt.low_msg_time_s" => crate::crew::eicas::HBB_LOW_MSG_MS as f64 / 1000.0,
            "batt.switch_off_msg_s" => crate::crew::eicas::BATTERY_OFF_MSG_MS as f64 / 1000.0,
            "batt.shutdown_period_s" => secs(crate::control::bpcu::MBR_SHUTDOWN_HOLD_US),
            "batt.ccs_holdup_after_mbr_s" => secs(crate::control::bpcu::CCS_HOLD_US),
            "batt.ground_battery_only_off_delay_s" => secs(crate::control::bpcu::GROUND_BATTERY_ONLY_OFF_US),
            "batt.standby_parallel_instr_bus_v" => crate::control::bpcu::STANDBY_PARALLEL_INSTR_V,
            "batt.standby_parallel_tru_a" => crate::control::bpcu::STANDBY_PARALLEL_TRU_A,
            "vfsg.start_torque_ftlb" => super::rig::VFSG_START_TORQUE_FTLB,
            "cmsc.overload_kw" => super::rig::CMSC_OVERLOAD_W / 1000.0,
            "cmsc.overload_time_min" => super::rig::CMSC_MES_LIMIT_S / 60.0,
            "vfsg.duty_cycle_full_torque_min" => super::rig::CMSC_MES_LIMIT_S / 60.0,
            "start.vfsg_torque_full_s" => super::rig::CMSC_MES_LIMIT_S,
            "gec1.close_pmg_hz_max" => super::rig::GEC1_CLOSE_HZ,
            "gec1.open_pmg_hz_above" => super::rig::GEC1_OPEN_HZ,
            "gec2.close_pmg_hz_max" => super::rig::GEC2_CLOSE_HZ,
            "gec2.open_pmg_hz_min" => super::rig::GEC2_OPEN_HZ,
            "gec1.close_input_rpm" => super::rig::GEC1_CLOSE_HZ / vfsg?.pmg_freq_hz(1.0),
            "gec2.close_input_rpm" => super::rig::GEC2_CLOSE_HZ / vfsg?.pmg_freq_hz(1.0),
            "gec2.open_input_rpm" => super::rig::GEC2_OPEN_HZ / vfsg?.pmg_freq_hz(1.0),
            "vfsg.gen_mode_entry_rpm" => super::rig::ENGINE_IDLE_RPM,
            "vfsg.gen_mode_entry_freq_hz" => vfsg?.electrical_freq_hz(super::rig::ENGINE_IDLE_RPM),
            "apu.ready_to_load_delay_s" => super::rig::APU_READY_HOLD_S,
            "apu.ready_to_load_speed_pct" => super::rig::APU_READY_FRACTION * 100.0,
            "batt.apu_start_attempt_s" => super::rig::APU_BATT_ATTEMPT_S,
            "batt.apu_start_attempts" => f64::from(super::rig::APU_ATTEMPTS),
            "atru.rating_787_9_continuous_kva" => crate::protection::atru_prot::ATRU_KVA_TIERS[0].0,
            "atru.rating_787_9_10min_kva" => crate::protection::atru_prot::ATRU_KVA_TIERS[1].0,
            "atru.rating_787_9_5min_kva" => crate::protection::atru_prot::ATRU_KVA_TIERS[2].0,
            "atru.rating_787_9_30s_kva" => crate::protection::atru_prot::ATRU_KVA_TIERS[3].0,
            "atru.rating_787_9_5s_kva" => crate::protection::atru_prot::ATRU_KVA_TIERS[4].0,
            "atru.rating_115v_input_kva" => crate::protection::atru_prot::ATRU_115_INPUT_KVA,
            "atru.out_vdc_min_half" => crate::protection::atru_prot::ATRU_OUT_HALF_V.0,
            "atru.out_vdc_nominal_half" => crate::protection::atru_prot::ATRU_OUT_HALF_V.1,
            "atru.out_vdc_max_half" => crate::protection::atru_prot::ATRU_OUT_HALF_V.2,
            "atru.in_freq_min_hz" => crate::protection::atru_prot::ATRU_IN_HZ.0,
            "atru.in_freq_max_hz" => crate::protection::atru_prot::ATRU_IN_HZ.1,
            "atru.in_115_freq_min_hz" => crate::protection::atru_prot::ATRU_IN_115_HZ.0,
            "atru.in_115_freq_max_hz" => crate::protection::atru_prot::ATRU_IN_115_HZ.1,
            "load_mgmt.hpx_repeat_window_min" => {
                crate::control::load_mgmt::HPX_REPEAT_WINDOW_US as f64 / 60.0e6
            }
            "msg.elec_235v_bus_uv_vrms" => crate::crew::eicas::AC_BUS_MSG_V,
            "msg.elec_235v_bus_uv_time_ms" => crate::crew::eicas::AC_BUS_MSG_MS as f64,
            "msg.apu_batt_charger_s" => crate::crew::eicas::APU_CHARGER_MSG_MS as f64 / 1000.0,
            "rat.rcb_rating_a" => super::interlock::listed_rating_a("RCB")?,
            "bpcu.count" => self.bpcu_out.controlling.len() as f64,
            "ext_pwr.epc_chatter_changes" => crate::protection::epc_supervisor::EPC_CHATTER_CHANGES as f64,
            "ext_pwr.epc_chatter_window_s" => secs(crate::protection::epc_supervisor::EPC_CHATTER_WINDOW_US),
            "ext_pwr.fwd_off_current_arms" => crate::protection::epc_supervisor::FWD_OFF_CURRENT_A,
            "ext_pwr.fwd_off_time_ms" => ms_u(crate::protection::epc_supervisor::FWD_OFF_US),
            "ext_pwr.aft_fail_close_current_a" => crate::protection::epc_supervisor::AFT_FAIL_CLOSE_CURRENT_A,
            "ext_pwr.aft_fail_close_time_ms" => ms_u(crate::protection::epc_supervisor::AFT_FAIL_CLOSE_US),
            "load_mgmt.pecs_thermal_shed_c" => crate::control::load_mgmt::PECS_THERMAL_SHED_C.round(),
            "load_mgmt.ep_cmsc_stop_current_a" => crate::control::load_mgmt::EP_CMSC_STOP_A,
            "load_mgmt.ep_cmsc_stop_time_min" => crate::control::load_mgmt::EP_CMSC_STOP_US as f64 / 60.0e6,
            "load_mgmt.ep_cmsc_stop_current_fast_a" => crate::control::load_mgmt::EP_CMSC_STOP_FAST_A,
            "prot.tie_diff_threshold_a" => crate::protection::zone_diff::TIE_DIFF_A,
            "prot.tie_diff_time_s" => secs(crate::protection::zone_diff::TIE_DIFF_HW_US),
            "prot.tie_diff_sw_delay_ms" => ms_u(crate::protection::zone_diff::TIE_DIFF_SW_US),
            "prot.atu_diff_a" => crate::protection::zone_diff::ATU_DIFF_A,
            "prot.atu_diff_time_ms" => ms_u(crate::protection::zone_diff::ATU_DIFF_US),
            "prot.unbalanced_feeder_a" => crate::sources::gcu::UNBALANCED_FEEDER_A,
            "prot.unbalanced_feeder_time_s" => crate::sources::gcu::UNBALANCED_FEEDER_DELAY_S,
            "prot.field_driver_vr_disable_ms" => ms(crate::sources::gcu::FIELD_DRIVER_VR_DISABLE_S),
            "prot.field_driver_breaker_open_ms" => ms(crate::sources::gcu::FIELD_DRIVER_BREAKER_OPEN_S),
            "prot.field_driver_fault_time_ms" => ms(crate::sources::gcu::FIELD_DRIVER_FAULT_S),
            "prot.ground_fault_a" => crate::sources::gcu::GROUND_FAULT_A,
            "prot.ground_fault_time_s" => crate::sources::gcu::GROUND_FAULT_DELAY_S,
            "prot.dc_content_a" => crate::sources::gcu::DC_CONTENT_A,
            "prot.dc_content_time_ms" => ms(crate::sources::gcu::DC_CONTENT_DELAY_S),
            "prot.dc_content_reset_a" => crate::sources::gcu::DC_CONTENT_RESET_A,
            "prot.srd_time_s" => crate::sources::gcu::SRD_DELAY_S,
            "prot.engine_idle_pmg_hz" => crate::sources::gcu::ENGINE_IDLE_PMG_HZ,
            "prot.ovtpu_reset_ms" => ms(crate::sources::gcu::OVTPU_RESET_S),
            "prot.ovtpu_uv_inhibit_ms" => ms(crate::sources::gcu::OVTPU_UV_INHIBIT_S),
            "prot.ovtpu_srd_inhibit_ms" => ms(crate::sources::gcu::OVTPU_SRD_INHIBIT_S),
            "prot.tru_oc_tier1_a" => TRU_OC_TIERS[0].0,
            "prot.tru_oc_tier1_time_s" => TRU_OC_TIERS[0].1,
            "prot.tru_oc_tier2_a" => TRU_OC_TIERS[1].0,
            "prot.tru_oc_tier2_time_s" => TRU_OC_TIERS[1].1,
            "prot.tru_oc_tier3_a" => TRU_OC_TIERS[2].0,
            "prot.tru_oc_tier3_time_s" => TRU_OC_TIERS[2].1,
            "prot.fail_to_close_btb_ms" => ms_u(crate::control::bpcu::FAIL_TO_CLOSE_BTB_US),
            "prot.fail_to_close_bsb_tru_ms" => ms_u(crate::control::bpcu::FAIL_TO_CLOSE_BSB_US),
            "vfsg.count_per_engine" => (vfsg_count / EPC_BUS.len()) as f64,
            "asg.count" => count_kind(&self.machine_params, MachineKind::Asg) as f64,
            "gcu.count" => self.gcus.len() as f64,
            "cmsc.count" => net.conversion_count("CMSC") as f64,
            "atru.count" => net.conversion_count("ATRU") as f64,
            "atu.count" => net.conversion_count("ATU") as f64,
            "tru.count" => net.conversion_count("TRU") as f64,
            "spu.count" => net.conversion_count("SPU") as f64,
            "ext_pwr.receptacle_count" => self.ext_mon.len() as f64,
            "vfsg.rating_kva" => vfsg?.rated_va / 1000.0,
            "vfsg.voltage_vll" => GcuParams::vfsg().regulated_v,
            "vfsg.pole_count" => vfsg?.pole_pairs * 2.0,
            "asg.rating_kva" => asg?.rated_va / 1000.0,
            "asg.voltage_vll" => GcuParams::asg().regulated_v,
            "ext_pwr.fwd_rating_kva" | "start.ext_pwr_fwd_source_kva" => EXT_KVA,
            "ext_pwr.aft_rating_kva" => EP_RECOMMENDED_SOURCE_VA / 1000.0,
            "ext_pwr.voltage_vll" => V_EXT_PHASE,
            "ext_pwr.freq_hz" => EXT_FREQ_HZ,
            "ext_pwr.fwd_quality_time_ms" => ms(EP_POWER_QUALITY_DELAY_S),
            "ext_pwr.transfer_delay_ms" => secs(EXT_CONTACTOR_HOLD_US) * 1000.0,
            "ext_pwr.oc_stage1_tier1_a" => EP_OC1_A,
            "ext_pwr.oc_stage1_tier1_time_s" => EP_OC1_DELAY_S,
            "ext_pwr.oc_stage1_tier2_a" => EP_OC2_A,
            "ext_pwr.oc_stage1_tier2_time_s" => EP_OC2_DELAY_S,
            "ext_pwr.ov_rms" => EP_OV_V,
            "ext_pwr.uv_rms" => EP_UV_V,
            "ext_pwr.uv_delay_s" => EP_UV_DELAY_S,
            "ext_pwr.of_hz" => EP_OF_HZ,
            "ext_pwr.of_reset_hz" => EP_OF_RESET_HZ,
            "ext_pwr.uf_hz" => EP_UF_HZ,
            "ext_pwr.uf_reset_hz" => EP_UF_RESET_HZ,
            "ext_pwr.freq_trip_delay_s" => EP_OF_DELAY_S,
            "ext_pwr.load_shed_5min_kva" => EP_5MIN_KVA,
            "ext_pwr.load_shed_5s_kva" => EP_5S_KVA,
            "ext_pwr.load_shed_interval_s" => secs(EP_SHED_INTERVAL_US),
            "rat.rating_kva" => rat.rated_va / 1000.0,
            "rat.voltage_vll" => rat.nominal_phase_v,
            "rat.rcb_close_min_hz" => RAT_BREAKER_CLOSE_HZ,
            "rat.deploy_loss_time_s" => secs(RAT_REQUEST_DELAY_US),
            "bus.main_235_count" => BUS_COUNT as f64,
            "bus.btb_count" => BTB_COUNT as f64,
            "bus.ring_bus_count" => NODE_COUNT as f64,
            "bus.l_r_115_count" => net.bus_names().iter().filter(|b| b.starts_with("AC115_")).count() as f64,
            "rpdu.ac12_sspc_count" => AC12_RATINGS_A.len() as f64,
            "rpdu.dc19_sspc_count" => DC19_RATINGS_A.len() as f64,
            "rpdu.small_load_limit_a" => max_of(&RPDU_RATINGS_A),
            "spdu.max_dc_a" => max_of(&SpduCardKind::Dc2.ratings_a()),
            "spdu.max_ac_a" => max_of(&SpduCardKind::Ac1.ratings_a()),
            "spdu.dc2_sspc_a" => SpduCardKind::Dc2.ratings_a()[0],
            "spdu.ac1_sspc_a" => SpduCardKind::Ac1.ratings_a()[0],
            "spdu.ac2_sspc_a" => SpduCardKind::Ac2.ratings_a()[0],
            "prot.ov_tier1_vrms" => OV_TIERS[0].0,
            "prot.ov_tier1_time_s" => OV_TIERS[0].1,
            "prot.ov_tier9_vrms" => OV_TIERS[OV_TIERS.len() - 1].0,
            "prot.ov_tier9_time_s" => OV_TIERS[OV_TIERS.len() - 1].1,
            "prot.uv_inhibit_vfsg_a" => UV_INHIBIT_OC_VFSG_A,
            "prot.uv_inhibit_asg_a" => UV_INHIBIT_OC_ASG_A,
            "prot.phase_seq_delay_ms" => ms(PHASE_SEQ_DELAY_S),
            "prot.vfsg_oc_a" => OC_VFSG_A,
            "prot.vfsg_oc_reset_a" => OC_VFSG_RESET_A,
            "prot.vfsg_oc_min_time_s" => OC_MIN_DELAY_S,
            "prot.vfsg_oc_max_time_s" => OC_MAX_DELAY_S,
            "prot.asg_oc_a" => OC_ASG_A,
            "prot.asg_oc_reset_a" => OC_ASG_RESET_A,
            "prot.open_phase_time_s" => OPEN_PHASE_DELAY_S,
            "prot.open_phase_low_a" => OPEN_PHASE_LOW_A,
            "prot.open_phase_next_a" => OPEN_PHASE_SPREAD_A,
            "prot.diff_sw_delay_ms" => ms(DIFF_DELAY_S),
            "prot.diff_threshold_a" => DIFF_THRESH_LOW_A,
            "prot.pmg_to_por_ratio_vfsg" => pmg_ratio(vfsg?),
            "prot.pmg_to_por_ratio_asg" => pmg_ratio(asg?),
            "prot.vfsg_rpm_to_por_ratio" => 60.0 / vfsg?.pole_pairs,
            "prot.vfsg_underfreq_pmg_hz" => UF_PMG_HZ,
            "prot.vfsg_underfreq_por_hz" => UF_PMG_HZ / pmg_ratio(vfsg?),
            "prot.vfsg_overfreq_pmg_hz" => OF_PMG_HZ_VFSG,
            "prot.vfsg_overfreq_por_hz" => OF_PMG_HZ_VFSG / pmg_ratio(vfsg?),
            "prot.vfsg_overfreq_time_ms" => ms(OF_DELAY_S),
            "prot.asg_of1_pmg_hz" => OF_PMG_HZ_ASG,
            "prot.asg_of1_time_s" => OF_DELAY_ASG_S,
            "prot.asg_of2_pmg_hz" => OF2_PMG_HZ_ASG,
            "prot.asg_uf1_pmg_hz" => UF_PMG_HZ_ASG,
            "prot.asg_uf1_time_s" => UF_DELAY_ASG_S,
            "prot.asg_uf2_pmg_hz" => UF2_PMG_HZ_ASG,
            "prot.asg_uf2_time_ms" => ms(UF2_DELAY_ASG_S),
            "batt.main_capacity_ah" => net.battery().params.capacity_ah,
            "batt.charger_current_a" => CHARGER_CURRENT_LIMIT_A,
            "batt.charger_voltage_v" => CHARGER_OUTPUT_V,
            "batt.mbr_open_delay_s" => secs(MBR_OPEN_DELAY_US),
            "batt.cit_ground_to_air_s" => secs(CIT_GROUND_TO_AIR_US),
            "batt.cit_post_transfer_s" => secs(INSTR_PARALLEL_HOLD_US),
            "msg.elec_28v_bus_v" => DC28_BUS_MSG_V,
            "msg.elec_28v_bus_time_s" => DC28_BUS_MSG_MS as f64 / 1000.0,
            "msg.apu_batt_overheat_s" => APU_BATT_OVERHEAT_MS as f64 / 1000.0,
            "msg.tru_current_a" => TRU_LOW_CURRENT_A,
            "prot.gcb_status_disagree_ms" => GCB_DISAGREE_MS as f64,
            "prot.gcu_28v_fail_s" => GCU_28V_FAIL_MS as f64 / 1000.0,
            "batt.disch_inhibit_time_s" => DISCH_HIGH_MS as f64 / 1000.0,
            "batt.disch_inhibit_low_time_min" => DISCH_LOW_MS as f64 / 60_000.0,
            "batt.low_msg_hbb_v" => HBB_LOW_V,
            "batt.low_msg_reset_v" => HBB_LOW_RESET_V,
            "gcu.counter_limit_days" => COUNTER_LIMIT_TICKS as f64 * secs(TICK_US) / 86_400.0,
            _ => return None,
        })
    }
}
