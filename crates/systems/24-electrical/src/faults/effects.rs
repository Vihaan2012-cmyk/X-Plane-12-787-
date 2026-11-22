#[derive(Clone, Debug, Default, PartialEq)]
pub struct Magnitudes {
    entries: Vec<(String, f64)>,
}

impl Magnitudes {
    pub fn new(entries: Vec<(String, f64)>) -> Self {
        Self { entries }
    }

    pub fn from_pairs(pairs: &[(&str, f64)]) -> Self {
        Self {
            entries: pairs.iter().map(|(m, v)| ((*m).to_string(), *v)).collect(),
        }
    }

    pub fn get(&self, mode: &str) -> f64 {
        self.entries
            .iter()
            .find(|(m, _)| m == mode)
            .map_or(0.0, |(_, v)| v.clamp(0.0, 1.0))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(|(_, v)| *v <= 0.0)
    }
}

pub const BINARY_THRESHOLD: f64 = 0.5;
pub const MAX_RESISTANCE_GROWTH: f64 = 99.0;
pub const GCU_DRIFT_MAX_V: f64 = 40.0;
pub const TICK_US: u64 = 10_000;
pub const COUNTER_LIMIT_TICKS: u64 = 1 << 31;
pub const TELEMETRY_BIAS_MAX: f64 = 0.25;

pub fn is_set(m: f64) -> bool {
    m >= BINARY_THRESHOLD
}

pub fn resistance_multiplier(m: f64) -> f64 {
    1.0 + MAX_RESISTANCE_GROWTH * m.clamp(0.0, 1.0)
}

pub fn ground_fault_resistance_ohm(m: f64) -> Option<f64> {
    let m = m.clamp(0.0, 1.0);
    if m <= 0.0 {
        None
    } else {
        Some(1e6_f64 * (0.05_f64 / 1e6_f64).powf(m))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactorEffect {
    pub welded: bool,
    pub coil_open: bool,
    pub aux_stuck: bool,
    pub stuck_open: bool,
    pub resistance_mult: f64,
    pub operate_time_mult: f64,
}

pub fn contactor_effect(m: &Magnitudes) -> ContactorEffect {
    ContactorEffect {
        welded: is_set(m.get("weld")),
        coil_open: is_set(m.get("coil-open")),
        aux_stuck: is_set(m.get("aux-stuck")),
        stuck_open: is_set(m.get("stuck-open")),
        resistance_mult: resistance_multiplier(m.get("high-resistance")),
        operate_time_mult: 1.0 + 9.0 * m.get("slow-operate"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SspcEffect {
    pub stuck_open: bool,
    pub stuck_closed: bool,
    pub force_trip: bool,
    pub arc_false_trip: bool,
    pub current_telemetry_scale: f64,
    pub voltage_telemetry_scale: f64,
    pub leakage_mult: f64,
    pub trip_disabled: bool,
}

pub fn sspc_effect(m: &Magnitudes) -> SspcEffect {
    let bias = TELEMETRY_BIAS_MAX * m.get("telemetry-bias");
    SspcEffect {
        stuck_open: is_set(m.get("stuck-open")),
        stuck_closed: is_set(m.get("stuck-closed")),
        force_trip: is_set(m.get("spurious-trip")),
        arc_false_trip: is_set(m.get("arc-detector-false-trip")),
        current_telemetry_scale: 1.0 + bias,
        voltage_telemetry_scale: 1.0 - 0.5 * bias,
        leakage_mult: 1.0 + 49.0 * m.get("leakage-rise"),
        trip_disabled: is_set(m.get("fail-to-trip")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WireEffect {
    pub open: bool,
    pub resistance_mult: f64,
    pub ground_fault_ohm: Option<f64>,
    pub bus_short: bool,
    pub arcing: bool,
}

pub fn wire_effect(m: &Magnitudes) -> WireEffect {
    let strand = m.get("high-resistance");
    WireEffect {
        open: is_set(m.get("open")),
        resistance_mult: resistance_multiplier(strand),
        ground_fault_ohm: ground_fault_resistance_ohm(m.get("chafe-to-ground")),
        bus_short: is_set(m.get("short-to-bus")),
        arcing: is_set(m.get("arc-tracking")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConnectorEffect {
    pub open: bool,
    pub resistance_mult: f64,
    pub pin_short: bool,
}

pub fn connector_effect(m: &Magnitudes) -> ConnectorEffect {
    ConnectorEffect {
        open: is_set(m.get("intermittent-open")),
        resistance_mult: resistance_multiplier(m.get("high-resistance")),
        pin_short: is_set(m.get("pin-short")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellEffect {
    pub internal_short_ohm: Option<f64>,
    pub capacity_scale: f64,
    pub r0_mult: f64,
    pub extra_heat_w: f64,
    pub runaway_precursor: bool,
}

pub fn battery_cell_effect(m: &Magnitudes) -> CellEffect {
    let fade = m.get("capacity-fade");
    let growth = m.get("resistance-growth");
    let precursor = m.get("thermal-runaway-precursor");
    CellEffect {
        internal_short_ohm: ground_fault_resistance_ohm(m.get("internal-short")).map(|r| r.clamp(0.05, 1e4)),
        capacity_scale: 1.0 - 0.6 * fade,
        r0_mult: (1.0 + 2.0 * fade) * (1.0 + 4.0 * growth),
        extra_heat_w: 80.0 * precursor,
        runaway_precursor: is_set(precursor),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MachineEffect {
    pub friction_loss_w: f64,
    pub vibration: f64,
    pub oil_flow_scale: f64,
    pub exciter_ripple: f64,
    pub phase_open: bool,
    pub winding_short: bool,
    pub pmg_scale: f64,
    pub disconnect_jam: bool,
}

pub fn machine_effect(m: &Magnitudes) -> MachineEffect {
    let bearing = m.get("bearing-wear");
    let imbalance = m.get("rotor-imbalance");
    MachineEffect {
        friction_loss_w: 1500.0 * bearing,
        vibration: (bearing + imbalance).min(1.0),
        oil_flow_scale: 1.0 - m.get("low-oil"),
        exciter_ripple: m.get("rotating-diode-short"),
        phase_open: is_set(m.get("open-phase")),
        winding_short: is_set(m.get("winding-short")),
        pmg_scale: 1.0 - m.get("pmg-failure"),
        disconnect_jam: is_set(m.get("disconnect-jam")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GcuEffect {
    pub setpoint_offset_v: f64,
    pub cpu_failed: bool,
    pub powered_time_preload_us: u64,
    pub ttpc_lost: bool,
    pub ct_offset_a: f64,
    pub fan_failed: bool,
    pub field_driver_short: bool,
}

pub const CT_OFFSET_MAX_A: f64 = 45.0;

pub fn counter_preload_us(m: f64) -> u64 {
    (m.clamp(0.0, 1.0) * (COUNTER_LIMIT_TICKS * TICK_US) as f64) as u64
}

pub fn gcu_effect(m: &Magnitudes) -> GcuEffect {
    GcuEffect {
        setpoint_offset_v: GCU_DRIFT_MAX_V * (m.get("regulator-drift-high") - m.get("regulator-drift-low")),
        cpu_failed: is_set(m.get("cpu-fail")),
        powered_time_preload_us: counter_preload_us(m.get("counter-overflow-248d")),
        ttpc_lost: is_set(m.get("ttpc-loss")),
        ct_offset_a: CT_OFFSET_MAX_A * m.get("current-transformer-offset"),
        fan_failed: is_set(m.get("fan-failure")),
        field_driver_short: is_set(m.get("field-driver-short")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConverterEffect {
    pub output_scale: f64,
    pub cooling_scale: f64,
    pub ripple: f64,
    pub ground_fault: bool,
    pub forced_trip: bool,
}

pub fn converter_effect(m: &Magnitudes) -> ConverterEffect {
    ConverterEffect {
        output_scale: 1.0 - 0.5 * m.get("degraded-output"),
        cooling_scale: 1.0 - m.get("cooling-loss"),
        ripple: m.get("diode-open"),
        ground_fault: is_set(m.get("ground-fault")),
        forced_trip: is_set(m.get("overload")) || is_set(m.get("overcurrent-trip")) || is_set(m.get("short")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndicationEffect {
    pub forced_low: bool,
    pub forced_high: bool,
    pub bias: f64,
    pub dim: f64,
    pub element_open: bool,
}

pub fn indication_effect(m: &Magnitudes) -> IndicationEffect {
    IndicationEffect {
        forced_low: is_set(m.get("stuck-low")),
        forced_high: is_set(m.get("stuck-high")),
        bias: 0.2 * m.get("bias"),
        dim: m.get("dim"),
        element_open: is_set(m.get("filament-open")) || is_set(m.get("light-element-open")),
    }
}
