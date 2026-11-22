use failures::Registry;

pub const VFSG: &str = "vfsg";
pub const ASG: &str = "asg";
pub const RAT_GEN: &str = "rat-gen";
pub const GCU: &str = "gcu";
pub const BPCU: &str = "bpcu";
pub const BATTERY_CELL: &str = "battery-cell";
pub const BATTERY_PACK: &str = "battery-pack";
pub const BMU: &str = "bmu";
pub const CHARGER: &str = "charger";
pub const DIODE_MODULE: &str = "diode-module";
pub const ATRU: &str = "atru";
pub const ATU: &str = "atu";
pub const TRU: &str = "tru";
pub const SPU: &str = "spu";
pub const CMSC: &str = "cmsc";
pub const CONTACTOR: &str = "contactor";
pub const ELCC: &str = "elcc";
pub const ELCU: &str = "elcu";
pub const SSPC: &str = "sspc";
pub const TCB: &str = "tcb";
pub const RPDU_CONTROLLER: &str = "rpdu-controller";
pub const RPDU_MODULE: &str = "rpdu-module";
pub const SPDU_CARD: &str = "spdu-card";
pub const WIRE_SEGMENT: &str = "wire-segment";
pub const CONNECTOR: &str = "connector";
pub const GROUND_STUD: &str = "ground-stud";
pub const PANEL_SWITCH: &str = "panel-switch";
pub const LAMP: &str = "lamp";
pub const EICAS_INPUT: &str = "eicas-input";
pub const OIL_AIR_VALVE: &str = "oil-air-valve";
pub const RAT_HEATER: &str = "rat-heater";
pub const EXT_POWER_RECEPTACLE: &str = "ext-power-receptacle";
pub const PECS_PUMP: &str = "pecs-pump";

pub const OBSERVABLE_PREFIXES: [&str; 6] = ["eicas:", "bite:", "sig:", "lamp:", "cbic:", "therm:"];

type Entry = (&'static str, &'static str, &'static [&'static str]);

pub const CATALOG: &[Entry] = &[
    (
        VFSG,
        "bearing-wear",
        &[
            "sig:vfsg.vibration",
            "sig:vfsg.oil_temp",
            "lamp:gen-drive-disc",
            "eicas:ELEC GEN DEBRIS",
        ],
    ),
    (
        VFSG,
        "rotor-imbalance",
        &["sig:vfsg.vibration", "sig:vfsg.freq_ripple"],
    ),
    (
        VFSG,
        "low-oil",
        &[
            "eicas:ELEC GEN DRIVE",
            "lamp:gen-drive-disc",
            "sig:vfsg.oil_press",
        ],
    ),
    (
        VFSG,
        "winding-short",
        &[
            "bite:gcu.differential",
            "eicas:ELEC GEN OFF",
            "sig:vfsg.phase_current",
        ],
    ),
    (
        VFSG,
        "rotating-diode-short",
        &[
            "sig:vfsg.exciter_field",
            "sig:vfsg.phase_voltage_ripple",
            "bite:gcu.regulator",
        ],
    ),
    (
        VFSG,
        "open-phase",
        &[
            "bite:gcu.open_phase",
            "eicas:ELEC GEN OFF",
            "sig:vfsg.phase_current",
        ],
    ),
    (
        VFSG,
        "pmg-failure",
        &["eicas:ELEC GEN OFF", "sig:vfsg.pmg_voltage", "bite:gcu.pmg"],
    ),
    (
        VFSG,
        "disconnect-jam",
        &["lamp:gen-drive-disc", "sig:vfsg.disconnect_state"],
    ),
    (
        ASG,
        "bearing-wear",
        &["sig:asg.vibration", "sig:asg.oil_temp", "eicas:ELEC GEN DEBRIS"],
    ),
    (
        ASG,
        "winding-short",
        &["bite:gcu.differential", "eicas:ELEC GEN OFF APU"],
    ),
    (
        ASG,
        "rotating-diode-short",
        &["sig:asg.exciter_field", "bite:gcu.regulator"],
    ),
    (
        ASG,
        "pmg-failure",
        &["eicas:ELEC GEN OFF APU", "sig:asg.pmg_voltage"],
    ),
    (
        ASG,
        "starter-winding-open",
        &["eicas:ELEC APU START SYS", "sig:asg.start_current"],
    ),
    (
        RAT_GEN,
        "bearing-wear",
        &["sig:rat.vibration", "sig:rat.gen_temp"],
    ),
    (
        RAT_GEN,
        "winding-short",
        &["bite:gcu.differential", "sig:rat.output_voltage"],
    ),
    (RAT_GEN, "pmg-failure", &["sig:rat.pmg_voltage", "bite:gcu.pmg"]),
    (
        GCU,
        "field-driver-short",
        &["bite:gcu.field_driver", "eicas:ELEC GEN OFF", "sig:bus.voltage"],
    ),
    (
        GCU,
        "regulator-drift-high",
        &["sig:bus.voltage", "bite:gcu.overvoltage"],
    ),
    (
        GCU,
        "regulator-drift-low",
        &["sig:bus.voltage", "bite:gcu.undervoltage"],
    ),
    (
        GCU,
        "cpu-fail",
        &[
            "eicas:ELEC GEN CTRL UNIT",
            "bite:gcu.failsafe",
            "eicas:ELEC GEN OFF",
        ],
    ),
    (
        GCU,
        "counter-overflow-248d",
        &["eicas:ELEC GEN OFF", "bite:gcu.powered_time_overflow"],
    ),
    (GCU, "ttpc-loss", &["eicas:ELEC GEN DEBRIS", "bite:gcu.ttpc"]),
    (
        GCU,
        "pin-error",
        &["eicas:ELEC GEN CTRL UNIT", "bite:gcu.pin_config"],
    ),
    (
        GCU,
        "current-transformer-offset",
        &["bite:gcu.differential", "sig:gcu.feeder_current"],
    ),
    (GCU, "fan-failure", &["eicas:ELEC GEN DEBRIS", "therm:gcu.case"]),
    (
        BPCU,
        "cpu-fail",
        &["eicas:ELEC BUS PWR CTRL UNIT", "bite:bpcu.fault"],
    ),
    (
        BPCU,
        "counter-overflow-248d",
        &["eicas:ELEC BUS PWR CTRL UNIT", "bite:bpcu.powered_time_overflow"],
    ),
    (
        BPCU,
        "fan-failure",
        &["eicas:ELEC BUS PWR CTRL UNIT", "therm:bpcu.case"],
    ),
    (BPCU, "ttpc-loss", &["bite:bpcu.ttpc", "eicas:ELEC STANDBY SYS"]),
    (
        BPCU,
        "pin-error",
        &["bite:bpcu.pin_config", "eicas:ELEC STANDBY SYS"],
    ),
    (
        BATTERY_CELL,
        "internal-short",
        &[
            "sig:battery.cell_voltage",
            "sig:battery.temp",
            "eicas:MAIN BATTERY",
        ],
    ),
    (
        BATTERY_CELL,
        "capacity-fade",
        &[
            "sig:battery.soc",
            "sig:battery.cell_voltage",
            "eicas:MAIN BATTERY LOW",
        ],
    ),
    (
        BATTERY_CELL,
        "thermal-runaway-precursor",
        &[
            "sig:battery.temp",
            "sig:battery.cell_voltage",
            "eicas:MAIN BATTERY",
        ],
    ),
    (
        BATTERY_CELL,
        "resistance-growth",
        &["sig:battery.terminal_voltage", "sig:battery.temp"],
    ),
    (
        BATTERY_PACK,
        "imbalance",
        &["bite:bmu.imbalance", "eicas:MAIN BATTERY"],
    ),
    (
        BATTERY_PACK,
        "thermal-runaway",
        &["sig:battery.temp", "eicas:MAIN BATTERY", "lamp:battery-vent-odor"],
    ),
    (
        BATTERY_PACK,
        "burst-disc-open",
        &["lamp:battery-vent-odor", "eicas:MAIN BATTERY"],
    ),
    (
        BATTERY_PACK,
        "cold-lockout",
        &["sig:battery.terminal_voltage", "eicas:MAIN BATTERY LOW"],
    ),
    (
        BATTERY_PACK,
        "temp-sensor-bias",
        &["sig:battery.temp", "bite:bmu.over_temp"],
    ),
    (BMU, "bmu-fail", &["bite:bmu.fail", "eicas:MAIN BATTERY"]),
    (
        BMU,
        "balance-circuit-stuck",
        &["bite:bmu.imbalance", "sig:battery.cell_spread"],
    ),
    (
        BMU,
        "voltage-sense-offset",
        &["sig:battery.cell_voltage", "bite:bmu.over_voltage"],
    ),
    (
        BMU,
        "disconnect-stuck",
        &["sig:battery.current", "bite:bmu.disconnect"],
    ),
    (
        CHARGER,
        "internal-fault",
        &["eicas:ELEC APU BATTERY CHARGER", "sig:charger.output_current"],
    ),
    (
        CHARGER,
        "overtemp",
        &["eicas:ELEC APU BATTERY CHARGER OVERHEAT", "therm:charger.case"],
    ),
    (
        CHARGER,
        "output-drift",
        &["sig:charger.output_voltage", "lamp:battery-charge-level"],
    ),
    (
        CHARGER,
        "cold-lockout",
        &["sig:charger.output_current", "lamp:battery-charge-level"],
    ),
    (
        DIODE_MODULE,
        "open",
        &["sig:battery.current", "eicas:ELEC STANDBY SYS"],
    ),
    (DIODE_MODULE, "short", &["sig:battery.current", "sig:bus.voltage"]),
    (
        DIODE_MODULE,
        "forward-drop-rise",
        &["sig:bus.voltage", "therm:diode.case"],
    ),
    (
        ATRU,
        "degraded-output",
        &["sig:hvdc.voltage", "eicas:ELEC HVDC SYS"],
    ),
    (ATRU, "cooling-loss", &["therm:atru.case", "eicas:ELEC HVDC SYS"]),
    (ATRU, "ground-fault", &["bite:atru.gfi", "eicas:ELEC HVDC SYS"]),
    (ATRU, "overload", &["sig:hvdc.current", "eicas:ELEC HVDC SYS"]),
    (
        ATRU,
        "saturation",
        &["sig:atru.input_current", "sig:ext_power.quality"],
    ),
    (
        ATU,
        "winding-short",
        &["sig:atu.output_voltage", "sig:atu.input_current"],
    ),
    (
        ATU,
        "contactor-failed-open",
        &["eicas:ELEC TRU", "sig:atu.output_voltage"],
    ),
    (ATU, "overtemp", &["therm:atu.case", "sig:atu.output_voltage"]),
    (
        TRU,
        "degraded-output",
        &["sig:tru.output_voltage", "eicas:ELEC TRU"],
    ),
    (TRU, "cooling-loss", &["therm:tru.case", "eicas:ELEC TRU"]),
    (TRU, "diode-open", &["sig:tru.ripple", "sig:tru.output_current"]),
    (
        TRU,
        "overcurrent-trip",
        &["sig:tru.output_current", "eicas:ELEC TRU"],
    ),
    (
        SPU,
        "short",
        &["eicas:ELEC APU START PWR UNIT", "sig:spu.output_current"],
    ),
    (
        SPU,
        "internal-fault",
        &["eicas:ELEC APU START PWR UNIT", "bite:spu.fault"],
    ),
    (
        SPU,
        "housekeeping-fault",
        &["eicas:ELEC APU START SYS", "bite:spu.housekeeping"],
    ),
    (CMSC, "internal-fault", &["eicas:ELEC MC HYD", "bite:cmsc.fault"]),
    (CMSC, "can-loss", &["eicas:ELEC MC HYD", "bite:cmsc.can"]),
    (
        CMSC,
        "cooling-loss",
        &["therm:cmsc.case", "eicas:ELEC ENGINE STARTER"],
    ),
    (
        CMSC,
        "motor-select-stuck",
        &["eicas:ELEC ENGINE STARTER", "sig:cmsc.motor_current"],
    ),
    (
        CONTACTOR,
        "weld",
        &[
            "sig:contactor.aux",
            "eicas:ELEC GEN SYS",
            "sig:contactor.current_when_open",
        ],
    ),
    (
        CONTACTOR,
        "coil-open",
        &[
            "sig:contactor.aux",
            "eicas:ELEC GEN OFF",
            "sig:contactor.coil_current",
        ],
    ),
    (
        CONTACTOR,
        "aux-stuck",
        &["sig:contactor.aux", "eicas:ELEC STANDBY SYS"],
    ),
    (
        CONTACTOR,
        "high-resistance",
        &["sig:contactor.drop", "therm:contactor.contact"],
    ),
    (
        CONTACTOR,
        "slow-operate",
        &["sig:contactor.aux_delay", "eicas:ELEC GEN SYS"],
    ),
    (
        CONTACTOR,
        "stuck-open",
        &["sig:contactor.aux", "eicas:ELEC GEN OFF"],
    ),
    (ELCC, "stuck-open", &["sig:elcc.aux", "eicas:ELEC STANDBY SYS"]),
    (ELCC, "stuck-closed", &["sig:elcc.aux", "sig:elcc.current"]),
    (ELCC, "nuisance-trip", &["cbic:TRIPPED", "sig:elcc.current"]),
    (ELCC, "fail-to-trip", &["sig:elcc.current", "therm:elcc.contact"]),
    (
        ELCU,
        "power-sense-fail",
        &["bite:elcu.power_sense", "eicas:ELEC STANDBY SYS"],
    ),
    (ELCU, "ttpc-loss", &["bite:elcu.ttpc", "cbic:UNKNOWN"]),
    (ELCU, "internal-fault", &["bite:elcu.fault", "cbic:UNKNOWN"]),
    (
        SSPC,
        "stuck-open",
        &["cbic:OPEN", "sig:sspc.v_out", "sig:sspc.load_current"],
    ),
    (
        SSPC,
        "stuck-closed",
        &["sig:sspc.v_out", "sig:sspc.load_current", "cbic:CLOSED"],
    ),
    (SSPC, "spurious-trip", &["cbic:TRIPPED", "sig:sspc.load_current"]),
    (
        SSPC,
        "arc-detector-false-trip",
        &["cbic:TRIPPED", "sig:sspc.health"],
    ),
    (
        SSPC,
        "telemetry-bias",
        &["sig:sspc.telemetry_current", "sig:sspc.telemetry_voltage"],
    ),
    (
        SSPC,
        "leakage-rise",
        &["sig:sspc.leakage_voltage", "sig:sspc.v_out"],
    ),
    (
        SSPC,
        "fail-to-trip",
        &["therm:sspc.module", "sig:sspc.load_current"],
    ),
    (TCB, "nuisance-trip", &["cbic:TRIPPED", "sig:tcb.state"]),
    (TCB, "stuck-closed", &["sig:tcb.state", "therm:tcb.body"]),
    (TCB, "stuck-open", &["sig:tcb.state", "sig:tcb.load_voltage"]),
    (
        RPDU_CONTROLLER,
        "micro-channel-fail",
        &["eicas:ELEC RPDU CHANNEL", "bite:rpdu.channel"],
    ),
    (
        RPDU_CONTROLLER,
        "gateway-fail",
        &["eicas:ELEC RPDU GATEWAY", "bite:rpdu.gateway"],
    ),
    (
        RPDU_CONTROLLER,
        "backplane-fail",
        &["bite:rpdu.backplane", "eicas:ELEC RPDU CHANNEL"],
    ),
    (
        RPDU_CONTROLLER,
        "comms-loss",
        &["sig:rpdu.default_active", "bite:rpdu.comms"],
    ),
    (
        RPDU_CONTROLLER,
        "power-loss",
        &["sig:rpdu.default_active", "eicas:ELEC RPDU CHANNEL"],
    ),
    (
        RPDU_MODULE,
        "power-module-fail",
        &["eicas:ELEC RPDU CHANNEL", "bite:rpdu.module"],
    ),
    (
        RPDU_MODULE,
        "overtemp",
        &["therm:rpdu.module", "bite:rpdu.module"],
    ),
    (
        RPDU_MODULE,
        "cooling-loss",
        &["therm:rpdu.module", "bite:rpdu.cooling"],
    ),
    (SPDU_CARD, "card-fail", &["bite:spdu.card", "cbic:UNKNOWN"]),
    (SPDU_CARD, "overtemp", &["therm:spdu.card", "bite:spdu.card"]),
    (
        SPDU_CARD,
        "backplane-fail",
        &["bite:spdu.backplane", "cbic:UNKNOWN"],
    ),
    (
        WIRE_SEGMENT,
        "chafe-to-ground",
        &[
            "bite:gcu.differential",
            "cbic:TRIPPED",
            "sig:wire.leakage_current",
        ],
    ),
    (
        WIRE_SEGMENT,
        "open",
        &[
            "sig:wire.load_voltage",
            "sig:sspc.load_current",
            "bite:sspc.load_failed_open",
        ],
    ),
    (
        WIRE_SEGMENT,
        "high-resistance",
        &["therm:wire.conductor", "sig:wire.drop"],
    ),
    (
        WIRE_SEGMENT,
        "short-to-bus",
        &["cbic:TRIPPED", "sig:wire.leakage_current"],
    ),
    (
        WIRE_SEGMENT,
        "arc-tracking",
        &["bite:sspc.arc", "cbic:TRIPPED", "therm:wire.conductor"],
    ),
    (
        CONNECTOR,
        "high-resistance",
        &["sig:connector.drop", "therm:connector.contact"],
    ),
    (
        CONNECTOR,
        "intermittent-open",
        &["sig:connector.drop", "bite:lru.intermittent"],
    ),
    (
        CONNECTOR,
        "pin-short",
        &["cbic:TRIPPED", "sig:connector.leakage_current"],
    ),
    (
        GROUND_STUD,
        "high-resistance",
        &["sig:ground.offset_voltage", "therm:ground.stud"],
    ),
    (
        GROUND_STUD,
        "open",
        &["sig:ground.offset_voltage", "bite:lru.ground_loss"],
    ),
    (
        PANEL_SWITCH,
        "stuck",
        &["sig:switch.position", "lamp:switch-light"],
    ),
    (
        PANEL_SWITCH,
        "contact-bounce",
        &["sig:switch.discrete", "bite:lru.intermittent"],
    ),
    (
        PANEL_SWITCH,
        "open-contact",
        &["sig:switch.discrete", "lamp:switch-light"],
    ),
    (PANEL_SWITCH, "light-element-open", &["lamp:switch-light"]),
    (LAMP, "filament-open", &["lamp:output", "sig:lamp.current"]),
    (LAMP, "dim", &["lamp:output"]),
    (LAMP, "short", &["sig:lamp.current", "cbic:TRIPPED"]),
    (
        EICAS_INPUT,
        "stuck-low",
        &["sig:eicas.discrete", "eicas:false-message"],
    ),
    (
        EICAS_INPUT,
        "stuck-high",
        &["sig:eicas.discrete", "eicas:false-message"],
    ),
    (EICAS_INPUT, "bias", &["sig:eicas.analog", "eicas:false-message"]),
    (
        EICAS_INPUT,
        "intermittent",
        &["sig:eicas.discrete", "bite:lru.intermittent"],
    ),
    (
        OIL_AIR_VALVE,
        "locked-open",
        &["sig:vfsg.oil_temp", "eicas:ELEC GEN AIR/OIL VALVE"],
    ),
    (
        OIL_AIR_VALVE,
        "locked-closed",
        &["sig:vfsg.oil_temp", "eicas:ELEC GEN AIR/OIL VALVE"],
    ),
    (
        OIL_AIR_VALVE,
        "position-sensor-fail",
        &["eicas:ELEC GEN AIR/OIL VALVE", "bite:gcu.valve_sensor"],
    ),
    (
        RAT_HEATER,
        "element-open",
        &["eicas:RAT GEN HEAT", "sig:rat.heater_current"],
    ),
    (
        RAT_HEATER,
        "thermostat-stuck",
        &["sig:rat.heater_current", "therm:rat.gen"],
    ),
    (
        EXT_POWER_RECEPTACLE,
        "pin-ef-open",
        &["lamp:ext-pwr-avail", "sig:ext_power.pin_ef"],
    ),
    (
        EXT_POWER_RECEPTACLE,
        "contact-high-resistance",
        &["sig:ext_power.voltage", "therm:ext_power.contact"],
    ),
    (
        EXT_POWER_RECEPTACLE,
        "chatter",
        &["lamp:ext-pwr-avail", "bite:bpcu.ep_trip"],
    ),
    (
        PECS_PUMP,
        "pump-loss",
        &["therm:pecs.coolant", "eicas:ELEC HVDC SYS"],
    ),
    (
        PECS_PUMP,
        "coolant-temp-rise",
        &["therm:pecs.coolant", "eicas:ELEC HVDC SYS"],
    ),
];

pub fn catalog() -> Registry {
    let mut r = Registry::new();
    for (part, mode, obs) in CATALOG {
        let _ = r.add(part, mode, obs);
    }
    r
}

pub fn part_types() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = Vec::new();
    for (p, _, _) in CATALOG {
        if !v.contains(p) {
            v.push(p);
        }
    }
    v
}
