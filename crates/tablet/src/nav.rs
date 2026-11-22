#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavGroup {
    pub name: &'static str,
    pub items: &'static [NavItem],
}

const fn item(id: &'static str, label: &'static str) -> NavItem {
    NavItem { id, label }
}

pub const NAV: &[NavGroup] = &[
    NavGroup {
        name: "POWER",
        items: &[
            item("electrical", "Electrical"),
            item("electrical.overhead", "  overhead panel"),
            item("electrical.synoptic", "  synoptic"),
            item("electrical.eicas", "  EICAS and CBIC"),
            item("electrical.panels", "  power panels"),
            item("electrical.rpdu", "  RPDUs"),
            item("electrical.breaker", "  breaker"),
            item("electrical.protection", "  protection"),
            item("engines", "Engines"),
            item("apu", "APU"),
            item("fuel", "Fuel"),
            item("hydraulics", "Hydraulics"),
        ],
    },
    NavGroup {
        name: "FLIGHT",
        items: &[
            item("flight-controls", "Flight controls"),
            item("gear-brakes", "Gear, brakes"),
            item("autoflight", "Autoflight"),
            item("fms", "FMS"),
        ],
    },
    NavGroup {
        name: "ENVIRONMENT",
        items: &[
            item("air-pressurisation", "Air, pressurisation"),
            item("ice-rain", "Ice and rain"),
            item("fire", "Fire"),
        ],
    },
    NavGroup {
        name: "AVIONICS",
        items: &[
            item("air-data", "Air data, inertial"),
            item("nav-radios", "Navigation radios"),
            item("surveillance", "Surveillance"),
            item("comms", "Communications"),
            item("displays-alerting", "Displays, alerting"),
            item("ccs", "CCS computers"),
            item("cdn", "CDN network"),
        ],
    },
    NavGroup {
        name: "CABIN",
        items: &[
            item("lights", "Lights"),
            item("doors-oxygen-water", "Doors, oxygen, water"),
            item("efb", "EFB, info systems"),
        ],
    },
];
