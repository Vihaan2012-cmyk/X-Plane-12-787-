use crate::cabinet::{CabinetState, GPM_COUNT};
use crate::ccs::Ccs;
use crate::cdn::DevKind;
use crate::icd::{Dataset, Encoding, FS_NORMAL};
use crate::network::{CAS_HOSTS, GPMS};

pub const CAS_APP: &str = "CREW ALERTING FUNCTION";
pub const APPROX_CCSHM_MSG_DELAY_US: u64 = 30_000_000;
pub const ELEC_DATASET: &str = "BPCU ELEC STATUS";
pub const GND_TEST_DELAY_US: u64 = 2_500_000;

impl Ccs {
    pub(crate) fn step_software(&mut self) {
        let now = self.now_us;
        for c in 0..2 {
            let boots = self.cabinets[c].boots;
            let off = self.cabinets[c].state == CabinetState::Off;
            if boots != self.last_boots[c] || off {
                for k in 0..GPM_COUNT {
                    self.sw[c * GPM_COUNT + k].reset();
                }
                self.last_boots[c] = boots;
            }
        }
        if self.gnd_test_sel != self.gnd_test_eff {
            let leaving = self.gnd_test_eff != 0 && self.gnd_test_sel < self.gnd_test_eff;
            if !leaving || now.saturating_sub(self.gnd_test_since) >= GND_TEST_DELAY_US {
                self.gnd_test_eff = self.gnd_test_sel;
            }
        } else {
            self.gnd_test_since = now;
        }
        let load_ok = self.gnd_test_eff == 2 && self.inputs.on_ground;
        if !load_ok {
            for sw in self.sw.iter_mut() {
                sw.data_load = false;
            }
        }
        let faults: Vec<(String, String)> = self.part_faults.iter().cloned().collect();
        for (gpm, part) in faults {
            if let Some(sw) = self.sw.iter_mut().find(|s| s.name == gpm)
                && let Some(p) = sw.partition_mut(&part)
            {
                p.crash_requested = true;
            }
        }
        for i in 0..self.sw.len() {
            let name = self.sw[i].name.clone();
            if self.gpm_running(&name) {
                self.sw[i].step(now);
            }
        }
        let alive = self.ccshm_alive();
        let any_up = self.cabinets.iter().any(|c| c.hosted_ready());
        if alive || !any_up {
            self.ccshm_lost_since = None;
        } else {
            self.ccshm_lost_since.get_or_insert(now);
        }
    }

    pub fn ccshm_alive(&self) -> bool {
        GPMS.iter().any(|g| self.instance_live("CCSHM", g))
    }

    pub fn set_elec_icd(&mut self, names: &[String]) {
        let defs: Vec<(&str, Encoding)> = names.iter().map(|n| (n.as_str(), Encoding::Discrete)).collect();
        self.elec_icd = Some(Dataset::new(ELEC_DATASET, 1, &defs));
    }

    pub fn encode_elec(&self, active: &[String]) -> Option<Vec<u8>> {
        let ds = self.elec_icd.as_ref()?;
        let vals: Vec<(&str, f64)> = active.iter().map(|n| (n.as_str(), 1.0)).collect();
        Some(ds.encode(&vals, FS_NORMAL))
    }

    pub fn cas_live_host(&self) -> Option<&'static str> {
        CAS_HOSTS.iter().copied().find(|g| self.instance_live(CAS_APP, g))
    }

    pub fn cas_elec_messages(&self) -> Option<(Vec<String>, bool)> {
        let host = self.cas_live_host()?;
        let ds = self.elec_icd.as_ref()?;
        let dest = format!("GPM {host}");
        let fresh = ["BPCU P300", "BPCU P400"]
            .iter()
            .filter_map(|v| self.read(&dest, v))
            .filter(|(d, _, valid)| {
                *valid && Dataset::status(d) == FS_NORMAL && Dataset::version_of(d) == ds.version
            })
            .min_by_key(|(_, age, _)| *age);
        match fresh {
            Some((data, _, _)) => {
                let msgs = ds
                    .params
                    .iter()
                    .filter(|p| ds.decode(&data, &p.name) == Some(1.0))
                    .map(|p| p.name.clone())
                    .collect();
                Some((msgs, true))
            }
            None => Some((Vec::new(), false)),
        }
    }

    pub fn component_faults(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (c, side) in ["L", "R"].iter().enumerate() {
            let cab = &self.cabinets[c];
            if !cab.powered() {
                continue;
            }
            for k in 0..GPM_COUNT {
                let sw = &self.sw[c * GPM_COUNT + k];
                let g = &cab.gpm[k];
                if g.failed || g.overtemp_stop || !sw.hardware_ok() {
                    out.push(format!("CCS GPM {side}{}", k + 1));
                }
            }
            for (i, p) in cab.pcm.iter().enumerate() {
                if p.failed {
                    out.push(format!("CCS PCM {side}{}", i + 1));
                }
            }
        }
        let mut fox = false;
        for d in &self.cdn.devices {
            if !d.failed {
                continue;
            }
            match d.kind {
                DevKind::Fox => fox = true,
                DevKind::Switch => out.push(format!("CCS {}", d.name)),
                DevKind::EndSystem => {}
            }
        }
        if fox {
            out.push("CCR FIBER OPTIC XLTR".into());
        }
        out
    }

    pub fn crew_status_messages(&self) -> Vec<String> {
        let mut m = self.status_messages();
        if self.ccshm_alive() {
            m.extend(self.component_faults());
        } else if self
            .ccshm_lost_since
            .is_some_and(|t| self.now_us.saturating_sub(t) >= APPROX_CCSHM_MSG_DELAY_US)
        {
            m.push("CCS HEALTH MANAGEMENT".into());
        }
        m.sort();
        m.dedup();
        m
    }

    pub fn maint_page(&self, page: u8) -> Vec<(String, String)> {
        let state = |fault: bool| if fault { "FAULT" } else { "NORMAL" }.to_string();
        let term = |dev: &str, ch: usize| -> bool {
            self.cdn.device(dev).is_some_and(|d| {
                let dv = &self.cdn.devices[d];
                dv.powered
                    && !dv.failed
                    && dv.ports.iter().any(|l| {
                        let lk = &self.cdn.links[*l];
                        lk.channel == ch && !lk.failed
                    })
            })
        };
        let comp = self.component_faults();
        let port_down = |dev: &str| {
            self.cdn.device(dev).is_some_and(|d| {
                self.cdn.devices[d]
                    .ports
                    .iter()
                    .any(|l| self.cdn.links[*l].failed)
            })
        };
        match page {
            1 => {
                let l = comp
                    .iter()
                    .any(|c| c.contains(" L") && (c.contains("GPM") || c.contains("PCM")));
                let r = comp
                    .iter()
                    .any(|c| c.contains(" R") && (c.contains("GPM") || c.contains("PCM")));
                let rdc = self.rdcs.iter().any(|r| r.failed);
                let sw = comp.iter().any(|c| c.contains("ARS") || c.contains("ACS"));
                vec![
                    ("CCS LEFT CCR".into(), state(l)),
                    ("CCS RIGHT CCR".into(), state(r)),
                    ("CCS DCA".into(), state(false)),
                    ("CCS RDC'S".into(), state(rdc)),
                    ("CCS SWITCHES".into(), state(sw)),
                    (
                        "CDN LRU".into(),
                        state(["P300", "P400"].iter().any(|d| !term(d, 0) || !term(d, 1))),
                    ),
                ]
            }
            2 | 3 => {
                let side = if page == 2 { "L" } else { "R" };
                let c = if page == 2 { 0 } else { 1 };
                let mut v = Vec::new();
                for k in 0..GPM_COUNT {
                    let g = format!("{side}{}", k + 1);
                    let dev = format!("GPM {g}");
                    let fault =
                        comp.contains(&format!("CCS GPM {g}")) || self.sw[c * GPM_COUNT + k].single_lane();
                    v.push((
                        format!("GPM {}", k + 1),
                        if fault { "SEE CMCF".into() } else { state(false) },
                    ));
                    v.push((
                        format!("GPM {} CDN TERMINAL CHNL A", k + 1),
                        state(!term(&dev, 0)),
                    ));
                    v.push((
                        format!("GPM {} CDN TERMINAL CHNL B", k + 1),
                        state(!term(&dev, 1)),
                    ));
                }
                for i in 0..2 {
                    let p = &self.cabinets[c].pcm[i];
                    v.push((format!("PCM {}", i + 1), state(p.failed)));
                    let aux = self.cabinets[c].fan_failed || self.cabinets[c].valve_failed;
                    v.push((
                        format!("PCM {} AUX COOLING", i + 1),
                        if aux { "SEE CMCF".into() } else { state(false) },
                    ));
                    v.push((format!("PCM {} MASTER", i + 1), String::new()));
                }
                for ch in ["A", "B"] {
                    let acs = format!("ACS {side} {ch}");
                    v.push((
                        format!("ACS {ch}"),
                        state(comp.contains(&format!("CCS {acs}")) || port_down(&acs)),
                    ));
                    let fox = format!("FOX {side} {ch}");
                    let fail = self.cdn.device(&fox).is_some_and(|d| {
                        let dv = &self.cdn.devices[d];
                        dv.failed || dv.ports.iter().any(|l| self.cdn.links[*l].failed)
                    });
                    v.push((format!("FOX {ch}"), state(fail)));
                }
                v
            }
            7 => self
                .rdcs
                .iter()
                .flat_map(|r| {
                    vec![
                        (r.def.id.clone(), state(r.failed)),
                        (format!("{} CHNL A", r.def.id), state(!term(&r.def.id, 0))),
                        (format!("{} CHNL B", r.def.id), state(!term(&r.def.id, 1))),
                    ]
                })
                .collect(),
            8 => (1..=6)
                .flat_map(|n| {
                    let name = format!("ARS {n}");
                    let fail = comp.contains(&format!("CCS {name}")) || port_down(&name);
                    vec![(name.clone(), state(fail))]
                })
                .collect(),
            9..=11 => {
                let lrus: &[&str] = match page {
                    9 => &["P300", "P400"],
                    10 => &["RPDU 71", "RPDU 72", "RPDU 81", "RPDU 82"],
                    _ => &["GG L1", "GG L2", "GG R1", "GG R2"],
                };
                lrus.iter()
                    .flat_map(|d| {
                        vec![
                            (format!("{d} CHNL A"), state(!term(d, 0))),
                            (format!("{d} CHNL B"), state(!term(d, 1))),
                        ]
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}
