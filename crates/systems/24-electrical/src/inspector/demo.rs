use super::data::{
    BatteryView, BusView, ChannelState, ChannelView, ContactorView, ConverterView, EicasLevel, EicasMessage,
    InspectorData, RpduView, SourceView,
};
use super::topo::{BARS, BOXES, CONTACTORS};

const LOADS: [(f64, &str, ChannelState, f64); 30] = [
    (5.0, "window heat ctl 1", ChannelState::On, 1.8),
    (5.0, "window heat ctl 2", ChannelState::On, 1.7),
    (7.5, "wiper L motor", ChannelState::Off, 0.0),
    (7.5, "wiper R motor", ChannelState::Off, 0.0),
    (3.0, "ISS 1 power", ChannelState::On, 1.1),
    (3.0, "WXR R/T", ChannelState::On, 2.2),
    (10.0, "pitot heat L", ChannelState::On, 6.3),
    (10.0, "pitot heat R", ChannelState::On, 6.5),
    (5.0, "AOA heat L", ChannelState::On, 2.9),
    (5.0, "AOA heat R", ChannelState::On, 3.0),
    (5.0, "TAT probe heat", ChannelState::On, 1.4),
    (10.0, "pitot heat C", ChannelState::On, 6.4),
    (15.0, "PDOS pump L", ChannelState::Off, 0.0),
    (15.0, "PDOS pump R", ChannelState::Off, 0.0),
    (25.0, "galley 1 chiller", ChannelState::Shed, 0.0),
    (25.0, "galley 2 chiller", ChannelState::Shed, 0.0),
    (3.0, "FD door lock", ChannelState::On, 0.4),
    (3.0, "EFB DU L", ChannelState::On, 1.9),
    (5.0, "landing light nose", ChannelState::TrippedArc, 0.0),
    (5.0, "taxi light", ChannelState::Off, 0.0),
    (5.0, "logo light L", ChannelState::Off, 0.0),
    (5.0, "logo light R", ChannelState::Off, 0.0),
    (3.0, "printer", ChannelState::On, 0.3),
    (3.0, "cabin sign ctl", ChannelState::On, 0.6),
    (10.0, "lav water heater", ChannelState::On, 4.8),
    (5.0, "CSS FAP", ChannelState::On, 1.2),
    (3.0, "SATCOM HPA ctl", ChannelState::On, 0.9),
    (7.5, "recirc fan fwd", ChannelState::On, 5.1),
    (3.0, "crew rest smoke det", ChannelState::On, 0.2),
    (5.0, "cabin lights ctl", ChannelState::On, 0.7),
];

fn channel(n: u32) -> ChannelView {
    let (rating, load, state, a) = if (n as usize) <= LOADS.len() {
        LOADS[n as usize - 1]
    } else {
        (2.5, "spare", ChannelState::Off, 0.0)
    };
    let trace: Vec<f32> = (0..120)
        .map(|i| {
            let wob = ((i * 37 + n * 11) % 17) as f32 / 17.0 - 0.5;
            if state == ChannelState::On {
                a as f32 + wob * 0.08 * a as f32
            } else {
                0.0
            }
        })
        .collect();
    ChannelView {
        channel: n,
        rating_a: rating,
        load: load.to_string(),
        state,
        current_a: a,
        voltage_out_v: if state == ChannelState::On { 114.9 } else { 0.0 },
        i2t_pct: if state == ChannelState::On {
            a / rating * 56.0
        } else {
            0.0
        },
        arc_events: if state == ChannelState::TrippedArc { 1 } else { 0 },
        instant_trip_a: rating * 8.0,
        temp_c: 34.0,
        trips_life: if state.is_fault() { 1 } else { 0 },
        wire: "18 AWG, 14.2 m, fwd E/E to nose".to_string(),
        bundle: "W5013 zone 110 to 120".to_string(),
        wire_temp_c: 27.0,
        history: vec![
            (2011.6, "on (air, icing)".to_string()),
            (1842.0, "low-power (ground)".to_string()),
            (0.0, "power-up BIT pass".to_string()),
        ],
        trace,
    }
}

impl InspectorData {
    pub fn demo() -> InspectorData {
        let mut d = InspectorData {
            sim_time_s: 2843.112,
            ..InspectorData::default()
        };
        let dead = ["EXT_AFT", "EXT_L_FWD", "EXT_R_FWD", "RAT_GEN", "TRU_R"];
        let open = [
            "L3 BTB",
            "L2 BTB",
            "L1 BTB",
            "R1 BTB",
            "R2 BTB",
            "R3 BTB",
            "L APB",
            "R APB",
            "L EPC",
            "R EPC",
            "L BTB 115",
            "R BTB 115",
            "LAEPC",
            "RCB",
            "BB RLY",
            "BB ISO RLY",
            "C1 TRU RELAY",
            "C2 TRU RELAY",
            "CIT",
            "FIT",
            "SPUB",
            "SPUC",
            "R TRU RLY",
        ];
        let off_buses = ["LTB", "RTB", "BB"];
        for b in BARS.iter() {
            let live = !off_buses.contains(&b.id);
            let (v, f) = match b.unit {
                "AC" if b.id.starts_with("AC115") => (115.2, 400.0),
                "AC" => (235.1, 398.2),
                "HV" => (540.0, 0.0),
                _ => (27.9, 0.0),
            };
            d.buses.push(BusView {
                name: b.id.to_string(),
                voltage_v: if live { v } else { 0.0 },
                live,
                freq_hz: if live { f } else { 0.0 },
                load_a: if live { 41.0 } else { 0.0 },
                fault: false,
            });
        }
        for k in CONTACTORS.iter() {
            let closed = !open.contains(&k.name);
            d.contactors.push(ContactorView {
                name: k.name.to_string(),
                closed,
                current_a: if closed { 37.5 } else { 0.0 },
                fault: k.name == "R TRU RLY",
                fault_text: if k.name == "R TRU RLY" {
                    "failed open".to_string()
                } else {
                    String::new()
                },
            });
        }
        for b in BOXES.iter() {
            let online = !dead.contains(&b.id) && !b.id.starts_with("BATT");
            if b.id.starts_with("VFSG")
                || b.id.starts_with("ASG")
                || b.id.starts_with("EXT")
                || b.id == "RAT_GEN"
            {
                let on = online && !b.id.starts_with("ASG");
                d.sources.push(SourceView {
                    name: b.id.to_string(),
                    online: on,
                    voltage_v: if on { 235.1 } else { 0.0 },
                    freq_hz: if on { 398.2 } else { 0.0 },
                    load_a: if on { 131.0 } else { 0.0 },
                    rpm: if on { 7964.0 } else { 0.0 },
                    status: if on { "online" } else { "off" }.to_string(),
                    fault: false,
                });
            } else if b.id.starts_with("BATT") {
                d.batteries.push(BatteryView {
                    name: b.id.to_string(),
                    voltage_v: 29.4,
                    current_a: 3.1,
                    soc_pct: 96.0,
                    temp_c: 31.0,
                    fault: false,
                });
            } else {
                d.converters.push(ConverterView {
                    name: b.id.to_string(),
                    live: online,
                    output_v: if b.id.starts_with("ATRU") || b.id.starts_with("CMSC") {
                        540.0
                    } else if b.id.starts_with("ATU") {
                        115.2
                    } else {
                        27.9
                    },
                    output_a: 40.0,
                    power_kw: if b.id.starts_with("ATRU") { 12.1 } else { 0.0 },
                    fault: false,
                });
            }
        }
        d.eicas.push(EicasMessage {
            level: EicasLevel::Caution,
            text: "ELEC TRU R".to_string(),
        });
        d.eicas.push(EicasMessage {
            level: EicasLevel::Warning,
            text: "ELEC RPDU 75 CHANNEL".to_string(),
        });
        d.eicas.push(EicasMessage {
            level: EicasLevel::Status,
            text: "ELEC BUS TIES OPEN".to_string(),
        });
        let mut r = RpduView {
            id: "RPDU 75".to_string(),
            location: "fwd EE bay".to_string(),
            input_v: 115.2,
            input_a: 61.4,
            powered: true,
            channels: (1..=48).map(channel).collect(),
        };
        r.channels[11].state = ChannelState::On;
        d.rpdus.push(r);
        for id in ["RPDU 71", "RPDU 73"] {
            d.rpdus.push(RpduView {
                id: id.to_string(),
                location: "fwd EE bay".to_string(),
                input_v: 115.1,
                input_a: 20.0,
                powered: true,
                channels: (1..=19).map(channel).collect(),
            });
        }
        d
    }
}
