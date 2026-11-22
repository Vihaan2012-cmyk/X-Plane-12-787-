mod csv;
mod overlay;

use crate::sources::{MachineInput, MachineParams, SyncMachine};
use crate::storage::{BatteryPack, BatteryParams, MAX_CELLS};
use csv::Table;
use solver_electrical::{Element, ElementId, LoadKind, Network, NodeId, NodeKind, Solver};
use std::collections::BTreeMap;
use std::path::Path;

const R_SWITCH: f64 = 1.0e-4;
const R_NEUTRAL: f64 = 1.0e-3;
const R_OFF: f64 = 1.0e6;
const V_PHASE: f64 = 235.0;
pub(crate) const V_EXT_PHASE: f64 = 115.0;
pub(crate) const EXT_FREQ_HZ: f64 = 400.0;
const EXT_R: f64 = 0.005;
const EXT_X: f64 = 0.005;
const APPROX_HVDC_CIRCUIT_W: f64 = 100.0;
const APPROX_HBB_STANDBY_W: f64 = 1.0;
const APPROX_DC_BREAKER_LOAD_FRACTION: f64 = 0.1;
pub const CCS_STANDIN_LOAD: &str = "STANDIN_CCS_L";
pub const APPROX_CCS_STANDIN_W: f64 = 220.0;
const APPROX_AC_BREAKER_LOAD_FRACTION: f64 = 0.02;
const APPROX_INSTRUMENT_BREAKER_LOAD_FRACTION: f64 = 0.01;
const APPROX_UNRATED_BREAKER_A: f64 = 5.0;

#[derive(Clone, Debug, PartialEq)]
pub struct BreakerCircuit {
    pub id: String,
    pub kind: String,
    pub rating_a: f64,
    pub load: String,
    pub load_index: usize,
    pub bus: String,
    pub sspc: bool,
}
const APPROX_115_CIRCUIT_W: f64 = 25.0;
const APPROX_DC_CIRCUIT_W: f64 = 10.0;
const APPROX_INSTRUMENT_CIRCUIT_W: f64 = 3.0;
const APPROX_235_CIRCUIT_W: f64 = 100.0;
const APPROX_MAX_CIRCUIT_W: f64 = 20_000.0;
pub const MANAGED_LOAD_BUSES: [(&str, [&str; 2]); 14] = [
    ("GALLEY_FWD", ["L2", "R2"]),
    ("GALLEY_AFT", ["L1", "R1"]),
    ("IFE", ["AC115_L", "AC115_R"]),
    ("CABIN_LIGHTING", ["AC115_L", "AC115_R"]),
    ("CARGO_HEAT", ["L2", "R2"]),
    ("RECIRC_FANS", ["L1", "R1"]),
    ("RAM_FAN", ["L2", "R2"]),
    ("WINDOW_HEAT", ["L1", "R1"]),
    ("ECS_PACKS", ["", ""]),
    ("FUEL_PUMPS_CTR", ["L2", "R2"]),
    ("HYD_DEMAND_PUMPS", ["", ""]),
    ("EXTERIOR_LIGHTS", ["AC115_L", "AC115_R"]),
    ("COMM_NAV", ["", ""]),
    ("FLIGHT_DISPLAYS_ESSENTIAL", ["", ""]),
];
pub const LEAK_ZONES: [&str; 4] = ["LTB", "RTB", "ATU_L", "ATU_R"];
const TRU_OUT_V: f64 = 28.6;
const TRU_R_OUT: f64 = 0.01;
const APPROX_BDM_TRU_LIVE_V: f64 = 24.0;
const TRU_EFFICIENCY: f64 = 0.92;
const ATRU_OUT_V: f64 = 270.0;
const ATRU_NO_LOAD_V: f64 = 280.0;
const ATRU_R_OUT: f64 = 0.0175;
const ATRU_EFFICIENCY: f64 = 0.97;
const ATU_EFFICIENCY: f64 = 0.97;
const SPU_OUT_V: f64 = 270.0;
const SPU_R_OUT: f64 = 0.05;
const SPU_EFFICIENCY: f64 = 0.85;
const SPU_MIN_IN_V: f64 = 18.0;
const AMBIENT_C: f64 = 25.0;
const OIL_C: f64 = 60.0;
const APPROX_DC_CHANNEL_W: f64 = 15.0;
const APPROX_AC_CHANNEL_W: f64 = 60.0;
const DIODE_PASSES: usize = 6;
const DIODE_MARGIN_DC: f64 = 0.05;
const DIODE_MARGIN_AC: f64 = 1.0;
const SQRT3: f64 = 1.732_050_807_568_877_2;

#[derive(Clone, Debug, PartialEq)]
pub struct BusReading {
    pub name: String,
    pub voltage_v: f64,
    pub live: bool,
    pub freq_hz: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContactorReading {
    pub name: String,
    pub closed: bool,
    pub current_a: f64,
    pub phase_a: [f64; 3],
    pub residual: (f64, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct MachineReading {
    pub name: String,
    pub freq_hz: f64,
    pub terminal_v: f64,
    pub current_a: f64,
    pub field_pu: f64,
    pub phase_a: [f64; 3],
    pub phase_v: [f64; 3],
    pub residual_a: f64,
    pub residual: (f64, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatteryReading {
    pub name: String,
    pub voltage_v: f64,
    pub current_a: f64,
    pub soc: f64,
    pub temp_k: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadReading {
    pub name: String,
    pub voltage_v: f64,
    pub current_a: f64,
    pub power_w: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SspcReading {
    pub id: String,
    pub rpdu: String,
    pub voltage_v: f64,
    pub current_a: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConverterReading {
    pub name: String,
    pub output_v: f64,
    pub output_a: f64,
    pub output_w: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub buses: Vec<BusReading>,
    pub contactors: Vec<ContactorReading>,
    pub machines: Vec<MachineReading>,
    pub batteries: Vec<BatteryReading>,
    pub loads: Vec<LoadReading>,
    pub sspcs: Vec<SspcReading>,
    pub converters: Vec<ConverterReading>,
    pub node_leak_a: Vec<(String, f64)>,
    pub battery_current_a: f64,
    pub converged: bool,
    pub outer_iterations: usize,
    pub outer_residual: f64,
    pub inner_residual: f64,
}

impl Snapshot {
    pub fn bus(&self, name: &str) -> Option<&BusReading> {
        self.buses.iter().find(|b| b.name == name)
    }

    pub fn contactor(&self, name: &str) -> Option<&ContactorReading> {
        self.contactors.iter().find(|c| c.name == name)
    }

    pub fn machine(&self, name: &str) -> Option<&MachineReading> {
        self.machines.iter().find(|m| m.name == name)
    }

    pub fn battery(&self, name: &str) -> Option<&BatteryReading> {
        self.batteries.iter().find(|b| b.name == name)
    }

    pub fn load(&self, name: &str) -> Option<&LoadReading> {
        self.loads.iter().find(|l| l.name == name)
    }

    pub fn leak_a(&self, node: &str) -> f64 {
        self.node_leak_a
            .iter()
            .find(|(n, _)| n == node)
            .map(|(_, a)| *a)
            .unwrap_or(0.0)
    }

    pub fn converter(&self, name: &str) -> Option<&ConverterReading> {
        self.converters.iter().find(|c| c.name == name)
    }

    pub fn sspc(&self, id: &str) -> Option<&SspcReading> {
        self.sspcs.iter().find(|s| s.id == id)
    }
}

struct Generator {
    name: String,
    machine: SyncMachine,
    element: ElementId,
    node: NodeId,
    rpm: f64,
    field_pu: f64,
    exciter_a: Option<f64>,
    oil_inlet_c: f64,
}

struct Contactor {
    name: String,
    element: Option<ElementId>,
    ac: bool,
    closed: bool,
}

struct Diode {
    switch: ElementId,
    inner: NodeId,
    far: NodeId,
    ac: bool,
    closed: bool,
    bdm: bool,
}

struct Load {
    name: String,
    element: ElementId,
    node: NodeId,
    ac: bool,
    v_nominal: f64,
    demand_w: f64,
    enabled: bool,
}

struct Sspc {
    id: String,
    rpdu: String,
    node: NodeId,
    load: Option<usize>,
    switch: Option<ElementId>,
    closed: bool,
}

struct Ext {
    name: String,
    element: ElementId,
    node: NodeId,
    connected: bool,
}

struct Batt {
    name: String,
    pack: BatteryPack,
    element: ElementId,
    node: NodeId,
    current_a: f64,
    charge_a: f64,
}

struct Spu {
    source: ElementId,
    source_node: NodeId,
    input: ElementId,
    input_node: NodeId,
    contactor: usize,
    enabled: bool,
    out_w: f64,
}

pub struct ElectricalNet {
    network: Network,
    solver: Solver,
    nodes: BTreeMap<String, NodeId>,
    bus_list: Vec<(String, NodeId)>,
    contactors: Vec<Contactor>,
    generators: Vec<Generator>,
    exts: Vec<Ext>,
    batteries: Vec<Batt>,
    diodes: Vec<Diode>,
    loads: Vec<Load>,
    sspcs: Vec<Sspc>,
    spu: Option<Spu>,
    contactor_names: Vec<String>,
    bus_names: Vec<String>,
    machine_names: Vec<String>,
    load_names: Vec<String>,
    sspc_ids: Vec<String>,
    conversion_kinds: Vec<String>,
    tru_outputs: Vec<NodeId>,
    rectifiers: Vec<(String, ElementId, NodeId, Option<NodeId>)>,
    leak_zones: Vec<(String, NodeId)>,
    circuits: Vec<(String, String)>,
    breaker_circuits: Vec<BreakerCircuit>,
    mbr: Option<usize>,
    bdm_block: bool,
    overlay: overlay::Overlay,
}

struct Builder {
    net: Network,
    nodes: BTreeMap<String, NodeId>,
    bus_list: Vec<(String, NodeId)>,
    kinds: BTreeMap<usize, NodeKind>,
    contactors: Vec<Contactor>,
    generators: Vec<Generator>,
    exts: Vec<Ext>,
    batteries: Vec<Batt>,
    diodes: Vec<Diode>,
    loads: Vec<Load>,
    sspcs: Vec<Sspc>,
    tru_outputs: Vec<NodeId>,
}

fn err<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}

impl Builder {
    fn node(&mut self, name: &str, ac: bool, bus: bool) -> NodeId {
        if let Some(n) = self.nodes.get(name) {
            return *n;
        }
        let kind = if ac { NodeKind::Ac3 } else { NodeKind::Dc };
        let id = self.net.add_node(kind);
        self.kinds.insert(id.0, kind);
        self.nodes.insert(name.to_string(), id);
        if bus {
            self.bus_list.push((name.to_string(), id));
        }
        id
    }

    fn is_ac(&self, n: NodeId) -> bool {
        self.kinds.get(&n.0) == Some(&NodeKind::Ac3)
    }

    fn add(&mut self, e: Element) -> Result<ElementId, String> {
        self.net.add_element(e).map_err(err)
    }

    fn switch(&mut self, a: NodeId, b: NodeId, closed: bool) -> Result<ElementId, String> {
        self.add(Element::Switch {
            a,
            b,
            closed,
            r_closed: R_SWITCH,
        })
    }

    fn diode(&mut self, inner: NodeId, far: NodeId) -> Result<(), String> {
        let ac = self.is_ac(inner);
        let switch = self.switch(inner, far, false)?;
        self.diodes.push(Diode {
            switch,
            inner,
            far,
            ac,
            closed: false,
            bdm: false,
        });
        Ok(())
    }

    fn load(
        &mut self,
        name: &str,
        node: NodeId,
        v_nominal: f64,
        demand_w: f64,
        enabled: bool,
    ) -> Result<usize, String> {
        let ac = self.is_ac(node);
        let element = self.add(load_element(
            node,
            ac,
            v_nominal,
            if enabled { demand_w } else { 0.0 },
        ))?;
        self.loads.push(Load {
            name: name.to_string(),
            element,
            node,
            ac,
            v_nominal,
            demand_w,
            enabled,
        });
        Ok(self.loads.len() - 1)
    }

    fn lookup(&self, name: &str) -> Option<NodeId> {
        let src = format!("SRC_{name}");
        self.nodes.get(&src).or_else(|| self.nodes.get(name)).copied()
    }
}

fn load_element(node: NodeId, ac: bool, v_nominal: f64, w: f64) -> Element {
    if ac {
        Element::balanced_ac_load(
            LoadKind::Power {
                p: w / 3.0,
                q: 0.0,
                v_nominal,
            },
            node,
        )
    } else {
        Element::DcLoad {
            node,
            kind: LoadKind::Power {
                p: w,
                q: 0.0,
                v_nominal,
            },
        }
    }
}

fn rpdu_number(text: &str) -> Option<String> {
    let t = text.trim();
    let rest = t.strip_prefix("RPDU")?.trim();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() { None } else { Some(digits) }
}

fn name_rpdu_number(name: &str) -> Option<String> {
    let idx = name.find("RPDU")?;
    rpdu_number(&name[idx..])
}

fn parse_watts(text: &str) -> Option<f64> {
    let t = text.trim();
    let digits: String = t
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let value: f64 = digits.parse().ok()?;
    let rest = t[digits.len()..].trim().to_ascii_lowercase();
    match rest.as_str() {
        "w" => Some(value),
        "kw" => Some(value * 1000.0),
        _ => None,
    }
}

fn is_ac_text(text: &str) -> bool {
    let t = text.to_ascii_lowercase();
    t.contains("115") || t.contains("235") || t.contains("v ac")
}

fn panel_bus(voltage: &str, unit: &str, k: usize) -> Option<&'static str> {
    let v = voltage.to_ascii_lowercase();
    let u = unit.trim().to_ascii_uppercase();
    let left = match u.as_str() {
        "P100" | "P300" | "P500" => true,
        "P200" | "P400" | "P600" => false,
        _ => k.is_multiple_of(2),
    };
    let pick4 = |names: [&'static str; 4]| {
        let base = if left { 0 } else { 2 };
        names[base + (k / 2) % 2]
    };
    if v.contains("270") {
        return Some(pick4(["HVDC_L1", "HVDC_L2", "HVDC_R1", "HVDC_R2"]));
    }
    if v.contains("235") || v.contains("230") {
        return Some(pick4(["L1", "L2", "R1", "R2"]));
    }
    if v.contains("115") || v.contains("v ac") || v.contains("vac") {
        return Some(if left { "AC115_L" } else { "AC115_R" });
    }
    if v.contains("apu") && (v.contains("hbb") || v.contains("hot")) || u == "P49" {
        return Some("HBB_APU");
    }
    if v.contains("hbb") || v.contains("hot batt") {
        return Some("HBB_MAIN");
    }
    if v.contains("instrument") {
        return Some(if left { "CIB" } else { "FIB" });
    }
    if v.contains("28") || v.contains("dc") {
        return Some(if left { "DC28_L" } else { "DC28_R" });
    }
    match u.as_str() {
        "P300" => Some("DC28_L"),
        "P400" => Some("DC28_R"),
        "SPDU" | "P700" => Some(if left { "DC28_L" } else { "DC28_R" }),
        _ => None,
    }
}

fn approx_draw_w(bus: &str) -> f64 {
    if bus.starts_with("HBB") {
        APPROX_HBB_STANDBY_W
    } else if bus == "CIB" || bus == "FIB" {
        APPROX_INSTRUMENT_CIRCUIT_W
    } else if bus.starts_with("HVDC") {
        APPROX_HVDC_CIRCUIT_W
    } else if matches!(bus, "L1" | "L2" | "R1" | "R2") {
        APPROX_235_CIRCUIT_W
    } else if bus.starts_with("AC115") {
        APPROX_115_CIRCUIT_W
    } else {
        APPROX_DC_CIRCUIT_W
    }
}

fn side_left(num: &str) -> bool {
    num.parse::<u32>().map(|n| n % 2 == 1).unwrap_or(true)
}

impl ElectricalNet {
    pub fn default_data_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("aircraft")
            .join("24-electrical")
    }

    pub fn new() -> Self {
        match Self::from_data(&Self::default_data_dir()) {
            Ok(n) => n,
            Err(e) => panic!("electrical net data: {e}"),
        }
    }

    pub fn from_data(dir: &Path) -> Result<ElectricalNet, String> {
        let sources = Table::load(dir, "sources.csv")?;
        let buses = Table::load(dir, "buses.csv")?;
        let conversion = Table::load(dir, "conversion.csv")?;
        let contactors = Table::load(dir, "contactors.csv")?;
        let breakers = Table::load(dir, "breakers.csv")?;
        let loads = Table::load(dir, "loads.csv")?;
        let rpdus = Table::load(dir, "rpdus.csv")?;
        let mut b = Builder {
            net: Network::new(),
            nodes: BTreeMap::new(),
            bus_list: Vec::new(),
            kinds: BTreeMap::new(),
            contactors: Vec::new(),
            generators: Vec::new(),
            exts: Vec::new(),
            batteries: Vec::new(),
            diodes: Vec::new(),
            loads: Vec::new(),
            sspcs: Vec::new(),
            tru_outputs: Vec::new(),
        };
        for row in &buses.rows {
            let id = buses.col(row, "id");
            let kind = buses.col(row, "kind");
            b.node(&id, kind == "AC3", true);
        }
        b.node("BBX", true, false);
        let mut spu = None;
        for row in &sources.rows {
            let id = sources.col(row, "id");
            let kind = sources.col(row, "kind");
            let bus = sources.col(row, "bus_fed");
            if kind == "VFSG" || kind == "ASG" || kind == "RAT generator" {
                let params = match kind.as_str() {
                    "VFSG" => MachineParams::vfsg(),
                    "ASG" => MachineParams::asg(),
                    _ => MachineParams::rat(),
                };
                let node = b.node(&format!("SRC_{id}"), true, false);
                let element = b.add(Element::AcSource {
                    node,
                    emf: 0.0,
                    angle: 0.0,
                    r: R_OFF,
                    x: 0.0,
                    frequency_hz: 400.0,
                    r_neutral_ground: R_NEUTRAL,
                })?;
                let name = if id == "RAT_GEN" {
                    "RAT".to_string()
                } else {
                    id.clone()
                };
                b.generators.push(Generator {
                    name,
                    machine: SyncMachine::new(params, OIL_C),
                    element,
                    node,
                    rpm: 0.0,
                    field_pu: 1.0,
                    exciter_a: None,
                    oil_inlet_c: OIL_C,
                });
            } else if kind.contains("battery") {
                let node = b.node(&format!("SRC_{id}"), false, false);
                let pack = BatteryPack::new(
                    BatteryParams::new(8, 50.0),
                    b.batteries.len() as u64 + 1,
                    0.9,
                    AMBIENT_C,
                );
                let (e, r) = pack.thevenin();
                let element = b.add(Element::DcSource { node, e, r })?;
                let far = b.lookup(&bus).ok_or_else(|| format!("battery bus {bus}"))?;
                b.diode(node, far)?;
                if id == "BATT_MAIN"
                    && let Some(d) = b.diodes.last_mut()
                {
                    d.bdm = true;
                }
                b.batteries.push(Batt {
                    name: id.clone(),
                    pack,
                    element,
                    node,
                    current_a: 0.0,
                    charge_a: 0.0,
                });
            } else if kind.contains("external") {
                let node = b.node(&id, true, false);
                let element = b.add(Element::AcSource {
                    node,
                    emf: 0.0,
                    angle: 0.0,
                    r: R_OFF,
                    x: EXT_X,
                    frequency_hz: EXT_FREQ_HZ,
                    r_neutral_ground: R_NEUTRAL,
                })?;
                let name = match id.as_str() {
                    "EXT_L_FWD" => "L FWD",
                    "EXT_R_FWD" => "R FWD",
                    _ => "AFT",
                };
                b.exts.push(Ext {
                    name: name.to_string(),
                    element,
                    node,
                    connected: false,
                });
            }
        }
        let mut cmsc_rows = Vec::new();
        for row in &conversion.rows {
            let id = conversion.col(row, "id");
            let kind = conversion.col(row, "kind");
            let out_bus = conversion.col(row, "output_bus");
            match kind.as_str() {
                "ATRU" => {
                    let input = b.node(&id, true, false);
                    let inner = b.node(&format!("{id}_OUT"), false, false);
                    let far = b.lookup(&out_bus).ok_or_else(|| format!("atru bus {out_bus}"))?;
                    b.add(Element::Rectifier {
                        ac: input,
                        pos: inner,
                        neg: None,
                        ratio: ATRU_NO_LOAD_V / (V_PHASE * SQRT3),
                        r_out: ATRU_R_OUT,
                        efficiency: ATRU_EFFICIENCY,
                        ac_v_nominal: V_PHASE,
                    })?;
                    b.diode(inner, far)?;
                    if id == "ATRU_L2" {
                        let in115 = b.node("ATRU_L2_115", true, false);
                        let inner2 = b.node("ATRU_L2_115_OUT", false, false);
                        b.add(Element::Rectifier {
                            ac: in115,
                            pos: inner2,
                            neg: None,
                            ratio: ATRU_NO_LOAD_V / (V_EXT_PHASE * SQRT3),
                            r_out: ATRU_R_OUT,
                            efficiency: ATRU_EFFICIENCY,
                            ac_v_nominal: V_EXT_PHASE,
                        })?;
                        b.diode(inner2, far)?;
                    }
                }
                "TRU" => {
                    let input = b.node(&id, true, false);
                    let inner = b.node(&format!("{id}_OUT"), false, false);
                    let far = b.lookup(&out_bus).ok_or_else(|| format!("tru bus {out_bus}"))?;
                    b.add(Element::Rectifier {
                        ac: input,
                        pos: inner,
                        neg: None,
                        ratio: TRU_OUT_V / (V_PHASE * SQRT3),
                        r_out: TRU_R_OUT,
                        efficiency: TRU_EFFICIENCY,
                        ac_v_nominal: V_PHASE,
                    })?;
                    b.diode(inner, far)?;
                    b.tru_outputs.push(inner);
                    if id == "TRU_C2" {
                        let bb = b.lookup("BB").ok_or("BB")?;
                        b.switch(bb, input, true)?;
                    }
                }
                "ATU" => {
                    let side = b.node(&id, true, true);
                    let n115 = b.node(&format!("{id}_115"), true, false);
                    let up = b.node(&format!("{id}_UP"), true, false);
                    let dn = b.node(&format!("{id}_DN"), true, false);
                    for (input, output, ratio, nominal) in [
                        (n115, up, V_PHASE / V_EXT_PHASE, V_EXT_PHASE),
                        (side, dn, V_EXT_PHASE / V_PHASE, V_PHASE),
                    ] {
                        b.add(Element::Autotransformer {
                            input,
                            output,
                            ratio,
                            r_leak: 0.01,
                            x_leak: 0.02,
                            efficiency: ATU_EFFICIENCY,
                            in_v_nominal: nominal,
                            r_neutral_ground: R_NEUTRAL,
                        })?;
                    }
                    b.diode(up, side)?;
                    b.diode(dn, n115)?;
                }
                "SPU" => {
                    let source_node = b.node("SPU_SRC", false, false);
                    let input_node = b.node("SPU_IN", false, false);
                    let out = b.node("SPU_OUT", false, false);
                    let far = b.lookup("HVDC_R2").ok_or("HVDC_R2")?;
                    let source = b.add(Element::DcSource {
                        node: source_node,
                        e: 0.0,
                        r: R_OFF,
                    })?;
                    let input = b.add(load_element(input_node, false, 28.0, 0.0))?;
                    b.diode(out, far)?;
                    spu = Some(Spu {
                        source,
                        source_node,
                        input,
                        input_node,
                        contactor: usize::MAX,
                        enabled: false,
                        out_w: 0.0,
                    });
                }
                "CMSC" => cmsc_rows.push((id, conversion.col(row, "input_bus"))),
                _ => {}
            }
        }
        for (id, bus) in &cmsc_rows {
            if let Some(n) = b.lookup(bus) {
                b.load(id, n, ATRU_OUT_V, 0.0, true)?;
            }
        }
        let rpdu_ids: Vec<(String, bool)> = rpdus
            .rows
            .iter()
            .filter_map(|r| {
                let n = rpdu_number(&rpdus.col(r, "id"))?;
                Some((n, rpdus.col(r, "feeds").contains("115 VAC")))
            })
            .collect();
        for (num, has_ac) in &rpdu_ids {
            b.node(&format!("RPDU {num} DC"), false, true);
            if *has_ac {
                b.node(&format!("RPDU {num} AC"), true, true);
            }
        }
        let mut seen_names: BTreeMap<String, usize> = BTreeMap::new();
        let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
        for row in &contactors.rows {
            let id = contactors.col(row, "id");
            let mut name = contactors.col(row, "name");
            if name.is_empty() {
                name = id.clone();
            }
            rows.push((
                id,
                name,
                contactors.col(row, "connects_a"),
                contactors.col(row, "connects_b"),
                contactors.col(row, "type"),
            ));
        }
        for row in &breakers.rows {
            let name = breakers.col(row, "name");
            if name.starts_with("AC FEED RPDU") && !rows.iter().any(|r| r.1 == name) {
                rows.push((
                    breakers.col(row, "id"),
                    name,
                    String::new(),
                    String::new(),
                    "contactor".to_string(),
                ));
            }
        }
        for (num, has_ac) in &rpdu_ids {
            let name = format!("AC FEED RPDU {num}");
            if *has_ac && !rows.iter().any(|r| r.1 == name) {
                rows.push((
                    format!("ELCC_{num}"),
                    name,
                    String::new(),
                    String::new(),
                    "contactor".to_string(),
                ));
            }
        }
        for (id, name, a_name, b_name, typ) in rows {
            let mut name = name;
            if seen_names.contains_key(&name) {
                name = format!("{name} {id}");
            }
            seen_names.insert(name.clone(), 0);
            let (mut a_node, mut b_node) = (b.lookup(&a_name), b.lookup(&b_name));
            match name.as_str() {
                "BB RLY" => b_node = b.nodes.get("BBX").copied(),
                "BB ISO RLY" => {
                    a_node = b.nodes.get("BBX").copied();
                    b_node = b.nodes.get("BB").copied();
                }
                "LAEPC" => b_node = b.nodes.get("ATRU_L2_115").copied(),
                "L BSB" => a_node = b.nodes.get("ATU_L_115").copied(),
                "R BSB" => a_node = b.nodes.get("ATU_R_115").copied(),
                "SPUB" => {
                    a_node = b.nodes.get("SPU_SRC").copied();
                    b_node = b.nodes.get("SPU_OUT").copied();
                }
                "SPU contactor" => {
                    a_node = b.nodes.get("HBB_APU").copied();
                    b_node = b.nodes.get("SPU_IN").copied();
                }
                _ => {}
            }
            if let Some(num) = name.strip_prefix("AC FEED RPDU ").map(str::trim) {
                let side = if side_left(num) { "AC115_L" } else { "AC115_R" };
                a_node = b.nodes.get(side).copied();
                b_node = b.nodes.get(&format!("RPDU {num} AC")).copied();
            }
            let default_closed = typ.contains("NC");
            let mut element = None;
            let mut ac = false;
            if let (Some(a), Some(bn)) = (a_node, b_node)
                && b.is_ac(a) == b.is_ac(bn)
            {
                ac = b.is_ac(a);
                element = Some(b.switch(a, bn, default_closed)?);
            }
            if name == "SPU contactor"
                && let Some(s) = spu.as_mut()
            {
                s.contactor = b.contactors.len();
            }
            b.contactors.push(Contactor {
                name,
                element,
                ac,
                closed: default_closed,
            });
        }
        for row in &breakers.rows {
            let name = breakers.col(row, "name");
            let id = breakers.col(row, "id");
            let (main, standby) = (
                name.starts_with("MAIN DC FEED RPDU"),
                name.starts_with("STANDBY DC FEED RPDU"),
            );
            if !(main || standby) {
                continue;
            }
            let Some(num) = name_rpdu_number(&name) else {
                continue;
            };
            let left = side_left(&num);
            let src = match (standby, left) {
                (false, true) => "DC28_L",
                (false, false) => "DC28_R",
                (true, true) => "CIB",
                (true, false) => "FIB",
            };
            let (Some(from), Some(to)) = (
                b.nodes.get(src).copied(),
                b.nodes.get(&format!("RPDU {num} DC")).copied(),
            ) else {
                continue;
            };
            let feed = b.node(
                &format!("RPDU {num} {}", if standby { "STBY" } else { "MAIN" }),
                false,
                false,
            );
            let sw = b.switch(from, feed, true)?;
            b.diode(feed, to)?;
            b.sspcs.push(Sspc {
                id,
                rpdu: format!("RPDU {num}"),
                node: to,
                load: None,
                switch: Some(sw),
                closed: true,
            });
        }
        let mut per_rpdu: BTreeMap<String, usize> = BTreeMap::new();
        let mut used: BTreeMap<String, usize> = BTreeMap::new();
        for row in &loads.rows {
            let Some(num) = rpdu_number(&loads.col(row, "unit_rpdu")) else {
                continue;
            };
            let ac_node = b.nodes.get(&format!("RPDU {num} AC")).copied();
            let dc_node = b.nodes.get(&format!("RPDU {num} DC")).copied();
            let want_ac = is_ac_text(&loads.col(row, "voltage_bus"));
            let (node, v_nom) = match (want_ac, ac_node, dc_node) {
                (true, Some(n), _) => (n, V_EXT_PHASE),
                (_, _, Some(n)) => (n, 28.0),
                _ => continue,
            };
            let ac = b.is_ac(node);
            let w = parse_watts(&loads.col(row, "draw")).unwrap_or(if ac {
                APPROX_AC_CHANNEL_W
            } else {
                APPROX_DC_CHANNEL_W
            });
            let mut name = loads.col(row, "load");
            if name.is_empty() {
                name = "channel".to_string();
            }
            if used.contains_key(&name) {
                name = format!("{name} @ RPDU {num}");
            }
            let mut unique = name.clone();
            let mut k = 2;
            while used.contains_key(&unique) {
                unique = format!("{name} #{k}");
                k += 1;
            }
            used.insert(unique.clone(), 0);
            let idx = per_rpdu.entry(num.clone()).or_insert(0);
            *idx += 1;
            let sspc_id = format!("RPDU{num}.{:02}", *idx);
            let li = b.load(&unique, node, v_nom, w, false)?;
            b.sspcs.push(Sspc {
                id: sspc_id,
                rpdu: format!("RPDU {num}"),
                node,
                load: Some(li),
                switch: None,
                closed: false,
            });
        }
        for (name, buses) in MANAGED_LOAD_BUSES {
            for (side, bus) in buses.iter().enumerate() {
                if bus.is_empty() {
                    continue;
                }
                let Some(node) = b.nodes.get(*bus).copied() else {
                    continue;
                };
                let v_nom = if b.is_ac(node) {
                    if bus.starts_with("AC115") {
                        V_EXT_PHASE
                    } else {
                        V_PHASE
                    }
                } else {
                    28.0
                };
                b.load(&format!("LM_{name}_{}", ["L", "R"][side]), node, v_nom, 0.0, true)?;
            }
        }
        let mut panel_added: BTreeMap<&'static str, f64> = BTreeMap::new();
        let mut circuits: Vec<(String, String)> = Vec::new();
        let mut breaker_circuits: Vec<BreakerCircuit> = Vec::new();
        for row in &breakers.rows {
            let id = breakers.col(row, "id");
            let name = breakers.col(row, "name");
            let kind = breakers.col(row, "prefix_type");
            let bus = breakers.col(row, "bus");
            let feed = name.contains("FEED RPDU") || name.contains("FEED - RPDU");
            if feed || bus.is_empty() || b.contactors.iter().any(|c| c.name == name) {
                continue;
            }
            let Some(node) = b.nodes.get(&bus).copied() else {
                continue;
            };
            let rating = breakers
                .col(row, "rating_a")
                .parse::<f64>()
                .unwrap_or(APPROX_UNRATED_BREAKER_A);
            let ac = b.is_ac(node);
            let v_nom = if ac {
                if bus.starts_with("AC115") {
                    V_EXT_PHASE
                } else {
                    V_PHASE
                }
            } else {
                28.0
            };
            let w = if bus.starts_with("HBB") {
                APPROX_HBB_STANDBY_W
            } else if bus == "CIB" || bus == "FIB" {
                APPROX_INSTRUMENT_BREAKER_LOAD_FRACTION * rating * v_nom
            } else if ac {
                APPROX_AC_BREAKER_LOAD_FRACTION * rating * v_nom * 3.0
            } else {
                APPROX_DC_BREAKER_LOAD_FRACTION * rating * v_nom
            };
            let load = format!("CB {id} {name}");
            let li = b.load(&load, node, v_nom, w, true)?;
            let sspc = kind == "ECB"
                && !breakers.col(row, "card_module").is_empty()
                && !breakers.col(row, "channel").is_empty();
            if sspc {
                b.sspcs.push(Sspc {
                    id: id.clone(),
                    rpdu: if bus.ends_with('R') || bus == "FIB" {
                        "R SPDU"
                    } else {
                        "L SPDU"
                    }
                    .to_string(),
                    node,
                    load: Some(li),
                    switch: None,
                    closed: true,
                });
            }
            circuits.push((load.clone(), bus.clone()));
            breaker_circuits.push(BreakerCircuit {
                id,
                kind,
                rating_a: rating,
                load,
                load_index: li,
                bus,
                sspc,
            });
        }
        for (k, row) in loads.rows.iter().enumerate() {
            if rpdu_number(&loads.col(row, "unit_rpdu")).is_some() {
                continue;
            }
            let Some(bus) = panel_bus(&loads.col(row, "voltage_bus"), &loads.col(row, "unit_rpdu"), k) else {
                continue;
            };
            let Some(node) = b.nodes.get(bus).copied() else {
                continue;
            };
            let w = parse_watts(&loads.col(row, "draw"))
                .filter(|w| *w < APPROX_MAX_CIRCUIT_W)
                .unwrap_or_else(|| approx_draw_w(bus));
            let mut name = loads.col(row, "load");
            if name.is_empty() {
                name = format!("circuit {k}");
            }
            let mut unique = name.clone();
            let mut n = 2;
            while used.contains_key(&unique) {
                unique = format!("{name} #{n}");
                n += 1;
            }
            used.insert(unique.clone(), 0);
            let v_nom = if b.is_ac(node) {
                if bus.starts_with("AC115") {
                    V_EXT_PHASE
                } else {
                    V_PHASE
                }
            } else if bus.starts_with("HVDC") {
                ATRU_OUT_V
            } else {
                28.0
            };
            b.load(&unique, node, v_nom, w, true)?;
            *panel_added.entry(bus).or_insert(0.0) += w;
            circuits.push((unique, bus.to_string()));
        }
        for (name, w) in [
            ("L1", 0.0),
            ("L2", 0.0),
            ("R1", 0.0),
            ("R2", 0.0),
            ("BB", 0.0),
            ("AC115_L", 0.0),
            ("AC115_R", 0.0),
            ("DC28_L", 560.0),
            ("DC28_R", 0.0),
            ("CIB", 0.0),
            ("FIB", 0.0),
            ("HBB_MAIN", 60.0),
            ("HBB_APU", 0.0),
        ] {
            if let Some(n) = b.nodes.get(name).copied() {
                let w = (w - panel_added.get(name).copied().unwrap_or(0.0)).max(0.0);
                let vn = if b.is_ac(n) {
                    if name.starts_with("AC115") {
                        V_EXT_PHASE
                    } else {
                        V_PHASE
                    }
                } else {
                    28.0
                };
                b.load(&format!("LOAD_{name}"), n, vn, w, true)?;
            }
        }
        if let Some(n) = b.nodes.get("CIB").copied() {
            b.load(CCS_STANDIN_LOAD, n, 28.0, APPROX_CCS_STANDIN_W, true)?;
        }
        let contactor_names = b.contactors.iter().map(|c| c.name.clone()).collect();
        let mbr = b.contactors.iter().position(|c| c.name == "MBR");
        let mut bus_names: Vec<String> = b.bus_list.iter().map(|x| x.0.clone()).collect();
        bus_names.sort();
        let machine_names = b.generators.iter().map(|g| g.name.clone()).collect();
        let load_names = b.loads.iter().map(|l| l.name.clone()).collect();
        let sspc_ids = b.sspcs.iter().map(|s| s.id.clone()).collect();
        let conversion_kinds = conversion
            .rows
            .iter()
            .map(|r| conversion.col(r, "kind"))
            .collect();
        let mut rectifiers = Vec::new();
        for i in 0..b.net.element_count() {
            if let Element::Rectifier { ac, pos, neg, .. } = *b.net.element(ElementId(i))
                && let Some((name, _)) = b.nodes.iter().find(|(_, n)| **n == ac)
            {
                rectifiers.push((name.clone(), ElementId(i), pos, neg));
            }
        }
        let leak_zones = LEAK_ZONES
            .iter()
            .filter_map(|z| b.nodes.get(*z).map(|n| (z.to_string(), *n)))
            .collect();
        Ok(ElectricalNet {
            network: b.net,
            solver: Solver::default(),
            nodes: b.nodes,
            bus_list: b.bus_list,
            contactors: b.contactors,
            generators: b.generators,
            exts: b.exts,
            batteries: b.batteries,
            diodes: b.diodes,
            loads: b.loads,
            sspcs: b.sspcs,
            spu,
            contactor_names,
            bus_names,
            machine_names,
            load_names,
            sspc_ids,
            conversion_kinds,
            tru_outputs: b.tru_outputs,
            rectifiers,
            leak_zones,
            circuits,
            breaker_circuits,
            mbr,
            bdm_block: false,
            overlay: overlay::Overlay::default(),
        })
    }

    pub fn contactor_names(&self) -> &[String] {
        &self.contactor_names
    }

    pub fn bus_names(&self) -> &[String] {
        &self.bus_names
    }

    pub fn machine_names(&self) -> &[String] {
        &self.machine_names
    }

    pub fn load_names(&self) -> &[String] {
        &self.load_names
    }

    pub fn sspc_ids(&self) -> &[String] {
        &self.sspc_ids
    }

    pub fn conversion_count(&self, kind: &str) -> usize {
        self.conversion_kinds
            .iter()
            .filter(|k| k.as_str() == kind)
            .count()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn battery(&self) -> &BatteryPack {
        &self.batteries[0].pack
    }

    pub fn battery_apu(&self) -> &BatteryPack {
        &self.batteries[self.batteries.len() - 1].pack
    }

    pub fn battery_pack_mut(&mut self, index: usize) -> Option<&mut BatteryPack> {
        let last = self.batteries.len().checked_sub(1)?;
        let i = if index == 0 { 0 } else { last };
        self.batteries.get_mut(i).map(|b| &mut b.pack)
    }

    pub fn set_machine_oil_inlet_c(&mut self, name: &str, oil_c: f64) -> bool {
        match self.generators.iter_mut().find(|g| g.name == name) {
            Some(g) => {
                g.oil_inlet_c = oil_c;
                true
            }
            None => false,
        }
    }

    pub fn machine_oil_temp_c(&self, name: &str) -> Option<f64> {
        self.generators
            .iter()
            .find(|g| g.name == name)
            .map(|g| g.machine.oil_temp_c())
    }

    pub fn machine_disconnected(&self, name: &str) -> bool {
        self.generators
            .iter()
            .find(|g| g.name == name)
            .is_some_and(|g| g.machine.disconnected())
    }

    pub fn set_contactor_resistance_ohm(&mut self, name: &str, r_closed: f64) -> bool {
        let Some(c) = self.contactors.iter().find(|c| c.name == name) else {
            return false;
        };
        let Some(e) = c.element else {
            return false;
        };
        let Element::Switch { a, b, closed, .. } = *self.network.element(e) else {
            return false;
        };
        self.network
            .set_element(
                e,
                Element::Switch {
                    a,
                    b,
                    closed,
                    r_closed: r_closed.max(1.0e-6),
                },
            )
            .is_ok()
    }

    pub fn set_contactor(&mut self, name: &str, closed: bool) -> bool {
        let Some(i) = self.contactors.iter().position(|c| c.name == name) else {
            return false;
        };
        self.contactors[i].closed = closed;
        match self.contactors[i].element {
            Some(e) => self.network.set_switch(e, closed).is_ok(),
            None => true,
        }
    }

    pub fn set_machine_rpm(&mut self, name: &str, rpm: f64) -> bool {
        match self.generators.iter_mut().find(|g| g.name == name) {
            Some(g) => {
                g.rpm = rpm.max(0.0);
                true
            }
            None => false,
        }
    }

    pub fn set_machine_field(&mut self, name: &str, field_pu: f64) -> bool {
        match self.generators.iter_mut().find(|g| g.name == name) {
            Some(g) => {
                g.field_pu = field_pu.max(0.0);
                true
            }
            None => false,
        }
    }

    pub fn set_machine_exciter(&mut self, name: &str, exciter_a: f64) -> bool {
        match self.generators.iter_mut().find(|g| g.name == name) {
            Some(g) => {
                g.exciter_a = Some(exciter_a.max(0.0));
                true
            }
            None => false,
        }
    }

    pub fn set_battery_charge(&mut self, index: usize, charge_a: f64) -> bool {
        match self.batteries.get_mut(index) {
            Some(b) => {
                b.charge_a = charge_a.max(0.0);
                true
            }
            None => false,
        }
    }

    pub fn set_external_power(&mut self, receptacle: &str, connected: bool) -> bool {
        match self.exts.iter_mut().find(|e| e.name == receptacle) {
            Some(e) => {
                e.connected = connected;
                true
            }
            None => false,
        }
    }

    pub fn breaker_circuits(&self) -> &[BreakerCircuit] {
        &self.breaker_circuits
    }

    pub fn sspc_load_names(&self) -> Vec<Option<String>> {
        self.sspcs
            .iter()
            .map(|s| s.load.map(|li| self.loads[li].name.clone()))
            .collect()
    }

    pub fn topology(&self) -> Vec<[String; 5]> {
        let names: BTreeMap<usize, &str> = self.nodes.iter().map(|(n, id)| (id.0, n.as_str())).collect();
        let nm = |id: NodeId| names.get(&id.0).copied().unwrap_or("?").to_string();
        let mut rows = Vec::new();
        let mut named = std::collections::BTreeSet::new();
        for c in &self.contactors {
            if let Some(e) = c.element {
                named.insert(e.0);
                if let Element::Switch { a, b, .. } = *self.network.element(e) {
                    rows.push(["contactor".into(), c.name.clone(), nm(a), nm(b), String::new()]);
                }
            }
        }
        for s in &self.sspcs {
            let load = s.load.map(|li| self.loads[li].name.clone()).unwrap_or_default();
            if let Some(e) = s.switch {
                named.insert(e.0);
            }
            rows.push(["sspc".into(), s.id.clone(), s.rpdu.clone(), nm(s.node), load]);
        }
        for b in &self.breaker_circuits {
            rows.push([
                "breaker".into(),
                b.id.clone(),
                b.bus.clone(),
                b.load.clone(),
                format!("{} {}", b.kind, b.rating_a),
            ]);
        }
        for l in &self.loads {
            named.insert(l.element.0);
            rows.push([
                "load".into(),
                l.name.clone(),
                nm(l.node),
                format!("{}", l.v_nominal),
                format!("{:.0}", l.demand_w),
            ]);
        }
        for (i, e) in self.network.elements().iter().enumerate() {
            if named.contains(&i) {
                continue;
            }
            let (kind, a, b) = match *e {
                Element::Resistor { a, b, .. } => ("resistor", nm(a), nm(b)),
                Element::Impedance { a, b, .. } => ("feeder", nm(a), nm(b)),
                Element::Switch { a, b, .. } => ("switch", nm(a), nm(b)),
                Element::DcSource { node, .. } => ("dc_source", nm(node), String::new()),
                Element::AcSource { node, .. } => ("ac_source", nm(node), String::new()),
                Element::Rectifier { ac, pos, .. } => ("rectifier", nm(ac), nm(pos)),
                Element::Autotransformer { input, output, .. } => ("autotransformer", nm(input), nm(output)),
                _ => continue,
            };
            rows.push([kind.into(), format!("e{i}"), a, b, String::new()]);
        }
        rows
    }

    pub fn load_classes(&self) -> Vec<(String, bool, f64, f64)> {
        self.loads
            .iter()
            .map(|l| (l.name.clone(), l.ac, l.v_nominal, l.demand_w))
            .collect()
    }

    pub fn circuit_bus(&self, load: &str) -> Option<&str> {
        self.circuits
            .iter()
            .find(|(n, _)| n == load)
            .map(|(_, b)| b.as_str())
    }

    pub fn is_ac_node(&self, node: &str) -> bool {
        self.nodes
            .get(node)
            .is_some_and(|n| self.network.node_kind(*n) == NodeKind::Ac3)
    }

    pub fn circuits(&self) -> &[(String, String)] {
        &self.circuits
    }

    pub fn load_demand_w(&self, name: &str) -> Option<f64> {
        self.loads.iter().find(|l| l.name == name).map(|l| l.demand_w)
    }

    pub fn set_load(&mut self, name: &str, demand_w: f64) -> bool {
        let Some(i) = self.loads.iter().position(|l| l.name == name) else {
            return false;
        };
        self.loads[i].demand_w = demand_w.max(0.0);
        self.write_load(i);
        true
    }

    pub fn set_sspc(&mut self, id: &str, closed: bool) -> bool {
        let Some(i) = self.sspcs.iter().position(|s| s.id == id) else {
            return false;
        };
        self.sspcs[i].closed = closed;
        if let Some(sw) = self.sspcs[i].switch {
            return self.network.set_switch(sw, closed).is_ok();
        }
        if let Some(li) = self.sspcs[i].load {
            self.loads[li].enabled = closed;
            self.write_load(li);
        }
        true
    }

    fn write_load(&mut self, i: usize) {
        let l = &self.loads[i];
        let w = if l.enabled { l.demand_w } else { 0.0 };
        let _ = self
            .network
            .set_element(l.element, load_element(l.node, l.ac, l.v_nominal, w));
    }

    fn update_sources(&mut self, dt_us: u64) {
        for g in self.generators.iter_mut() {
            let p = g.machine.params;
            let speed_ratio = g.rpm / p.ref_rpm;
            let exc = if let Some(a) = g.exciter_a {
                a
            } else if speed_ratio > 0.0 {
                g.field_pu * p.i_field_nominal_a / (p.exciter_gain * speed_ratio)
            } else {
                0.0
            };
            let out = g.machine.step(&MachineInput {
                dt_us,
                shaft_rpm: g.rpm,
                exciter_current_cmd_a: exc,
                oil_inlet_c: g.oil_inlet_c,
                ..MachineInput::default()
            });
            let t = out.thevenin;
            let live = t.emf_v > 0.0;
            let _ = self.network.set_element(
                g.element,
                Element::AcSource {
                    node: g.node,
                    emf: t.emf_v,
                    angle: t.angle_rad,
                    r: if live { t.r_ohm } else { R_OFF },
                    x: if live { t.x_ohm } else { 0.0 },
                    frequency_hz: t.freq_hz.max(1.0),
                    r_neutral_ground: R_NEUTRAL,
                },
            );
        }
        for e in self.exts.iter() {
            let _ = self.network.set_element(
                e.element,
                Element::AcSource {
                    node: e.node,
                    emf: if e.connected { V_EXT_PHASE } else { 0.0 },
                    angle: 0.0,
                    r: if e.connected { EXT_R } else { R_OFF },
                    x: EXT_X,
                    frequency_hz: EXT_FREQ_HZ,
                    r_neutral_ground: R_NEUTRAL,
                },
            );
        }
        for b in self.batteries.iter_mut() {
            b.pack
                .step(dt_us, b.current_a - b.charge_a, AMBIENT_C, [false; MAX_CELLS]);
            let (e, r) = b.pack.thevenin();
            let _ = self
                .network
                .set_element(b.element, Element::DcSource { node: b.node, e, r });
        }
        if let Some(s) = self.spu.as_ref() {
            let on = s.enabled;
            let _ = self.network.set_element(
                s.source,
                Element::DcSource {
                    node: s.source_node,
                    e: if on { SPU_OUT_V } else { 0.0 },
                    r: if on { SPU_R_OUT } else { R_OFF },
                },
            );
            let w = if on { s.out_w / SPU_EFFICIENCY } else { 0.0 };
            let _ = self
                .network
                .set_element(s.input, load_element(s.input_node, false, 28.0, w));
        }
    }

    fn bdm_target(&self, sol: &solver_electrical::Solution) -> bool {
        let mbr_closed = self.mbr.map(|i| self.contactors[i].closed).unwrap_or(false);
        let tru_live = self
            .tru_outputs
            .iter()
            .any(|n| sol.dc_voltage(*n) > APPROX_BDM_TRU_LIVE_V);
        mbr_closed && tru_live
    }

    fn settle_diodes(&mut self) -> Result<solver_electrical::Solution, String> {
        let mut sol = self.solver.solve(&self.network).map_err(err)?;
        for _ in 0..DIODE_PASSES {
            let mut changed = false;
            for (di, d) in self.diodes.iter_mut().enumerate() {
                let (vi, vf) = if d.ac {
                    let pi = sol.phase_voltages(d.inner);
                    let pf = sol.phase_voltages(d.far);
                    ((pi[0] - pi[3]).abs(), (pf[0] - pf[3]).abs())
                } else {
                    (sol.dc_voltage(d.inner), sol.dc_voltage(d.far))
                };
                let margin = if d.ac { DIODE_MARGIN_AC } else { DIODE_MARGIN_DC };
                let blocked = d.bdm && self.bdm_block;
                let want = if let Some(f) = self.overlay.diode_force.get(&di) {
                    *f
                } else if blocked {
                    false
                } else if d.closed {
                    vi > vf && vi > 1.0
                } else {
                    vi > vf + margin && vi > 1.0
                };
                if want != d.closed {
                    d.closed = want;
                    let _ = self.network.set_switch(d.switch, want);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
            sol = self.solver.solve(&self.network).map_err(err)?;
        }
        Ok(sol)
    }

    pub fn step(&mut self, dt_us: u64) -> Result<Snapshot, String> {
        self.update_sources(dt_us);
        let mut sol = self.settle_diodes()?;
        let block = self.bdm_target(&sol);
        if block != self.bdm_block {
            self.bdm_block = block;
            sol = self.settle_diodes()?;
        }
        if let Some(s) = self.spu.as_mut() {
            let closed = s.contactor < self.contactors.len() && self.contactors[s.contactor].closed;
            let vin = sol.dc_voltage(s.input_node);
            let hbb = self
                .nodes
                .get("HBB_APU")
                .map(|n| sol.dc_voltage(*n))
                .unwrap_or(0.0);
            s.enabled = closed && (vin >= SPU_MIN_IN_V || hbb >= SPU_MIN_IN_V);
            let i = sol.element_currents(s.source)[0].re.max(0.0);
            s.out_w = if s.enabled { SPU_OUT_V * i } else { 0.0 };
        }
        for b in self.batteries.iter_mut() {
            b.current_a = sol.element_currents(b.element)[0].re;
        }
        let mut buses = Vec::new();
        for (name, n) in self.bus_list.iter() {
            let live = sol.is_live(*n);
            let (v, f) = match self.network.node_kind(*n) {
                NodeKind::Dc => (sol.dc_voltage(*n), 0.0),
                NodeKind::Ac3 => {
                    let p = sol.phase_voltages(*n);
                    ((p[0] - p[3]).abs(), sol.frequency_of(*n))
                }
            };
            buses.push(BusReading {
                name: name.clone(),
                voltage_v: v,
                live,
                freq_hz: f,
            });
        }
        let mut contactors = Vec::new();
        for c in self.contactors.iter() {
            let (current_a, phase_a, residual) = match c.element {
                Some(e) => {
                    let cur = sol.element_currents(e);
                    if c.ac {
                        let r = cur[0] + cur[1] + cur[2];
                        (
                            cur[0].abs(),
                            [cur[0].abs(), cur[1].abs(), cur[2].abs()],
                            (r.re, r.im),
                        )
                    } else {
                        let d = cur[0].re.abs();
                        (d, [d, 0.0, 0.0], (0.0, 0.0))
                    }
                }
                None => (0.0, [0.0; 3], (0.0, 0.0)),
            };
            contactors.push(ContactorReading {
                name: c.name.clone(),
                closed: c.closed,
                current_a,
                phase_a,
                residual,
            });
        }
        let mut machines = Vec::new();
        for g in self.generators.iter() {
            let p = sol.phase_voltages(g.node);
            let cur = sol.element_currents(g.element);
            machines.push(MachineReading {
                name: g.name.clone(),
                freq_hz: g.machine.params.electrical_freq_hz(g.rpm),
                terminal_v: (p[0] - p[3]).abs(),
                current_a: cur[0].abs(),
                field_pu: g.field_pu,
                phase_a: [cur[0].abs(), cur[1].abs(), cur[2].abs()],
                phase_v: [(p[0] - p[3]).abs(), (p[1] - p[3]).abs(), (p[2] - p[3]).abs()],
                residual_a: (cur[0] + cur[1] + cur[2]).abs(),
                residual: ((cur[0] + cur[1] + cur[2]).re, (cur[0] + cur[1] + cur[2]).im),
            });
        }
        let mut batteries = Vec::new();
        for b in self.batteries.iter() {
            batteries.push(BatteryReading {
                name: b.name.clone(),
                voltage_v: b.pack.terminal_v(b.current_a),
                current_a: b.current_a,
                soc: b.pack.soc_mean(),
                temp_k: b.pack.temp_c() + 273.15,
            });
        }
        let load_reading = |l: &Load| -> LoadReading {
            let (v, i) = if l.ac {
                let p = sol.phase_voltages(l.node);
                ((p[0] - p[3]).abs(), sol.element_currents(l.element)[0].abs())
            } else {
                (sol.dc_voltage(l.node), sol.element_currents(l.element)[0].re)
            };
            LoadReading {
                name: l.name.clone(),
                voltage_v: v,
                current_a: i,
                power_w: sol.element_power(l.element).re,
            }
        };
        let loads: Vec<LoadReading> = self.loads.iter().map(load_reading).collect();
        let mut sspcs = Vec::new();
        for s in self.sspcs.iter() {
            let voltage_v = if self.network.node_kind(s.node) == NodeKind::Ac3 {
                let p = sol.phase_voltages(s.node);
                (p[0] - p[3]).abs()
            } else {
                sol.dc_voltage(s.node)
            };
            let current_a = if let Some(li) = s.load {
                loads[li].current_a
            } else if let Some(sw) = s.switch {
                sol.element_currents(sw)[0].re.abs()
            } else {
                0.0
            };
            sspcs.push(SspcReading {
                id: s.id.clone(),
                rpdu: s.rpdu.clone(),
                voltage_v,
                current_a,
            });
        }
        let converters = self
            .rectifiers
            .iter()
            .map(|(name, e, pos, neg)| {
                let v = sol.dc_voltage(*pos) - neg.map(|n| sol.dc_voltage(n)).unwrap_or(0.0);
                let w = sol.converter_output_power(*e);
                ConverterReading {
                    name: name.clone(),
                    output_v: v,
                    output_a: if v.abs() > 1.0 { w / v } else { 0.0 },
                    output_w: w,
                }
            })
            .collect();
        let mut leak = vec![[0.0_f64; 3]; self.leak_zones.len()];
        for i in 0..self.network.element_count() {
            if let Element::AcLoad { node, .. } = *self.network.element(ElementId(i))
                && let Some(z) = self.leak_zones.iter().position(|(_, n)| *n == node)
            {
                let cur = sol.element_currents(ElementId(i));
                for p in 0..3 {
                    leak[z][p] += cur[p].abs();
                }
            }
        }
        let node_leak_a = self
            .leak_zones
            .iter()
            .zip(leak)
            .map(|((name, _), a)| (name.clone(), a.iter().cloned().fold(0.0, f64::max)))
            .collect();
        Ok(Snapshot {
            buses,
            contactors,
            machines,
            batteries,
            loads,
            sspcs,
            converters,
            node_leak_a,
            battery_current_a: self.batteries[0].current_a,
            converged: sol.report.converged,
            outer_iterations: sol.report.outer_iterations,
            outer_residual: sol.report.outer_residual,
            inner_residual: sol.report.inner_residual,
        })
    }
}

impl Default for ElectricalNet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
