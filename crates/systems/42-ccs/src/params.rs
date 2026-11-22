use crate::cabinet::{
    FULL_PBIT_MAX_US, FULL_PBIT_MIN_US, GPM_COUNT, INHIBITED_DISPLAYS_US, INHIBITED_HOSTED_MAX_US,
    INHIBITED_HOSTED_MIN_US,
};
use crate::ccs::{DAYS_51_US, NETWORK_BOOT_MAX_US, RDC_ANALOG_IFZ, RDC_CAN_IFZ, RDC_IFZ};
use crate::cdn::{RATE_10M, RATE_100M};
use crate::gpm::IS_PARTITIONS;
use crate::network::{CAS_HOSTS, GPMS};
use crate::pcm::{HBB_HOLDUP_US, PCM_OUT_V, PCM_SWITCH_V};

pub const CABINETS: usize = 2;
pub const PCMS_PER_CABINET: usize = 2;
pub const ACS_PER_CABINET: usize = 2;
pub const FOX_PER_CABINET: usize = 2;
pub const ARS_COUNT: usize = 6;
pub const ARS_PER_CHANNEL: usize = 3;
pub const ACS_PORTS: usize = 24;
pub const ACS_FAST_PORTS: usize = 20;
pub const ARS_PORTS: usize = 24;
pub const ARS_FIBRE_PORTS: usize = 4;
pub const GPM_CPUS: usize = 2;

fn s(us: u64) -> f64 {
    us as f64 / 1.0e6
}

pub fn param(name: &str) -> Option<f64> {
    Some(match name {
        "cabinet_count" => CABINETS as f64,
        "gpm_per_cabinet" => GPM_COUNT as f64,
        "gpm_total" => GPMS.len() as f64,
        "pcm_per_cabinet" => PCMS_PER_CABINET as f64,
        "acs_per_cabinet" => ACS_PER_CABINET as f64,
        "fox_per_cabinet" => FOX_PER_CABINET as f64,
        "ars_count" => ARS_COUNT as f64,
        "ars_per_channel" => ARS_PER_CHANNEL as f64,
        "acs_ports" => ACS_PORTS as f64,
        "acs_100m_ports" => ACS_FAST_PORTS as f64,
        "ars_ports" => ARS_PORTS as f64,
        "ars_fibre_ports" => ARS_FIBRE_PORTS as f64,
        "pcm_switch_v" => PCM_SWITCH_V,
        "pcm_out_v" => PCM_OUT_V,
        "hbb_holdup_s" => s(HBB_HOLDUP_US),
        "full_pbit_max_s" => s(FULL_PBIT_MAX_US),
        "ccr_startup_min_s" => s(FULL_PBIT_MIN_US),
        "ccr_startup_max_s" => s(FULL_PBIT_MAX_US),
        "inhibited_displays_s" => s(INHIBITED_DISPLAYS_US),
        "inhibited_hosted_min_s" => s(INHIBITED_HOSTED_MIN_US),
        "inhibited_hosted_max_s" => s(INHIBITED_HOSTED_MAX_US),
        "network_boot_max_s" => s(NETWORK_BOOT_MAX_US),
        "fibre_mbps" => RATE_100M / 1.0e6,
        "copper_inside_mbps" => RATE_100M / 1.0e6,
        "copper_outside_mbps" => RATE_10M / 1.0e6,
        "is_partitions" => IS_PARTITIONS.len() as f64,
        "rdc_ifz" => RDC_IFZ as f64,
        "rdc_can_ifz" => RDC_CAN_IFZ as f64,
        "rdc_analog_ifz" => RDC_ANALOG_IFZ as f64,
        "gpm_cpus" => GPM_CPUS as f64,
        "stale_monitor_days" => DAYS_51_US as f64 / 86_400.0e6,
        "cas_hosts" => CAS_HOSTS.len() as f64,
        "gnd_test_delay_s" => s(crate::platform::GND_TEST_DELAY_US),
        _ => return None,
    })
}
