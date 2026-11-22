use super::Electrical;
use super::plant::MACHINE_NAMES;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const CONTACTORS_CSV: &str = include_str!("../../../../../aircraft/24-electrical/contactors.csv");

const DC_NODES: [&str; 8] = [
    "DC28_L",
    "DC28_R",
    "CIB",
    "FIB",
    "HBB_MAIN",
    "HBB_APU",
    "BATT_MAIN",
    "BATT_APU",
];

const SOURCE_LIVE_V: f64 = 50.0;

pub(crate) fn listed_rating_a(name: &str) -> Option<f64> {
    CONTACTORS_CSV.lines().skip(1).find_map(|line| {
        if line.contains('"') {
            return None;
        }
        let f: Vec<&str> = line.split(',').collect();
        (f.len() > 6 && f[1] == name)
            .then(|| f[6].trim().parse::<f64>().ok())
            .flatten()
    })
}

fn ends() -> &'static BTreeMap<String, (String, String)> {
    static ENDS: OnceLock<BTreeMap<String, (String, String)>> = OnceLock::new();
    ENDS.get_or_init(|| {
        let mut m = BTreeMap::new();
        for line in CONTACTORS_CSV.lines().skip(1) {
            if line.contains('"') {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            if f.len() < 4 || f[2].is_empty() || f[3].is_empty() {
                continue;
            }
            let ac = !DC_NODES.contains(&f[2]) && !DC_NODES.contains(&f[3]);
            if ac {
                m.insert(f[1].to_string(), (f[2].to_string(), f[3].to_string()));
            }
        }
        m
    })
}

struct Uf {
    parent: Vec<usize>,
    sourced: Vec<bool>,
}

impl Uf {
    fn find(&mut self, mut i: usize) -> usize {
        while self.parent[i] != i {
            self.parent[i] = self.parent[self.parent[i]];
            i = self.parent[i];
        }
        i
    }
}

impl Electrical {
    fn source_live(&self, node: &str) -> bool {
        if let Some(m) = MACHINE_NAMES.iter().position(|n| *n == node) {
            return self.plant.machine_sense(m).0 > SOURCE_LIVE_V;
        }
        if node == "RAT_GEN" {
            return self.plant.machine_sense(6).0 > SOURCE_LIVE_V;
        }
        let ext = match node {
            "EXT_L_FWD" => Some(0),
            "EXT_R_FWD" => Some(1),
            "EXT_AFT" => Some(2),
            _ => None,
        };
        if let Some(k) = ext {
            let e = &self.inputs.ext[k];
            return e.connected && e.v_ll > SOURCE_LIVE_V;
        }
        if node == "SPU" {
            return self.plant.bus_v("SPU") > SOURCE_LIVE_V;
        }
        false
    }

    pub(crate) fn interlock(&mut self, cmds: &mut [bool]) -> Vec<String> {
        let ends = ends();
        let mut index: BTreeMap<&str, usize> = BTreeMap::new();
        for (a, b) in ends.values() {
            let n = index.len();
            index.entry(a.as_str()).or_insert(n);
            let n = index.len();
            index.entry(b.as_str()).or_insert(n);
        }
        let mut uf = Uf {
            parent: (0..index.len()).collect(),
            sourced: vec![false; index.len()],
        };
        for (name, &i) in &index {
            uf.sourced[i] = self.source_live(name);
        }
        let mut order: Vec<usize> = (0..self.gear.len()).collect();
        order.sort_by_key(|&k| (!self.gear[k].out.contacts_closed, k));
        let mut blocked = Vec::new();
        for k in order {
            if !cmds[k] {
                continue;
            }
            let Some((a, b)) = ends.get(&self.gear[k].name) else {
                continue;
            };
            let (Some(&ia), Some(&ib)) = (index.get(a.as_str()), index.get(b.as_str())) else {
                continue;
            };
            let ra = uf.find(ia);
            let rb = uf.find(ib);
            if ra == rb {
                continue;
            }
            if uf.sourced[ra] && uf.sourced[rb] {
                cmds[k] = false;
                blocked.push(self.gear[k].name.clone());
                continue;
            }
            uf.parent[ra] = rb;
            uf.sourced[rb] = uf.sourced[ra] || uf.sourced[rb];
        }
        blocked
    }
}

#[cfg(test)]
mod tests {
    use super::ends;

    #[test]
    fn ac_contactors_have_ends_and_dc_ties_are_excluded() {
        let e = ends();
        assert!(e.contains_key("L1 GCB"));
        assert!(e.contains_key("C1 TRU RELAY"));
        assert!(!e.contains_key("L DCT"));
        assert!(!e.contains_key("MBR"));
    }
}
