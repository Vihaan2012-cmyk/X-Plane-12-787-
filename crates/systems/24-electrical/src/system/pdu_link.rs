use super::Electrical;
use crate::control::cbic::{BreakerCommand, BreakerKind, CbAction, CbState, Cbic, LockTag};
use crate::net::Snapshot;
use crate::pdu::channel::{ChannelAddr, ChannelCommand, ChannelLoad, DefaultState};
use crate::pdu::rpdu::{
    AC12_RATINGS_A, DC19_RATINGS_A, FIRST_PM_SLOT, PowerModuleKind, Rpdu, RpduInput, RpduKind,
};
use crate::pdu::spdu::{SPDU_SLOTS, Spdu, SpduCardKind};
use crate::protection::sspc::{CommandSource, INSTANT_TRIP_MULTIPLE, SspcHealth, SspcState};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const HISTORY_LEN: usize = 12;
const MIN_CHANNEL_RATING_A: f64 = 2.5;
const RESET_HOLD_STEPS: u8 = 2;
const CHANGE_WEIGHT: f64 = 1000.0;

const GATEWAYS: [&str; 4] = ["71", "72", "81", "82"];
const CLUSTERS: [(&str, &[&str]); 4] = [
    ("71", &["75", "21", "31", "33"]),
    ("72", &["76", "22", "32", "34"]),
    ("81", &["73", "41", "92"]),
    ("82", &["74", "42"]),
];
const SPDU_NAMES: [&str; 2] = ["L SPDU", "R SPDU"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingSource {
    Breakers,
    Approx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dev {
    Rpdu(usize),
    Spdu(usize),
}

#[derive(Clone, Debug)]
pub struct LinkChannel {
    pub net_id: String,
    pub load: String,
    pub breaker_id: Option<u32>,
    pub rating_a: f64,
    pub rating_source: RatingSource,
    pub ac: bool,
    pub shed_group: Option<usize>,
    pub state: SspcState,
    pub health: SspcHealth,
    pub current_a: f64,
    pub v_out: f64,
    pub i2t_pct: f64,
    pub arc_events: u32,
    pub trips: u32,
    pub default_active: bool,
    pub shed: bool,
    pub battery_only: bool,
    pub history: Vec<(f64, String)>,
    dev: Dev,
    addr: ChannelAddr,
    net_index: Option<usize>,
    src_bus: String,
    net_closed: bool,
    held: bool,
    status_index: usize,
    user_open: bool,
    user_lock: bool,
    reset_steps: u8,
    sent: Option<(bool, bool)>,
    inject_a: f64,
    noise_a: f64,
}

#[derive(Clone, Debug)]
struct RpduDev {
    id: String,
    location: String,
    gateway: bool,
    cluster_gateway: Option<usize>,
    backup: Option<usize>,
    dc_node: String,
    ac_node: String,
    rpdu: Rpdu,
    forced_comms_loss: bool,
    forced_power_loss: bool,
    comms_ok: bool,
    powered: bool,
    default_active: bool,
    resend: bool,
    input_v: f64,
    input_a: f64,
}

#[derive(Clone, Debug)]
pub struct ChannelRow {
    pub channel: u32,
    pub slot: u8,
    pub index: u8,
    pub net_id: String,
    pub load: String,
    pub rating_a: f64,
    pub rating_source: RatingSource,
    pub state: SspcState,
    pub health: SspcHealth,
    pub current_a: f64,
    pub voltage_out_v: f64,
    pub i2t_pct: f64,
    pub arc_events: u32,
    pub trips: u32,
    pub shed: bool,
    pub default_active: bool,
    pub ac: bool,
    pub instant_trip_a: f64,
    pub history: Vec<(f64, String)>,
}

#[derive(Clone, Debug)]
pub struct RpduRow {
    pub id: String,
    pub location: String,
    pub gateway: bool,
    pub input_v: f64,
    pub input_a: f64,
    pub powered: bool,
    pub comms_ok: bool,
    pub default_active: bool,
    pub channels: Vec<ChannelRow>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapSummary {
    pub total: usize,
    pub rpdu_channels: usize,
    pub feed_channels: usize,
    pub dormant_channels: usize,
    pub unmapped_net: usize,
    pub rating_from_breakers: usize,
    pub rating_approx: usize,
}

pub(crate) struct LinkInputs<'a> {
    pub dt_us: u64,
    pub now_us: u64,
    pub snap: &'a Snapshot,
    pub shed: [bool; 14],
    pub cdn_ok: bool,
    pub ground_battery_only: bool,
}

#[derive(Clone, Debug)]
pub struct PduLink {
    rpdus: Vec<RpduDev>,
    spdus: Vec<Spdu>,
    channels: Vec<LinkChannel>,
    by_breaker: BTreeMap<u32, usize>,
    by_net: BTreeMap<String, usize>,
    changes: u64,
    summary: MapSummary,
}

fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            ',' => row.push(std::mem::take(&mut field)),
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            _ => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

struct Table {
    head: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn load(dir: &Path, file: &str) -> Result<Table, String> {
        let text = fs::read_to_string(dir.join(file)).map_err(|e| format!("{file}: {e}"))?;
        let mut rows = parse_csv(&text);
        if rows.is_empty() {
            return Err(format!("{file}: empty"));
        }
        let head = rows.remove(0);
        Ok(Table { head, rows })
    }

    fn col(&self, row: &[String], name: &str) -> String {
        self.head
            .iter()
            .position(|h| h == name)
            .and_then(|i| row.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
}

fn rpdu_number(text: &str) -> Option<String> {
    let rest = text.trim().strip_prefix("RPDU")?.trim();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() { None } else { Some(digits) }
}

fn name_rpdu_number(name: &str) -> Option<String> {
    let idx = name.find("RPDU")?;
    rpdu_number(&name[idx..])
}

fn is_ac_text(text: &str) -> bool {
    let t = text.to_ascii_lowercase();
    t.contains("115") || t.contains("235") || t.contains("v ac")
}

fn side_left(num: &str) -> bool {
    num.parse::<u32>().map(|n| n % 2 == 1).unwrap_or(true)
}

fn shed_group_of(load: &str) -> Option<usize> {
    let l = load.to_ascii_lowercase();
    let rules: [(&[&str], usize); 9] = [
        (&["galley"], 0),
        (&["entertainment", " ife"], 2),
        (&["cabin light"], 3),
        (&["cargo heat"], 4),
        (&["recirc"], 5),
        (&["ram air"], 6),
        (&["window h", "windshield heater", "window ht"], 7),
        (&["fuel pump"], 9),
        (&["exterior light", "landing light", "strobe"], 11),
    ];
    rules
        .iter()
        .find(|(keys, _)| keys.iter().any(|k| l.contains(k)))
        .map(|(_, g)| *g)
}

const BATTERY_ONLY_LOADS: [&str; 14] = [
    "inboard HDD",
    "Left EFIS",
    "MCP",
    "MODE CTRL PNL-L",
    "Captain ACP",
    "First officer ACP",
    "L VHF",
    "Left TCP",
    "AGU FWD L",
    "ASG",
    "standby DC power",
    "power supply backup",
    "controller supply",
    "(primary)",
];

fn battery_only_load(load: &str) -> bool {
    BATTERY_ONLY_LOADS.iter().any(|k| load.contains(k))
}

fn default_of(load: &str, shed_group: Option<usize>) -> DefaultState {
    let l = load.to_ascii_lowercase();
    if l.contains("standby dc power") || l.contains("power supply backup") || l.contains("controller supply")
    {
        DefaultState::On
    } else if shed_group.is_some() {
        DefaultState::Off
    } else {
        DefaultState::LastCommanded
    }
}

struct ChanSeed {
    net_id: String,
    net_index: usize,
    load: String,
    ac: bool,
    breaker_id: Option<u32>,
    rating_req: f64,
    rated_by_breaker: bool,
}

struct SpduSeed {
    id: String,
    breaker_id: u32,
    name: String,
    spdu: usize,
    slot: usize,
    kind: SpduCardKind,
    channel: usize,
    csv_rating: f64,
    net_index: Option<usize>,
    src_bus: String,
}

fn parse_card(text: &str) -> Option<(SpduCardKind, usize)> {
    let mut kind = None;
    let mut slot = None;
    let mut words = text.split_whitespace().peekable();
    while let Some(w) = words.next() {
        match w {
            "DC2" => kind = Some(SpduCardKind::Dc2),
            "DC4" => kind = Some(SpduCardKind::Dc4),
            "DC19" => kind = Some(SpduCardKind::Dc19),
            "AC1" => kind = Some(SpduCardKind::Ac1),
            "AC2" => kind = Some(SpduCardKind::Ac2),
            "slot" => slot = words.next().and_then(|s| s.parse::<usize>().ok()),
            _ => {}
        }
    }
    Some((kind?, slot?))
}

impl PduLink {
    pub fn new(
        data_dir: &Path,
        sspc_ids: &[String],
        sspc_loads: &[Option<String>],
    ) -> Result<PduLink, String> {
        let rpdus_t = Table::load(data_dir, "rpdus.csv")?;
        let loads_t = Table::load(data_dir, "loads.csv")?;
        let breakers_t = Table::load(data_dir, "breakers.csv")?;
        let mut summary = MapSummary::default();

        let mut rpdu_info: Vec<(String, bool, bool, String)> = Vec::new();
        for r in &rpdus_t.rows {
            let Some(num) = rpdu_number(&rpdus_t.col(r, "id")) else {
                continue;
            };
            let has_ac = rpdus_t.col(r, "feeds").contains("115 VAC");
            let gateway = rpdus_t.col(r, "gateway") == "Y";
            let st = rpdus_t.col(r, "station");
            let location = if st.is_empty() {
                "location not documented".to_string()
            } else {
                format!(
                    "STA {} BL {} WL {}",
                    st,
                    rpdus_t.col(r, "buttline"),
                    rpdus_t.col(r, "waterline")
                )
            };
            rpdu_info.push((num, has_ac, gateway, location));
        }

        let channel_ids: Vec<(usize, &String)> = sspc_ids
            .iter()
            .enumerate()
            .filter(|(_, id)| id.starts_with("RPDU"))
            .collect();

        let mut breaker_rating: BTreeMap<String, f64> = BTreeMap::new();
        for r in &breakers_t.rows {
            if let Ok(v) = breakers_t.col(r, "rating_a").parse::<f64>() {
                breaker_rating.insert(breakers_t.col(r, "id"), v);
            }
        }

        let mut per_rpdu: BTreeMap<String, usize> = BTreeMap::new();
        let mut seeds: BTreeMap<String, Vec<ChanSeed>> = BTreeMap::new();
        let mut attrs: BTreeMap<String, (String, bool, Option<u32>, f64, bool)> = BTreeMap::new();
        for row in &loads_t.rows {
            let Some(num) = rpdu_number(&loads_t.col(row, "unit_rpdu")) else {
                continue;
            };
            let Some((_, has_ac, _, _)) = rpdu_info.iter().find(|i| i.0 == num) else {
                continue;
            };
            let ac = is_ac_text(&loads_t.col(row, "voltage_bus")) && *has_ac;
            let idx = per_rpdu.entry(num.clone()).or_insert(0);
            *idx += 1;
            let id = format!("RPDU{num}.{:02}", *idx);
            let bid_text = loads_t.col(row, "breaker_id");
            let breaker_id = bid_text.get(2..).and_then(|s| s.parse::<u32>().ok());
            let table_rating = breaker_rating
                .get(&bid_text)
                .copied()
                .or_else(|| loads_t.col(row, "rating").parse::<f64>().ok());
            attrs.insert(
                id,
                (
                    loads_t.col(row, "load"),
                    ac,
                    breaker_id,
                    table_rating.unwrap_or(0.0),
                    table_rating.is_some(),
                ),
            );
        }
        for (net_index, id) in channel_ids.iter() {
            let Some((load, ac, breaker_id, rating_req, rated)) = attrs.get(*id) else {
                summary.unmapped_net += 1;
                continue;
            };
            let num: String = id.chars().skip(4).take_while(|c| c.is_ascii_digit()).collect();
            let load_name = sspc_loads
                .get(*net_index)
                .cloned()
                .flatten()
                .unwrap_or_else(|| load.clone());
            seeds.entry(num).or_default().push(ChanSeed {
                net_id: (*id).clone(),
                net_index: *net_index,
                load: load_name,
                ac: *ac,
                breaker_id: *breaker_id,
                rating_req: *rating_req,
                rated_by_breaker: *rated,
            });
        }

        let mut rpdus: Vec<RpduDev> = Vec::new();
        let mut channels: Vec<LinkChannel> = Vec::new();
        for (num, has_ac, gateway, location) in &rpdu_info {
            let list = seeds.remove(num).unwrap_or_default();
            let n_ac = list.iter().filter(|s| s.ac).count();
            let n_dc = list.len() - n_ac;
            let m_dc = n_dc.div_ceil(19).max(1);
            let m_ac = if *has_ac { n_ac.div_ceil(12) } else { 0 };
            let mut kinds = [PowerModuleKind::Spare; 6];
            let mut pool: Vec<(f64, usize, usize, bool)> = Vec::new();
            let mut slot_i = 0;
            for _ in 0..m_dc.min(6) {
                kinds[slot_i] = PowerModuleKind::Dc19;
                for (c, r) in DC19_RATINGS_A.iter().enumerate() {
                    pool.push((*r, slot_i, c, false));
                }
                slot_i += 1;
            }
            for _ in 0..m_ac.min(6 - slot_i) {
                kinds[slot_i] = PowerModuleKind::Ac12;
                for (c, r) in AC12_RATINGS_A.iter().enumerate() {
                    pool.push((*r, slot_i, c, true));
                }
                slot_i += 1;
            }
            let kind = if *gateway {
                RpduKind::Gateway
            } else {
                RpduKind::Standard
            };
            let mut rpdu = Rpdu::new(kind, kinds);
            let dev_index = rpdus.len();
            let mut taken: Vec<bool> = vec![false; pool.len()];
            let offsets: Vec<usize> = {
                let mut acc = 0;
                rpdu.modules
                    .iter()
                    .map(|m| {
                        let o = acc;
                        acc += m.channels.len();
                        o
                    })
                    .collect()
            };
            for seed in list {
                let want = seed.rating_req.max(MIN_CHANNEL_RATING_A);
                let pick = pool
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| !taken[*i] && p.3 == seed.ac && p.0 >= want)
                    .min_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap().then(a.0.cmp(&b.0)))
                    .map(|(i, _)| i)
                    .or_else(|| {
                        pool.iter()
                            .enumerate()
                            .filter(|(i, p)| !taken[*i] && p.3 == seed.ac)
                            .max_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap().then(b.0.cmp(&a.0)))
                            .map(|(i, _)| i)
                    });
                let Some(pi) = pick else {
                    summary.unmapped_net += 1;
                    continue;
                };
                taken[pi] = true;
                let (rating, module, ch, _) = pool[pi];
                let addr = ChannelAddr {
                    slot: FIRST_PM_SLOT + module as u8,
                    channel: ch as u8,
                };
                let shed_group = shed_group_of(&seed.load);
                let battery_only = battery_only_load(&seed.load);
                if let Some(c) = rpdu.channel_mut(addr) {
                    c.default = default_of(&seed.load, shed_group);
                }
                let from_breaker = seed.rated_by_breaker && (seed.rating_req - rating).abs() < 1e-9;
                if from_breaker {
                    summary.rating_from_breakers += 1;
                } else {
                    summary.rating_approx += 1;
                }
                summary.rpdu_channels += 1;
                channels.push(LinkChannel {
                    net_id: seed.net_id,
                    load: seed.load,
                    breaker_id: seed.breaker_id,
                    rating_a: rating,
                    rating_source: if from_breaker {
                        RatingSource::Breakers
                    } else {
                        RatingSource::Approx
                    },
                    ac: seed.ac,
                    shed_group,
                    state: SspcState::Open,
                    health: SspcHealth::Ok,
                    current_a: 0.0,
                    v_out: 0.0,
                    i2t_pct: 0.0,
                    arc_events: 0,
                    trips: 0,
                    default_active: false,
                    shed: false,
                    battery_only,
                    history: Vec::new(),
                    dev: Dev::Rpdu(dev_index),
                    addr,
                    net_index: Some(seed.net_index),
                    src_bus: String::new(),
                    net_closed: false,
                    held: false,
                    status_index: offsets[module] + ch,
                    user_open: false,
                    user_lock: false,
                    reset_steps: 0,
                    sent: None,
                    inject_a: 0.0,
                    noise_a: 0.0,
                });
            }
            rpdus.push(RpduDev {
                id: format!("RPDU {num}"),
                location: location.clone(),
                gateway: *gateway,
                cluster_gateway: None,
                backup: None,
                dc_node: format!("RPDU {num} DC"),
                ac_node: format!("RPDU {num} AC"),
                rpdu,
                forced_comms_loss: false,
                forced_power_loss: false,
                comms_ok: true,
                powered: false,
                default_active: false,
                resend: true,
                input_v: 0.0,
                input_a: 0.0,
            });
        }
        let ids: Vec<String> = rpdus.iter().map(|r| r.id.clone()).collect();
        let index_of = |num: &str| ids.iter().position(|i| *i == format!("RPDU {num}"));
        for (gw, members) in CLUSTERS {
            if let Some(g) = index_of(gw) {
                for m in members {
                    if let Some(mi) = index_of(m) {
                        rpdus[mi].cluster_gateway = Some(g);
                    }
                }
            }
        }
        for gw in GATEWAYS {
            if let Some(g) = index_of(gw) {
                rpdus[g].cluster_gateway = Some(g);
            }
        }
        for row in &rpdus_t.rows {
            let Some(num) = rpdu_number(&rpdus_t.col(row, "id")) else {
                continue;
            };
            let feeds = rpdus_t.col(row, "feeds");
            let Some(pos) = feeds.find("(control backup)") else {
                continue;
            };
            let head = &feeds[..pos];
            let Some(from) = head.rfind("RPDU") else {
                continue;
            };
            if let (Some(me), Some(other)) = (
                index_of(&num),
                rpdu_number(&head[from..]).and_then(|n| index_of(&n)),
            ) {
                rpdus[me].backup = Some(other);
            }
        }

        let mut spdus = vec![Spdu::new(), Spdu::new()];
        let mut seeds: Vec<SpduSeed> = Vec::new();
        for r in &breakers_t.rows {
            let id = breakers_t.col(r, "id");
            let name = breakers_t.col(r, "name");
            let unit = breakers_t.col(r, "unit");
            let Some(spdu) = SPDU_NAMES.iter().position(|n| *n == unit) else {
                continue;
            };
            if breakers_t.col(r, "prefix_type") != "ECB" {
                continue;
            }
            let Some((kind, slot)) = parse_card(&breakers_t.col(r, "card_module")) else {
                continue;
            };
            let Ok(chan) = breakers_t.col(r, "channel").parse::<usize>() else {
                continue;
            };
            let Some(breaker_id) = id.get(2..).and_then(|s| s.parse::<u32>().ok()) else {
                continue;
            };
            if slot >= SPDU_SLOTS || chan == 0 || chan > kind.ratings_a().len() {
                continue;
            }
            let net_index = sspc_ids.iter().position(|s| *s == id);
            let standby = name.starts_with("STANDBY DC FEED RPDU");
            let left = if net_index.is_some() {
                name_rpdu_number(&name)
                    .map(|n| side_left(&n))
                    .unwrap_or(spdu == 0)
            } else {
                spdu == 0
            };
            let src_bus = match (standby, left) {
                (false, true) => "DC28_L",
                (false, false) => "DC28_R",
                (true, true) => "CIB",
                (true, false) => "FIB",
            };
            seeds.push(SpduSeed {
                id,
                breaker_id,
                name,
                spdu,
                slot,
                kind,
                channel: chan - 1,
                csv_rating: breakers_t.col(r, "rating_a").parse::<f64>().unwrap_or(0.0),
                net_index,
                src_bus: src_bus.to_string(),
            });
        }
        for s in &seeds {
            if spdus[s.spdu].slots[s.slot].is_none() {
                spdus[s.spdu].install(s.slot, s.kind);
            }
        }
        for s in seeds {
            let Some(card) = spdus[s.spdu].slots[s.slot].as_ref() else {
                continue;
            };
            if card.kind != s.kind {
                continue;
            }
            let rating = s.kind.ratings_a()[s.channel];
            let addr = ChannelAddr {
                slot: s.slot as u8,
                channel: s.channel as u8,
            };
            if channels
                .iter()
                .any(|c| c.dev == Dev::Spdu(s.spdu) && c.addr == addr)
            {
                continue;
            }
            let from_breaker = (s.csv_rating - rating).abs() < 1e-9 && s.csv_rating > 0.0;
            if from_breaker {
                summary.rating_from_breakers += 1;
            } else {
                summary.rating_approx += 1;
            }
            let feed = s.net_index.is_some();
            if feed {
                summary.feed_channels += 1;
            } else {
                summary.dormant_channels += 1;
            }
            channels.push(LinkChannel {
                net_id: if feed { s.id.clone() } else { String::new() },
                load: s.name,
                breaker_id: Some(s.breaker_id),
                rating_a: rating,
                rating_source: if from_breaker {
                    RatingSource::Breakers
                } else {
                    RatingSource::Approx
                },
                ac: false,
                shed_group: None,
                state: SspcState::Open,
                health: SspcHealth::Ok,
                current_a: 0.0,
                v_out: 0.0,
                i2t_pct: 0.0,
                arc_events: 0,
                trips: 0,
                default_active: false,
                shed: false,
                battery_only: true,
                history: Vec::new(),
                dev: Dev::Spdu(s.spdu),
                addr,
                net_index: s.net_index,
                src_bus: s.src_bus,
                net_closed: feed,
                held: false,
                status_index: 0,
                user_open: false,
                user_lock: false,
                reset_steps: 0,
                sent: None,
                inject_a: 0.0,
                noise_a: 0.0,
            });
        }
        summary.total = channels.len();
        let mut by_breaker = BTreeMap::new();
        let mut by_net = BTreeMap::new();
        for (i, c) in channels.iter().enumerate() {
            if let Some(b) = c.breaker_id {
                by_breaker.insert(b, i);
            }
            if !c.net_id.is_empty() {
                by_net.insert(c.net_id.clone(), i);
            }
        }
        Ok(PduLink {
            rpdus,
            spdus,
            channels,
            by_breaker,
            by_net,
            changes: 0,
            summary,
        })
    }

    pub fn summary(&self) -> MapSummary {
        self.summary
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn channels(&self) -> &[LinkChannel] {
        &self.channels
    }

    pub fn channel(&self, net_id: &str) -> Option<&LinkChannel> {
        self.by_net.get(net_id).map(|i| &self.channels[*i])
    }

    pub fn channel_by_load(&self, load: &str) -> Option<&LinkChannel> {
        self.channels.iter().find(|c| c.load == load)
    }

    pub fn rpdu_ids(&self) -> Vec<String> {
        self.rpdus.iter().map(|r| r.id.clone()).collect()
    }

    pub fn rpdu_powered(&self, rpdu: &str) -> Option<bool> {
        self.rpdus.iter().find(|r| r.id == rpdu).map(|r| r.powered)
    }

    pub fn rpdu_default_active(&self, rpdu: &str) -> Option<bool> {
        self.rpdus.iter().find(|r| r.id == rpdu).map(|r| r.default_active)
    }

    pub fn register(&self, cbic: &mut Cbic) {
        for c in &self.channels {
            if let (Some(id), Dev::Spdu(_)) = (c.breaker_id, c.dev) {
                cbic.add_breaker(id, &c.load, BreakerKind::Ecb, CbState::Open);
            }
        }
    }

    pub fn breaker_truth(&self, id: u32) -> Option<(CbState, f64)> {
        let c = &self.channels[*self.by_breaker.get(&id)?];
        let state = match c.state {
            SspcState::Open => CbState::Open,
            SspcState::Closed => CbState::Closed,
            SspcState::Tripped => CbState::Tripped,
            SspcState::LockedOpen => CbState::Locked(LockTag::DoNotClose),
        };
        Some((state, c.current_a))
    }

    pub(crate) fn hold_net(&mut self, net_id: &str, held: bool) {
        if let Some(i) = self.by_net.get(net_id).copied() {
            let c = &mut self.channels[i];
            if c.held && !held {
                c.net_closed = !(c.state == SspcState::Closed);
            }
            c.held = held;
        }
    }

    pub fn set_channel_open(&mut self, net_id: &str, open: bool) -> bool {
        match self.by_net.get(net_id) {
            Some(i) => {
                self.channels[*i].user_open = open;
                self.changes += 1;
                true
            }
            None => false,
        }
    }

    pub fn set_channel_lock(&mut self, net_id: &str, lock: bool) -> bool {
        match self.by_net.get(net_id) {
            Some(i) => {
                self.channels[*i].user_lock = lock;
                self.changes += 1;
                true
            }
            None => false,
        }
    }

    pub fn reset_channel(&mut self, net_id: &str) -> bool {
        match self.by_net.get(net_id) {
            Some(i) => {
                self.channels[*i].reset_steps = RESET_HOLD_STEPS;
                self.channels[*i].user_open = false;
                self.changes += 1;
                true
            }
            None => false,
        }
    }

    pub fn set_comms_loss(&mut self, rpdu: &str, lost: bool) -> bool {
        match self.rpdus.iter_mut().find(|r| r.id == rpdu) {
            Some(r) => {
                r.forced_comms_loss = lost;
                true
            }
            None => false,
        }
    }

    pub fn set_controller_power_loss(&mut self, rpdu: &str, lost: bool) -> bool {
        match self.rpdus.iter_mut().find(|r| r.id == rpdu) {
            Some(r) => {
                r.forced_power_loss = lost;
                true
            }
            None => false,
        }
    }

    pub fn inject_overload(&mut self, net_id: &str, extra_a: f64) -> bool {
        match self.by_net.get(net_id) {
            Some(i) => {
                self.channels[*i].inject_a = extra_a;
                true
            }
            None => false,
        }
    }

    pub fn inject_arc(&mut self, net_id: &str, noise_a: f64) -> bool {
        match self.by_net.get(net_id) {
            Some(i) => {
                self.channels[*i].noise_a = noise_a;
                true
            }
            None => false,
        }
    }

    pub fn queue(&mut self, cmds: Vec<BreakerCommand>, cbic: &mut Cbic) -> Vec<BreakerCommand> {
        let mut rest = Vec::new();
        for c in cmds {
            let Some(i) = self.by_breaker.get(&c.id).copied() else {
                rest.push(c);
                continue;
            };
            let ch = &mut self.channels[i];
            match c.action {
                CbAction::Open => {
                    ch.user_open = true;
                    cbic.report_status(c.id, CbState::Open, 0.0);
                }
                CbAction::Close => ch.user_open = false,
                CbAction::Lock(_) => {
                    ch.user_open = true;
                    ch.user_lock = true;
                }
                CbAction::Unlock => ch.user_lock = false,
                CbAction::RequestReset => {
                    ch.reset_steps = RESET_HOLD_STEPS;
                    ch.user_open = false;
                }
            }
            self.changes += 1;
        }
        rest
    }

    pub(crate) fn activity(&self) -> f64 {
        self.changes as f64 * CHANGE_WEIGHT
    }

    fn note(c: &mut LinkChannel, now_us: u64, text: &str) {
        if c.history.len() >= HISTORY_LEN {
            c.history.remove(0);
        }
        c.history.push((now_us as f64 * 1e-6, text.to_string()));
    }

    pub(crate) fn step(&mut self, inp: &LinkInputs, cbic: &mut Cbic) -> Vec<(String, bool)> {
        let snap = inp.snap;
        let dt = inp.dt_us;
        let mut net_writes: Vec<(String, bool)> = Vec::new();

        let n = self.rpdus.len();
        let mut gateway_ok = vec![true; n];
        for i in 0..n {
            if self.rpdus[i].gateway {
                gateway_ok[i] = inp.cdn_ok && !self.rpdus[i].forced_comms_loss;
            }
        }
        let node_v: Vec<f64> = self
            .rpdus
            .iter()
            .map(|r| snap.bus(&r.dc_node).map(|b| b.voltage_v).unwrap_or(0.0))
            .collect();
        let powered_prev: Vec<bool> = self.rpdus.iter().map(|r| r.powered).collect();

        let mut cmds: Vec<Vec<ChannelCommand>> = vec![Vec::new(); n];
        let mut loads: Vec<Vec<ChannelLoad>> = vec![Vec::new(); n];
        let mut spdu_status: Vec<(usize, crate::pdu::channel::ChannelStatus)> = Vec::new();

        for ci in 0..self.channels.len() {
            let c = &mut self.channels[ci];
            c.shed = c.shed_group.map(|g| inp.shed[g]).unwrap_or(false);
            let reset = c.reset_steps > 0;
            let close = !c.user_open && !c.shed && !reset && (!inp.ground_battery_only || c.battery_only);
            let cmd = ChannelCommand {
                addr: c.addr,
                close,
                source: if c.shed {
                    CommandSource::LoadShed
                } else {
                    CommandSource::System
                },
                reset,
                lock: Some(c.user_lock),
            };
            if reset {
                c.reset_steps -= 1;
            }
            let raw_a = c
                .net_index
                .and_then(|i| snap.sspcs.get(i))
                .map(|s| s.current_a)
                .unwrap_or(0.0);
            let load = ChannelLoad {
                addr: c.addr,
                currents_a: [raw_a + c.inject_a, 0.0, 0.0],
                noise_a: c.noise_a,
            };
            match c.dev {
                Dev::Rpdu(d) => {
                    let send = self.rpdus[d].resend || c.sent != Some((close, c.user_lock)) || reset;
                    if send {
                        cmds[d].push(cmd);
                        c.sent = Some((close, c.user_lock));
                    }
                    if load.currents_a[0] != 0.0 || load.noise_a != 0.0 {
                        loads[d].push(load);
                    }
                }
                Dev::Spdu(s) => {
                    let v_in = snap.bus(&c.src_bus).map(|b| b.voltage_v).unwrap_or(0.0);
                    if let Some(ch) = self.spdus[s].channel_mut(c.addr) {
                        ch.apply(&cmd);
                        let st = ch.step(c.addr, dt, v_in, Some(&load), false);
                        spdu_status.push((ci, st));
                    }
                }
            }
        }

        let mut statuses: Vec<Option<crate::pdu::rpdu::RpduOutput>> = (0..n).map(|_| None).collect();
        for d in 0..n {
            let (cluster_ok, backup_v, forced_power) = {
                let r = &self.rpdus[d];
                let cl = r.cluster_gateway.map(|g| gateway_ok[g]).unwrap_or(true);
                let bv = r
                    .backup
                    .filter(|b| powered_prev[*b] && !self.rpdus[*b].forced_power_loss)
                    .map(|b| node_v[b])
                    .unwrap_or(0.0);
                (cl, bv, r.forced_power_loss)
            };
            let r = &mut self.rpdus[d];
            let dc_v = if forced_power { 0.0 } else { node_v[d] };
            let ac_v = snap.bus(&r.ac_node).map(|b| b.voltage_v).unwrap_or(0.0);
            let comms_ok = cluster_ok && !r.forced_comms_loss;
            let input = RpduInput {
                dt_us: dt,
                ac_v,
                dc1_v: dc_v,
                dc2_v: dc_v,
                backup_28v: if forced_power { 0.0 } else { backup_v },
                comms_ok,
                commands: std::mem::take(&mut cmds[d]),
                loads: std::mem::take(&mut loads[d]),
            };
            let out = r.rpdu.step(&input);
            r.resend = !comms_ok || !out.controller_powered;
            r.comms_ok = comms_ok;
            r.powered = out.controller_powered;
            r.default_active = out.default_active;
            r.input_v = dc_v;
            statuses[d] = Some(out);
        }

        let mut sums = vec![0.0; n];
        for ci in 0..self.channels.len() {
            let (state_new, health, v_out, current_a, default_active, heat, dev_comms) = {
                let c = &self.channels[ci];
                match c.dev {
                    Dev::Rpdu(d) => {
                        let out = statuses[d].as_ref().unwrap();
                        let st = &out.statuses[c.status_index];
                        let heat = self.rpdus[d]
                            .rpdu
                            .modules
                            .get((c.addr.slot - FIRST_PM_SLOT) as usize)
                            .and_then(|m| m.channels.get(c.addr.channel as usize))
                            .map(|ch| ch.sspc.heat_fraction())
                            .unwrap_or(0.0);
                        (
                            st.state,
                            st.telemetry.health,
                            st.v_out,
                            st.current_a,
                            st.default_active,
                            heat,
                            self.rpdus[d].comms_ok && self.rpdus[d].powered,
                        )
                    }
                    Dev::Spdu(_) => {
                        let st = &spdu_status.iter().find(|(i, _)| *i == ci).unwrap().1;
                        let heat = match c.dev {
                            Dev::Spdu(s) => self.spdus[s]
                                .slots
                                .get(c.addr.slot as usize)
                                .and_then(|x| x.as_ref())
                                .and_then(|card| card.channels.get(c.addr.channel as usize))
                                .map(|ch| ch.sspc.heat_fraction())
                                .unwrap_or(0.0),
                            _ => 0.0,
                        };
                        (
                            st.state,
                            st.telemetry.health,
                            st.v_out,
                            st.current_a,
                            false,
                            heat,
                            true,
                        )
                    }
                }
            };
            let c = &mut self.channels[ci];
            if state_new != c.state {
                self.changes += 1;
                if state_new == SspcState::Tripped {
                    c.trips += 1;
                    if health == SspcHealth::ArcFaultDetected {
                        c.arc_events += 1;
                        Self::note(c, inp.now_us, "ARC TRIP");
                    } else {
                        Self::note(c, inp.now_us, "I2T TRIP");
                    }
                } else {
                    let text = match state_new {
                        SspcState::Open => "OPEN",
                        SspcState::Closed => "CLOSED",
                        SspcState::LockedOpen => "LOCKED",
                        SspcState::Tripped => "TRIP",
                    };
                    Self::note(c, inp.now_us, text);
                }
            }
            if default_active != c.default_active {
                self.changes += 1;
            }
            c.state = state_new;
            c.health = health;
            c.v_out = v_out;
            c.current_a = current_a;
            c.default_active = default_active;
            c.i2t_pct = heat * 100.0;
            if let Dev::Rpdu(d) = c.dev {
                sums[d] += current_a;
            }
            let closed = state_new == SspcState::Closed;
            if c.net_index.is_some() && !c.held && closed != c.net_closed {
                c.net_closed = closed;
                net_writes.push((c.net_id.clone(), closed));
            }
            if let Some(id) = c.breaker_id {
                if dev_comms {
                    let reported = match state_new {
                        SspcState::Open => CbState::Open,
                        SspcState::Closed => CbState::Closed,
                        SspcState::Tripped => CbState::Tripped,
                        SspcState::LockedOpen => CbState::Locked(LockTag::DoNotClose),
                    };
                    cbic.report_status(id, reported, current_a);
                } else {
                    cbic.report_comm_loss(id);
                }
            }
        }
        for d in 0..n {
            self.rpdus[d].input_a = sums[d];
        }
        net_writes
    }

    pub fn rpdu_rows(&self) -> Vec<RpduRow> {
        let mut rows = Vec::new();
        for (d, r) in self.rpdus.iter().enumerate() {
            let mut list: Vec<&LinkChannel> =
                self.channels.iter().filter(|c| c.dev == Dev::Rpdu(d)).collect();
            list.sort_by_key(|c| (c.addr.slot, c.addr.channel));
            let channels = list
                .iter()
                .enumerate()
                .map(|(k, c)| ChannelRow {
                    channel: k as u32 + 1,
                    slot: c.addr.slot,
                    index: c.addr.channel,
                    net_id: c.net_id.clone(),
                    load: c.load.clone(),
                    rating_a: c.rating_a,
                    rating_source: c.rating_source,
                    state: c.state,
                    health: c.health,
                    current_a: c.current_a,
                    voltage_out_v: c.v_out,
                    i2t_pct: c.i2t_pct,
                    arc_events: c.arc_events,
                    trips: c.trips,
                    shed: c.shed,
                    default_active: c.default_active,
                    ac: c.ac,
                    instant_trip_a: c.rating_a * INSTANT_TRIP_MULTIPLE,
                    history: c.history.clone(),
                })
                .collect();
            rows.push(RpduRow {
                id: r.id.clone(),
                location: r.location.clone(),
                gateway: r.gateway,
                input_v: r.input_v,
                input_a: r.input_a,
                powered: r.powered,
                comms_ok: r.comms_ok,
                default_active: r.default_active,
                channels,
            });
        }
        rows
    }

    #[cfg(feature = "inspector")]
    pub fn inspector_rpdus(&self) -> Vec<crate::inspector::RpduView> {
        use crate::inspector::{ChannelState, ChannelView, RpduView};
        self.rpdu_rows()
            .into_iter()
            .map(|r| RpduView {
                id: r.id,
                location: r.location,
                input_v: r.input_v,
                input_a: r.input_a,
                powered: r.powered,
                channels: r
                    .channels
                    .into_iter()
                    .map(|c| ChannelView {
                        channel: c.channel,
                        rating_a: c.rating_a,
                        load: c.load,
                        state: match c.state {
                            SspcState::Closed => ChannelState::On,
                            SspcState::Tripped if c.health == SspcHealth::ArcFaultDetected => {
                                ChannelState::TrippedArc
                            }
                            SspcState::Tripped => ChannelState::Tripped,
                            SspcState::Open if c.shed => ChannelState::Shed,
                            _ => ChannelState::Off,
                        },
                        current_a: c.current_a,
                        voltage_out_v: c.voltage_out_v,
                        i2t_pct: c.i2t_pct,
                        arc_events: c.arc_events,
                        instant_trip_a: c.instant_trip_a,
                        trips_life: c.trips,
                        history: c.history,
                        ..ChannelView::default()
                    })
                    .collect(),
            })
            .collect()
    }
}

impl Electrical {
    pub fn pdu(&self) -> &PduLink {
        &self.pdu
    }

    pub fn pdu_mut(&mut self) -> &mut PduLink {
        &mut self.pdu
    }

    pub fn rpdu_views(&self) -> Vec<RpduRow> {
        self.pdu.rpdu_rows()
    }

    #[cfg(feature = "inspector")]
    pub fn inspector_data(&self) -> crate::inspector::InspectorData {
        let mut d = match self.plant.snapshot() {
            Some(s) => crate::inspector::InspectorData::from_net_snapshot(s),
            None => crate::inspector::InspectorData::default(),
        };
        d.sim_time_s = self.now_us as f64 * 1e-6;
        d.rpdus = self.pdu.inspector_rpdus();
        d
    }

    pub(crate) fn stage_pdu(&mut self, dt_us: u64) {
        let Some(snap) = self.plant.snapshot() else {
            return;
        };
        let input = LinkInputs {
            dt_us,
            now_us: self.now_us,
            snap,
            shed: self.lm_out.shed,
            cdn_ok: !(self.faults.bpcu_fail[0] && self.faults.bpcu_fail[1]),
            ground_battery_only: self.mode() == "on_ground_battery_only",
        };
        let writes = self.pdu.step(&input, &mut self.cbic.app);
        for (id, closed) in writes {
            self.plant.set_sspc(&id, closed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::ElectricalInputs;

    fn sys() -> Electrical {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/24-electrical");
        Electrical::new(&dir).expect("system builds")
    }

    fn flying() -> ElectricalInputs {
        ElectricalInputs {
            engine_rpm: [7600.0; 2],
            engine_running: [true; 2],
            on_ground: false,
            radio_alt_ft: 20000.0,
            ..ElectricalInputs::default()
        }
    }

    fn run(e: &mut Electrical, seconds: f64) {
        let inputs = flying();
        for _ in 0..(seconds * 100.0) as u64 {
            e.step(10_000, &inputs);
        }
    }

    fn powered() -> Electrical {
        let mut e = sys();
        for i in 0..4 {
            e.panel_mut().gen_ctrl[i].set_latched(true);
        }
        e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
        run(&mut e, 20.0);
        e
    }

    fn pick(e: &Electrical, rpdu: &str, rating: f64) -> LinkChannel {
        e.pdu()
            .channels()
            .iter()
            .find(|c| {
                c.net_id.starts_with(rpdu)
                    && !c.ac
                    && c.rating_a == rating
                    && c.shed_group.is_none()
                    && c.state == SspcState::Closed
            })
            .cloned()
            .expect("closed dc channel")
    }

    fn state(e: &Electrical, id: &str) -> SspcState {
        e.pdu().channel(id).unwrap().state
    }

    #[test]
    fn every_net_channel_is_mapped_with_rating_sources() {
        let e = sys();
        let m = e.pdu().summary();
        assert_eq!(m.rpdu_channels, 128);
        assert!(m.feed_channels >= 23, "{}", m.feed_channels);
        assert_eq!(m.unmapped_net, 0);
        assert_eq!(m.rating_from_breakers + m.rating_approx, m.total);
        assert!(m.rating_from_breakers >= 23);
        assert_eq!(e.pdu().rpdu_ids().len(), 17);
        for id in e.plant.net().sspc_ids().iter().filter(|i| i.starts_with("RPDU")) {
            assert!(e.pdu().channel(id).is_some(), "{id}");
        }
        let rows = e.rpdu_views();
        assert_eq!(rows.len(), 17);
        assert_eq!(rows.iter().filter(|r| r.gateway).count(), 4);
        assert_eq!(rows.iter().map(|r| r.channels.len()).sum::<usize>(), 128);
    }

    #[test]
    fn channels_close_when_the_rpdu_powers_up() {
        let e = powered();
        let closed = e
            .pdu()
            .channels()
            .iter()
            .filter(|c| c.net_id.starts_with("RPDU") && c.state == SspcState::Closed)
            .count();
        assert!(closed > 60, "{closed}");
        assert_eq!(state(&e, "CE1471831"), SspcState::Closed);
        assert_eq!(e.pdu().rpdu_powered("RPDU 73"), Some(true));
    }

    #[test]
    fn overload_trips_at_i2t_time_and_load_goes_dead() {
        let mut e = powered();
        let ch = pick(&e, "RPDU73.", 2.5);
        assert!(e.plant.load_power(&ch.load) > 1.0);
        e.plant.set_load(&ch.load, 28.0 * 6.0);
        let mut t = 0.0;
        let inputs = flying();
        while state(&e, &ch.net_id) == SspcState::Closed && t < 10.0 {
            e.step(10_000, &inputs);
            t += 0.01;
        }
        assert_eq!(state(&e, &ch.net_id), SspcState::Tripped);
        assert!((1.5..3.5).contains(&t), "{t}");
        run(&mut e, 0.05);
        assert!(e.plant.load_power(&ch.load) < 0.5);
        assert_eq!(e.pdu().channel(&ch.net_id).unwrap().trips, 1);
        e.plant.set_load(&ch.load, 15.0);
        run(&mut e, 2.0);
        assert_eq!(state(&e, &ch.net_id), SspcState::Tripped);
        e.pdu_mut().reset_channel(&ch.net_id);
        run(&mut e, 0.5);
        assert_eq!(state(&e, &ch.net_id), SspcState::Closed);
        assert!(e.plant.load_power(&ch.load) > 1.0);
    }

    #[test]
    fn arc_fault_input_trips_the_channel() {
        let mut e = powered();
        let ch = pick(&e, "RPDU73.", 5.0);
        e.plant.set_load(&ch.load, 28.0 * 1.5);
        run(&mut e, 0.2);
        assert_eq!(state(&e, &ch.net_id), SspcState::Closed);
        e.pdu_mut().inject_arc(&ch.net_id, 0.8);
        run(&mut e, 0.3);
        let c = e.pdu().channel(&ch.net_id).unwrap();
        assert_eq!(c.state, SspcState::Tripped);
        assert_eq!(c.health, SspcHealth::ArcFaultDetected);
        assert_eq!(c.arc_events, 1);
        assert!(e.plant.load_power(&ch.load) < 0.5);
    }

    #[test]
    fn load_shed_command_opens_the_channel() {
        let mut e = powered();
        let id = e
            .pdu()
            .channels()
            .iter()
            .find(|c| c.shed_group == Some(7))
            .map(|c| c.net_id.clone())
            .unwrap();
        for k in 0..3 {
            let snap = e.plant.snapshot().unwrap().clone();
            let mut shed = [false; 14];
            shed[7] = true;
            let input = LinkInputs {
                dt_us: 10_000,
                now_us: e.now_us + k,
                snap: &snap,
                shed,
                cdn_ok: true,
                ground_battery_only: false,
            };
            e.pdu.step(&input, &mut e.cbic.app);
        }
        let c = e.pdu().channel(&id).unwrap();
        assert!(c.shed);
        assert_eq!(c.state, SspcState::Open);
        let snap = e.plant.snapshot().unwrap().clone();
        let input = LinkInputs {
            dt_us: 10_000,
            now_us: e.now_us,
            snap: &snap,
            shed: [false; 14],
            cdn_ok: true,
            ground_battery_only: false,
        };
        for _ in 0..3 {
            e.pdu.step(&input, &mut e.cbic.app);
        }
        assert!(!e.pdu().channel(&id).unwrap().shed);
    }

    fn ground_run(e: &mut Electrical, base: &ElectricalInputs, steps: usize) {
        for _ in 0..steps {
            let mut inp = base.clone();
            inp.cbic_session = e.inputs().cbic_session;
            e.step(10_000, &inp);
        }
    }

    #[test]
    fn cbic_lock_prevents_close_and_open_removes_power() {
        let mut e = powered();
        let ground = ElectricalInputs {
            on_ground: true,
            doors_open: true,
            radio_alt_ft: 0.0,
            ..flying()
        };
        ground_run(&mut e, &ground, 200);
        e.action("crew", "open_page", "CBIC", None).unwrap();
        e.action("crew", "cbic_open", "CE1471831", None).unwrap();
        ground_run(&mut e, &ground, 10);
        assert_eq!(state(&e, "CE1471831"), SspcState::Open);
        assert!(e.plant.snapshot().unwrap().sspc("CE1471831").unwrap().current_a < 0.01);
        e.action("crew", "cbic_lock", "CE1471831", None).unwrap();
        ground_run(&mut e, &ground, 10);
        assert_eq!(state(&e, "CE1471831"), SspcState::LockedOpen);
        e.action("crew", "cbic_close", "CE1471831", None).unwrap();
        ground_run(&mut e, &ground, 20);
        assert_eq!(state(&e, "CE1471831"), SspcState::LockedOpen);
        assert_eq!(e.flag("elec.cb.CE1471831.locked"), Some(true));
        e.action("crew", "cbic_unlock", "CE1471831", None).unwrap();
        e.action("crew", "cbic_close", "CE1471831", None).unwrap();
        ground_run(&mut e, &ground, 20);
        assert_eq!(state(&e, "CE1471831"), SspcState::Closed);
        assert_eq!(e.text("elec.cb.CE1471831.state").as_deref(), Some("closed"));
    }

    fn set_default_off(e: &mut Electrical, id: &str) {
        let i = *e.pdu.by_net.get(id).unwrap();
        let (dev, addr) = (e.pdu.channels[i].dev, e.pdu.channels[i].addr);
        if let Dev::Rpdu(d) = dev {
            e.pdu.rpdus[d].rpdu.set_default(addr, DefaultState::Off);
        }
    }

    #[test]
    fn comms_loss_defaults_after_three_seconds() {
        let mut e = powered();
        let ch = pick(&e, "RPDU73.", 2.5);
        set_default_off(&mut e, &ch.net_id);
        e.pdu_mut().set_comms_loss("RPDU 73", true);
        run(&mut e, 2.8);
        assert_eq!(state(&e, &ch.net_id), SspcState::Closed);
        assert_eq!(e.pdu().rpdu_default_active("RPDU 73"), Some(false));
        run(&mut e, 0.5);
        assert_eq!(state(&e, &ch.net_id), SspcState::Open);
        assert_eq!(e.pdu().rpdu_default_active("RPDU 73"), Some(true));
        assert!(e.plant.load_power(&ch.load) < 0.5);
        e.pdu_mut().set_comms_loss("RPDU 73", false);
        run(&mut e, 0.5);
        assert_eq!(state(&e, &ch.net_id), SspcState::Closed);
    }

    #[test]
    fn controller_power_loss_defaults_immediately() {
        let mut e = powered();
        let ch = pick(&e, "RPDU73.", 2.5);
        set_default_off(&mut e, &ch.net_id);
        e.pdu_mut().set_controller_power_loss("RPDU 73", true);
        run(&mut e, 0.03);
        assert_eq!(state(&e, &ch.net_id), SspcState::Closed);
        run(&mut e, 0.1);
        assert_eq!(e.pdu().rpdu_powered("RPDU 73"), Some(false));
        assert_eq!(state(&e, &ch.net_id), SspcState::Open);
    }

    #[test]
    fn identical_runs_give_identical_device_state() {
        let one = || {
            let mut e = powered();
            let ch = pick(&e, "RPDU73.", 2.5);
            e.plant.set_load(&ch.load, 28.0 * 6.0);
            run(&mut e, 4.0);
            format!("{:?}", e.rpdu_views())
        };
        assert_eq!(one(), one());
    }

    #[cfg(feature = "inspector")]
    #[test]
    fn inspector_rows_are_live() {
        let e = powered();
        let d = e.inspector_data();
        assert_eq!(d.rpdus.len(), 17);
        let v = d.rpdu("RPDU 73").unwrap();
        assert!(v.powered);
        assert!(
            v.channels
                .iter()
                .any(|c| c.state == crate::inspector::ChannelState::On)
        );
    }
}
