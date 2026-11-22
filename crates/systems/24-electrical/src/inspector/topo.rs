pub const VW: f32 = 1000.0;
pub const VH: f32 = 672.0;

pub struct BoxDef {
    pub id: &'static str,
    pub label: &'static str,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct BarDef {
    pub id: &'static str,
    pub x0: f32,
    pub x1: f32,
    pub y: f32,
    pub unit: &'static str,
}

pub struct ConDef {
    pub name: &'static str,
    pub a: &'static str,
    pub b: &'static str,
    pub panel: &'static str,
    pub x: f32,
    pub y: f32,
}

pub struct Wire {
    pub pts: &'static [(f32, f32)],
    pub live: &'static str,
}

const fn bx(id: &'static str, label: &'static str, x: f32, y: f32, w: f32, h: f32) -> BoxDef {
    BoxDef {
        id,
        label,
        x,
        y,
        w,
        h,
    }
}

const fn bar(id: &'static str, x0: f32, x1: f32, y: f32, unit: &'static str) -> BarDef {
    BarDef { id, x0, x1, y, unit }
}

const fn con(
    name: &'static str,
    a: &'static str,
    b: &'static str,
    panel: &'static str,
    x: f32,
    y: f32,
) -> ConDef {
    ConDef {
        name,
        a,
        b,
        panel,
        x,
        y,
    }
}

const fn wire(pts: &'static [(f32, f32)], live: &'static str) -> Wire {
    Wire { pts, live }
}

pub const BOXES: [BoxDef; 31] = [
    bx("VFSG_L1", "VFSG L1", 37.0, 12.0, 96.0, 34.0),
    bx("VFSG_L2", "VFSG L2", 187.0, 12.0, 96.0, 34.0),
    bx("ASG_L", "ASG L", 357.0, 12.0, 96.0, 34.0),
    bx("ASG_R", "ASG R", 547.0, 12.0, 96.0, 34.0),
    bx("VFSG_R1", "VFSG R1", 717.0, 12.0, 96.0, 34.0),
    bx("VFSG_R2", "VFSG R2", 867.0, 12.0, 96.0, 34.0),
    bx("ATRU_L1", "ATRU L1", 20.0, 206.0, 66.0, 30.0),
    bx("ATU_L", "ATU L", 108.0, 206.0, 42.0, 30.0),
    bx("ATRU_L2", "ATRU L2", 190.0, 206.0, 66.0, 30.0),
    bx("EXT_AFT", "EXT AFT", 300.0, 206.0, 80.0, 30.0),
    bx("EXT_L_FWD", "EXT L FWD", 395.0, 206.0, 80.0, 30.0),
    bx("EXT_R_FWD", "EXT R FWD", 525.0, 206.0, 80.0, 30.0),
    bx("ATU_R", "ATU R", 700.0, 206.0, 42.0, 30.0),
    bx("ATRU_R1", "ATRU R1", 764.0, 206.0, 66.0, 30.0),
    bx("ATRU_R2", "ATRU R2", 864.0, 206.0, 66.0, 30.0),
    bx("CMSC_CAC_L1", "CAC L1", 20.0, 276.0, 34.0, 20.0),
    bx("CMSC_EMP_L", "EMP L", 56.0, 276.0, 34.0, 20.0),
    bx("CMSC_CAC_L2", "CAC L2", 180.0, 276.0, 44.0, 20.0),
    bx("CMSC_EMP_C1", "EMP C1", 226.0, 276.0, 44.0, 20.0),
    bx("CMSC_CAC_R1", "CAC R1", 755.0, 276.0, 40.0, 20.0),
    bx("CMSC_EMP_R", "EMP R", 797.0, 276.0, 40.0, 20.0),
    bx("CMSC_CAC_R2", "CAC R2", 860.0, 276.0, 44.0, 20.0),
    bx("CMSC_EMP_C2", "EMP C2", 906.0, 276.0, 44.0, 20.0),
    bx("RAT_GEN", "RAT GEN", 455.0, 350.0, 90.0, 26.0),
    bx("TRU_L", "TRU L", 68.0, 440.0, 60.0, 28.0),
    bx("TRU_C1", "TRU C1", 415.0, 440.0, 60.0, 28.0),
    bx("TRU_C2", "TRU C2", 525.0, 440.0, 60.0, 28.0),
    bx("TRU_R", "TRU R", 718.0, 440.0, 60.0, 28.0),
    bx("SPU", "SPU", 860.0, 560.0, 70.0, 28.0),
    bx("BATT_MAIN", "MAIN BATT", 370.0, 622.0, 100.0, 30.0),
    bx("BATT_APU", "APU BATT", 720.0, 622.0, 100.0, 30.0),
];

pub const BARS: [BarDef; 19] = [
    bar("L1", 20.0, 150.0, 150.0, "AC"),
    bar("L2", 170.0, 300.0, 150.0, "AC"),
    bar("LTB", 345.0, 465.0, 150.0, "AC"),
    bar("RTB", 535.0, 655.0, 150.0, "AC"),
    bar("R1", 700.0, 830.0, 150.0, "AC"),
    bar("R2", 850.0, 980.0, 150.0, "AC"),
    bar("HVDC_L1", 20.0, 90.0, 262.0, "HV"),
    bar("HVDC_L2", 180.0, 270.0, 262.0, "HV"),
    bar("HVDC_R1", 755.0, 840.0, 262.0, "HV"),
    bar("HVDC_R2", 860.0, 950.0, 262.0, "HV"),
    bar("AC115_L", 110.0, 450.0, 330.0, "AC"),
    bar("AC115_R", 550.0, 910.0, 330.0, "AC"),
    bar("BB", 440.0, 565.0, 404.0, "AC"),
    bar("DC28_L", 20.0, 300.0, 520.0, "DC"),
    bar("CIB", 330.0, 480.0, 520.0, "DC"),
    bar("FIB", 520.0, 670.0, 520.0, "DC"),
    bar("DC28_R", 700.0, 980.0, 520.0, "DC"),
    bar("HBB_MAIN", 330.0, 480.0, 580.0, "DC"),
    bar("HBB_APU", 700.0, 820.0, 580.0, "DC"),
];

pub const CONTACTORS: [ConDef; 40] = [
    con("L1 GCB", "VFSG_L1", "L1", "P100", 85.0, 82.0),
    con("L2 GCB", "VFSG_L2", "L2", "P100", 235.0, 82.0),
    con("L APB", "ASG_L", "LTB", "P150", 405.0, 82.0),
    con("R APB", "ASG_R", "RTB", "P150", 595.0, 82.0),
    con("R1 GCB", "VFSG_R1", "R1", "P200", 765.0, 82.0),
    con("R2 GCB", "VFSG_R2", "R2", "P200", 915.0, 82.0),
    con("L3 BTB", "L1", "L2", "P100", 160.0, 150.0),
    con("L2 BTB", "L2", "LTB", "P100", 322.0, 150.0),
    con("L1 BTB", "L1", "RTB", "P100", 330.0, 132.0),
    con("R1 BTB", "R1", "LTB", "P200", 520.0, 118.0),
    con("R2 BTB", "R2", "RTB", "P200", 720.0, 104.0),
    con("R3 BTB", "R1", "R2", "P200", 840.0, 150.0),
    con("L1 ATRUC", "L1", "ATRU_L1", "P100", 53.0, 180.0),
    con("L ATUC", "L1", "ATU_L", "P100", 129.0, 180.0),
    con("L2 ATRUC", "L2", "ATRU_L2", "P100", 223.0, 180.0),
    con("R ATUC", "R1", "ATU_R", "P200", 721.0, 180.0),
    con("R1 ATRUC", "R1", "ATRU_R1", "P200", 797.0, 180.0),
    con("R2 ATRUC", "R2", "ATRU_R2", "P200", 897.0, 180.0),
    con("LAEPC", "EXT_AFT", "ATRU_L2", "P700", 278.0, 221.0),
    con("L BSB", "ATU_L", "AC115_L", "P300", 129.0, 283.0),
    con("R BSB", "ATU_R", "AC115_R", "P400", 721.0, 283.0),
    con("L EPC", "EXT_L_FWD", "AC115_L", "P300", 435.0, 283.0),
    con("R EPC", "EXT_R_FWD", "AC115_R", "P400", 565.0, 283.0),
    con("L BTB 115", "AC115_L", "AC115_R", "P300", 475.0, 330.0),
    con("R BTB 115", "AC115_R", "AC115_L", "P400", 525.0, 330.0),
    con("L TRU RLY", "L1", "TRU_L", "P300", 98.0, 410.0),
    con("R TRU RLY", "R1", "TRU_R", "P400", 748.0, 410.0),
    con("RCB", "RAT_GEN", "BB", "P400", 500.0, 390.0),
    con("BB RLY", "R2", "BB", "P200", 950.0, 404.0),
    con("BB ISO RLY", "BB", "R2", "P400", 900.0, 404.0),
    con("C1 TRU RELAY", "BB", "TRU_C1", "P400", 445.0, 424.0),
    con("C2 TRU RELAY", "BB", "TRU_C2", "P400", 555.0, 424.0),
    con("L DCT", "DC28_L", "DC28_R", "P300", 350.0, 500.0),
    con("R DCT", "DC28_R", "DC28_L", "P400", 650.0, 500.0),
    con("CIT", "CIB", "FIB", "P300", 490.0, 520.0),
    con("FIT", "FIB", "CIB", "P400", 510.0, 520.0),
    con("MBR", "HBB_MAIN", "CIB", "P300", 420.0, 548.0),
    con("BATT RLY", "BATT_MAIN", "HBB_MAIN", "P300", 420.0, 603.0),
    con("SPUC", "BATT_APU", "SPU", "P49", 840.0, 574.0),
    con("SPUB", "SPU", "ATRU_R2", "P800", 968.0, 300.0),
];

pub const WIRES: [Wire; 54] = [
    wire(&[(85.0, 46.0), (85.0, 150.0)], "L1"),
    wire(&[(235.0, 46.0), (235.0, 150.0)], "L2"),
    wire(&[(405.0, 46.0), (405.0, 150.0)], "LTB"),
    wire(&[(595.0, 46.0), (595.0, 150.0)], "RTB"),
    wire(&[(765.0, 46.0), (765.0, 150.0)], "R1"),
    wire(&[(915.0, 46.0), (915.0, 150.0)], "R2"),
    wire(&[(150.0, 150.0), (170.0, 150.0)], "L1"),
    wire(&[(300.0, 150.0), (345.0, 150.0)], "L2"),
    wire(&[(830.0, 150.0), (850.0, 150.0)], "R1"),
    wire(
        &[(30.0, 150.0), (30.0, 132.0), (545.0, 132.0), (545.0, 150.0)],
        "L1",
    ),
    wire(
        &[(820.0, 150.0), (820.0, 118.0), (455.0, 118.0), (455.0, 150.0)],
        "R1",
    ),
    wire(
        &[(860.0, 150.0), (860.0, 104.0), (645.0, 104.0), (645.0, 150.0)],
        "R2",
    ),
    wire(&[(53.0, 150.0), (53.0, 206.0)], "L1"),
    wire(&[(129.0, 150.0), (129.0, 206.0)], "L1"),
    wire(&[(223.0, 150.0), (223.0, 206.0)], "L2"),
    wire(&[(721.0, 150.0), (721.0, 206.0)], "R1"),
    wire(&[(797.0, 150.0), (797.0, 206.0)], "R1"),
    wire(&[(897.0, 150.0), (897.0, 206.0)], "R2"),
    wire(&[(53.0, 236.0), (53.0, 262.0)], "HVDC_L1"),
    wire(&[(223.0, 236.0), (223.0, 262.0)], "HVDC_L2"),
    wire(&[(797.0, 236.0), (797.0, 262.0)], "HVDC_R1"),
    wire(&[(897.0, 236.0), (897.0, 262.0)], "HVDC_R2"),
    wire(&[(37.0, 262.0), (37.0, 276.0)], "HVDC_L1"),
    wire(&[(73.0, 262.0), (73.0, 276.0)], "HVDC_L1"),
    wire(&[(202.0, 262.0), (202.0, 276.0)], "HVDC_L2"),
    wire(&[(248.0, 262.0), (248.0, 276.0)], "HVDC_L2"),
    wire(&[(775.0, 262.0), (775.0, 276.0)], "HVDC_R1"),
    wire(&[(817.0, 262.0), (817.0, 276.0)], "HVDC_R1"),
    wire(&[(882.0, 262.0), (882.0, 276.0)], "HVDC_R2"),
    wire(&[(928.0, 262.0), (928.0, 276.0)], "HVDC_R2"),
    wire(&[(300.0, 221.0), (256.0, 221.0)], "EXT_AFT"),
    wire(&[(129.0, 236.0), (129.0, 330.0)], "ATU_L"),
    wire(&[(721.0, 236.0), (721.0, 330.0)], "ATU_R"),
    wire(&[(435.0, 236.0), (435.0, 330.0)], "EXT_L_FWD"),
    wire(&[(565.0, 236.0), (565.0, 330.0)], "EXT_R_FWD"),
    wire(&[(450.0, 330.0), (550.0, 330.0)], "AC115_L"),
    wire(&[(98.0, 150.0), (98.0, 440.0)], "L1"),
    wire(&[(748.0, 150.0), (748.0, 440.0)], "R1"),
    wire(&[(500.0, 376.0), (500.0, 404.0)], "RAT_GEN"),
    wire(&[(992.0, 404.0), (565.0, 404.0)], "R2"),
    wire(&[(980.0, 150.0), (992.0, 150.0), (992.0, 404.0)], "R2"),
    wire(&[(445.0, 404.0), (445.0, 440.0)], "BB"),
    wire(&[(555.0, 404.0), (555.0, 440.0)], "BB"),
    wire(&[(98.0, 468.0), (98.0, 520.0)], "DC28_L"),
    wire(&[(445.0, 468.0), (445.0, 520.0)], "CIB"),
    wire(&[(555.0, 468.0), (555.0, 520.0)], "FIB"),
    wire(&[(748.0, 468.0), (748.0, 520.0)], "DC28_R"),
    wire(
        &[(290.0, 520.0), (290.0, 500.0), (710.0, 500.0), (710.0, 520.0)],
        "DC28_L",
    ),
    wire(&[(480.0, 520.0), (520.0, 520.0)], "CIB"),
    wire(&[(420.0, 520.0), (420.0, 580.0)], "HBB_MAIN"),
    wire(&[(420.0, 580.0), (420.0, 622.0)], "BATT_MAIN"),
    wire(&[(770.0, 580.0), (770.0, 622.0)], "BATT_APU"),
    wire(&[(820.0, 574.0), (860.0, 574.0)], "HBB_APU"),
    wire(
        &[(930.0, 574.0), (968.0, 574.0), (968.0, 222.0), (930.0, 222.0)],
        "SPU",
    ),
];

pub struct PanelDef {
    pub id: &'static str,
    pub place: &'static str,
    pub text: &'static [&'static str],
    pub buses: &'static [&'static str],
    pub units: &'static [&'static str],
}

pub const PANELS: [PanelDef; 10] = [
    PanelDef {
        id: "P100",
        place: "aft EE bay",
        text: &[
            "L 235 VAC power",
            "L1+L2 GCU, ELCU, 2 OVTPU",
            "GCB BTB SC GNR GEC",
            "no TCBs",
        ],
        buses: &["L1", "L2", "LTB"],
        units: &[],
    },
    PanelDef {
        id: "P150",
        place: "aft EE bay",
        text: &["APU panel", "L/R AGCU, APB", "APU SC GNR GEC", "no ELCU"],
        buses: &[],
        units: &["ASG_L", "ASG_R"],
    },
    PanelDef {
        id: "P200",
        place: "aft EE bay",
        text: &[
            "R 235 VAC power",
            "R1+R2 GCU, ELCU, 2 OVTPU",
            "GCB BTB SC GNR GEC",
            "no TCBs",
        ],
        buses: &["R1", "R2", "RTB"],
        units: &[],
    },
    PanelDef {
        id: "P300",
        place: "fwd EE bay",
        text: &[
            "115 VAC, 28 VDC, BPCU",
            "ELCU, SPDU, TCBs, ELCCs",
            "WH1 heater control",
        ],
        buses: &["AC115_L", "DC28_L", "CIB", "HBB_MAIN"],
        units: &[],
    },
    PanelDef {
        id: "P400",
        place: "fwd EE bay",
        text: &["115 VAC, 28 VDC, backup bus", "BPCU, ELCU, SPDU, TCBs", "RAT GCU"],
        buses: &["AC115_R", "DC28_R", "FIB", "BB"],
        units: &["RAT_GEN"],
    },
    PanelDef {
        id: "P500",
        place: "conversion",
        text: &["ATU L, TRU L, TRU C1", "EBPSU LI, RI"],
        buses: &[],
        units: &["ATU_L", "TRU_L", "TRU_C1"],
    },
    PanelDef {
        id: "P600",
        place: "conversion",
        text: &["ATU R, TRU C2, TRU R", "EBPSU LO, RO"],
        buses: &[],
        units: &["ATU_R", "TRU_C2", "TRU_R"],
    },
    PanelDef {
        id: "P700",
        place: "aft EE bay",
        text: &["HVDC L: ATRU L1 L2", "4 CMSC, RFMC", "6 contactors"],
        buses: &["HVDC_L1", "HVDC_L2"],
        units: &[
            "ATRU_L1",
            "ATRU_L2",
            "CMSC_CAC_L1",
            "CMSC_EMP_L",
            "CMSC_CAC_L2",
            "CMSC_EMP_C1",
        ],
    },
    PanelDef {
        id: "P800",
        place: "aft EE bay",
        text: &["HVDC R: ATRU R1 R2", "4 CMSC, RFMC", "6 contactors"],
        buses: &["HVDC_R1", "HVDC_R2"],
        units: &[
            "ATRU_R1",
            "ATRU_R2",
            "CMSC_CAC_R1",
            "CMSC_EMP_R",
            "CMSC_CAC_R2",
            "CMSC_EMP_C2",
        ],
    },
    PanelDef {
        id: "P49",
        place: "aft EE bay, below P150",
        text: &[
            "APU start / aux",
            "APU HBB, RAT deploy relay",
            "TCBs not monitored",
        ],
        buses: &["HBB_APU"],
        units: &["BATT_APU", "SPU"],
    },
];
