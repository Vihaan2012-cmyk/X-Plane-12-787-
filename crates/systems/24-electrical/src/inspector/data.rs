use crate::net::Snapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BusView {
    pub name: String,
    pub voltage_v: f64,
    pub live: bool,
    pub freq_hz: f64,
    pub load_a: f64,
    pub fault: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContactorView {
    pub name: String,
    pub closed: bool,
    pub current_a: f64,
    pub fault: bool,
    pub fault_text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceView {
    pub name: String,
    pub online: bool,
    pub voltage_v: f64,
    pub freq_hz: f64,
    pub load_a: f64,
    pub rpm: f64,
    pub status: String,
    pub fault: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConverterView {
    pub name: String,
    pub live: bool,
    pub output_v: f64,
    pub output_a: f64,
    pub power_kw: f64,
    pub fault: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatteryView {
    pub name: String,
    pub voltage_v: f64,
    pub current_a: f64,
    pub soc_pct: f64,
    pub temp_c: f64,
    pub fault: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChannelState {
    #[default]
    Off,
    On,
    Shed,
    Tripped,
    TrippedArc,
}

impl ChannelState {
    pub fn label(self) -> &'static str {
        match self {
            ChannelState::Off => "off",
            ChannelState::On => "on",
            ChannelState::Shed => "shed",
            ChannelState::Tripped => "tripped",
            ChannelState::TrippedArc => "tripped arc",
        }
    }

    pub fn is_fault(self) -> bool {
        matches!(self, ChannelState::Tripped | ChannelState::TrippedArc)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChannelView {
    pub channel: u32,
    pub rating_a: f64,
    pub load: String,
    pub state: ChannelState,
    pub current_a: f64,
    pub voltage_out_v: f64,
    pub i2t_pct: f64,
    pub arc_events: u32,
    pub instant_trip_a: f64,
    pub temp_c: f64,
    pub trips_life: u32,
    pub wire: String,
    pub bundle: String,
    pub wire_temp_c: f64,
    pub history: Vec<(f64, String)>,
    pub trace: Vec<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RpduView {
    pub id: String,
    pub location: String,
    pub input_v: f64,
    pub input_a: f64,
    pub powered: bool,
    pub channels: Vec<ChannelView>,
}

impl RpduView {
    pub fn count(&self, state: ChannelState) -> usize {
        self.channels.iter().filter(|c| c.state == state).count()
    }

    pub fn tripped(&self) -> usize {
        self.channels.iter().filter(|c| c.state.is_fault()).count()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EicasLevel {
    Warning,
    Caution,
    Advisory,
    #[default]
    Status,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EicasMessage {
    pub level: EicasLevel,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorData {
    pub sim_time_s: f64,
    pub buses: Vec<BusView>,
    pub contactors: Vec<ContactorView>,
    pub sources: Vec<SourceView>,
    pub converters: Vec<ConverterView>,
    pub batteries: Vec<BatteryView>,
    pub rpdus: Vec<RpduView>,
    pub eicas: Vec<EicasMessage>,
    pub protections: Vec<ProtectionRow>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProtectionRow {
    pub group: String,
    pub name: String,
    pub state: String,
    pub tripped: bool,
    pub detail: String,
}

fn alias(net_name: &str) -> &str {
    match net_name {
        "EXT_L EPC" => "L EPC",
        "L1 TRUC" => "L TRU RLY",
        other => other,
    }
}

impl InspectorData {
    pub fn bus(&self, name: &str) -> Option<&BusView> {
        self.buses.iter().find(|b| b.name == name)
    }

    pub fn contactor(&self, name: &str) -> Option<&ContactorView> {
        self.contactors.iter().find(|c| c.name == name)
    }

    pub fn source(&self, name: &str) -> Option<&SourceView> {
        self.sources.iter().find(|s| s.name == name)
    }

    pub fn converter(&self, name: &str) -> Option<&ConverterView> {
        self.converters.iter().find(|s| s.name == name)
    }

    pub fn battery(&self, name: &str) -> Option<&BatteryView> {
        self.batteries.iter().find(|s| s.name == name)
    }

    pub fn rpdu(&self, id: &str) -> Option<&RpduView> {
        self.rpdus.iter().find(|r| r.id == id)
    }

    pub fn bus_live(&self, name: &str) -> bool {
        self.bus(name).is_some_and(|b| b.live)
    }

    pub fn closed(&self, name: &str) -> bool {
        self.contactor(name).is_some_and(|c| c.closed)
    }

    pub fn from_net_snapshot(s: &Snapshot) -> InspectorData {
        let mut d = InspectorData::default();
        for b in &s.buses {
            d.buses.push(BusView {
                name: b.name.to_string(),
                voltage_v: b.voltage_v,
                live: b.live,
                freq_hz: b.freq_hz,
                load_a: 0.0,
                fault: false,
            });
        }
        for c in &s.contactors {
            d.contactors.push(ContactorView {
                name: alias(&c.name.to_string()).to_string(),
                closed: c.closed,
                current_a: c.current_a,
                fault: false,
                fault_text: String::new(),
            });
        }
        for (src, breaker, bus) in [
            ("VFSG_L1", "L1 GCB", "L1"),
            ("VFSG_L2", "L2 GCB", "L2"),
            ("VFSG_R1", "R1 GCB", "R1"),
            ("VFSG_R2", "R2 GCB", "R2"),
            ("ASG_L", "L APB", "LTB"),
            ("ASG_R", "R APB", "RTB"),
        ] {
            let closed = d.closed(breaker);
            let (live, v, f) = d
                .bus(bus)
                .map(|b| (b.live, b.voltage_v, b.freq_hz))
                .unwrap_or((false, 0.0, 0.0));
            let amps = d.contactor(breaker).map_or(0.0, |c| c.current_a);
            d.sources.push(SourceView {
                name: src.to_string(),
                online: closed && live,
                voltage_v: v,
                freq_hz: f,
                load_a: amps,
                rpm: 0.0,
                status: if closed && live { "online" } else { "off" }.to_string(),
                fault: false,
            });
        }
        let ext = d.closed("L EPC");
        d.sources.push(SourceView {
            name: "EXT_L_FWD".to_string(),
            online: ext,
            voltage_v: d.bus("AC115_L").map_or(0.0, |b| b.voltage_v),
            freq_hz: d.bus("AC115_L").map_or(0.0, |b| b.freq_hz),
            load_a: d.contactor("L EPC").map_or(0.0, |c| c.current_a),
            rpm: 0.0,
            status: if ext { "online" } else { "off" }.to_string(),
            fault: false,
        });
        for (name, bus) in [("TRU_L", "DC28_L"), ("ATU_L", "ATU_L")] {
            let b = d.bus(bus).cloned().unwrap_or_default();
            d.converters.push(ConverterView {
                name: name.to_string(),
                live: b.live,
                output_v: b.voltage_v,
                output_a: b.load_a,
                power_kw: b.voltage_v * b.load_a / 1000.0,
                fault: false,
            });
        }
        let hbb = d.bus("HBB_MAIN").cloned().unwrap_or_default();
        d.batteries.push(BatteryView {
            name: "BATT_MAIN".to_string(),
            voltage_v: hbb.voltage_v,
            current_a: s.battery_current_a,
            soc_pct: 0.0,
            temp_c: 0.0,
            fault: false,
        });
        if !s.converged {
            d.eicas.push(EicasMessage {
                level: EicasLevel::Caution,
                text: "ELEC SOLVER NOT CONVERGED".to_string(),
            });
        }
        d
    }
}
