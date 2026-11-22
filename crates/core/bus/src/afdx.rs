use std::collections::BTreeMap;

pub const LINK_BPS: u64 = 100_000_000;
pub const NS_PER_BYTE: u64 = 8_000_000_000 / LINK_BPS;
pub const ETH_HEADER: u32 = 14;
pub const IP_HEADER: u32 = 20;
pub const UDP_HEADER: u32 = 8;
pub const FCS: u32 = 4;
pub const PREAMBLE_IFG: u32 = 20;
pub const MIN_FRAME: u32 = 64;
pub const OVERHEAD: u32 = ETH_HEADER + IP_HEADER + UDP_HEADER + FCS;

pub fn frame_bytes(payload: u32) -> u32 {
    (payload + OVERHEAD).max(MIN_FRAME)
}

pub fn wire_bytes(payload: u32) -> u32 {
    frame_bytes(payload) + PREAMBLE_IFG
}

pub fn wire_time_ns(payload: u32) -> u64 {
    u64::from(wire_bytes(payload)) * NS_PER_BYTE
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtualLink {
    pub id: u16,
    pub bag_us: u32,
    pub lmax: u32,
    pub priority: Priority,
}

impl VirtualLink {
    pub fn new(id: u16, bag_us: u32, lmax: u32, priority: Priority) -> Self {
        VirtualLink {
            id,
            bag_us,
            lmax,
            priority,
        }
    }
    pub fn max_payload(&self) -> u32 {
        self.lmax.saturating_sub(OVERHEAD)
    }
    pub fn max_bandwidth_bps(&self) -> u64 {
        u64::from(self.lmax) * 8 * 1_000_000 / u64::from(self.bag_us)
    }
}

#[derive(Clone, Debug, Default)]
pub struct EndSystem {
    vl_next_ns: BTreeMap<u16, u64>,
    port_free_ns: [u64; 2],
}

impl EndSystem {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn schedule(&mut self, vl: &VirtualLink, net: usize, payload: u32, ready_ns: u64) -> (u64, u64) {
        let next = self.vl_next_ns.get(&vl.id).copied().unwrap_or(0);
        let start = ready_ns.max(next).max(self.port_free_ns[net & 1]);
        let end = start + wire_time_ns(payload);
        self.port_free_ns[net & 1] = end;
        (start, end)
    }
    pub fn commit(&mut self, vl: &VirtualLink, start_ns: u64) {
        self.vl_next_ns
            .insert(vl.id, start_ns + u64::from(vl.bag_us) * 1000);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub vl: u16,
    pub seq: u8,
    pub net: u8,
    pub priority: Priority,
    pub payload: Vec<u8>,
    pub order: u64,
}

#[derive(Clone, Debug)]
struct Queued {
    port: u32,
    avail_ns: u64,
    frame: Frame,
}

#[derive(Clone, Debug)]
pub struct Switch {
    pub fabric_latency_ns: u64,
    pub failed: bool,
    queue: Vec<Queued>,
    port_free_ns: BTreeMap<u32, u64>,
}

impl Switch {
    pub fn new(fabric_latency_ns: u64) -> Self {
        Switch {
            fabric_latency_ns,
            failed: false,
            queue: Vec::new(),
            port_free_ns: BTreeMap::new(),
        }
    }
    pub fn enqueue(&mut self, port: u32, rx_complete_ns: u64, frame: Frame) {
        if self.failed {
            return;
        }
        self.queue.push(Queued {
            port,
            avail_ns: rx_complete_ns + self.fabric_latency_ns,
            frame,
        });
    }
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }
    pub fn run(&mut self, until_ns: u64) -> Vec<(u32, Frame, u64)> {
        let mut out = Vec::new();
        loop {
            let mut pick: Option<(u64, usize)> = None;
            let mut ports: Vec<u32> = self.queue.iter().map(|q| q.port).collect();
            ports.sort_unstable();
            ports.dedup();
            for port in ports {
                let free = self.port_free_ns.get(&port).copied().unwrap_or(0);
                let earliest = self
                    .queue
                    .iter()
                    .filter(|q| q.port == port)
                    .map(|q| q.avail_ns)
                    .min()
                    .unwrap_or(0);
                let start = free.max(earliest);
                if start > until_ns {
                    continue;
                }
                let mut best: Option<usize> = None;
                for (i, q) in self.queue.iter().enumerate() {
                    if q.port != port || q.avail_ns > start {
                        continue;
                    }
                    let better = match best {
                        None => true,
                        Some(b) => {
                            let o = &self.queue[b];
                            (q.frame.priority, q.avail_ns, q.frame.order)
                                < (o.frame.priority, o.avail_ns, o.frame.order)
                        }
                    };
                    if better {
                        best = Some(i);
                    }
                }
                if let Some(i) = best {
                    match pick {
                        Some((s, _)) if s <= start => {}
                        _ => pick = Some((start, i)),
                    }
                }
            }
            let Some((start, i)) = pick else { break };
            let q = self.queue.remove(i);
            let end = start + wire_time_ns(q.frame.payload.len() as u32);
            self.port_free_ns.insert(q.port, end);
            out.push((q.port, q.frame, end));
        }
        out
    }
}

#[derive(Clone, Debug, Default)]
pub struct RedundancyManager {
    last: BTreeMap<(u32, u16), u8>,
    pub window: u8,
    pub discarded: u64,
}

impl RedundancyManager {
    pub fn new(window: u8) -> Self {
        RedundancyManager {
            last: BTreeMap::new(),
            window,
            discarded: 0,
        }
    }
    pub fn accept(&mut self, dest: u32, vl: u16, seq: u8) -> bool {
        let key = (dest, vl);
        let ok = match self.last.get(&key) {
            None => true,
            Some(last) => {
                let d = seq.wrapping_sub(*last);
                d >= 1 && d <= self.window
            }
        };
        if ok {
            self.last.insert(key, seq);
        } else {
            self.discarded += 1;
        }
        ok
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub time_us: u64,
    pub vl: u16,
    pub dest: u32,
    pub net: u8,
    pub seq: u8,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug)]
struct Route {
    vl: VirtualLink,
    src: u32,
    dests: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct AfdxNetwork {
    routes: BTreeMap<u16, Route>,
    end_systems: BTreeMap<u32, EndSystem>,
    pub switches: [Switch; 2],
    seqs: BTreeMap<u16, u8>,
    rm: RedundancyManager,
    order: u64,
    pub rejected_oversize: u64,
}

impl AfdxNetwork {
    pub fn new(fabric_latency_ns: u64) -> Self {
        AfdxNetwork {
            routes: BTreeMap::new(),
            end_systems: BTreeMap::new(),
            switches: [Switch::new(fabric_latency_ns), Switch::new(fabric_latency_ns)],
            seqs: BTreeMap::new(),
            rm: RedundancyManager::new(127),
            order: 0,
            rejected_oversize: 0,
        }
    }
    pub fn add_virtual_link(&mut self, vl: VirtualLink, src: u32, dests: &[u32]) {
        self.end_systems.entry(src).or_default();
        self.routes.insert(
            vl.id,
            Route {
                vl,
                src,
                dests: dests.to_vec(),
            },
        );
    }
    pub fn set_network_failed(&mut self, net: usize, failed: bool) {
        self.switches[net & 1].failed = failed;
    }
    pub fn redundancy_discards(&self) -> u64 {
        self.rm.discarded
    }
    pub fn send(&mut self, vl: u16, payload: &[u8], now_us: u64) -> bool {
        let Some(route) = self.routes.get(&vl).cloned() else {
            return false;
        };
        if payload.len() as u32 > route.vl.max_payload() {
            self.rejected_oversize += 1;
            return false;
        }
        let seq = self.seqs.entry(vl).or_insert(0);
        *seq = seq.wrapping_add(1);
        let seq = *seq;
        self.order += 1;
        let order = self.order;
        let es = self.end_systems.entry(route.src).or_default();
        let ready = now_us * 1000;
        let mut start_a = u64::MAX;
        for net in 0..2usize {
            let (start, end) = es.schedule(&route.vl, net, payload.len() as u32, ready);
            start_a = start_a.min(start);
            for &d in &route.dests {
                self.switches[net].enqueue(
                    d,
                    end,
                    Frame {
                        vl,
                        seq,
                        net: net as u8,
                        priority: route.vl.priority,
                        payload: payload.to_vec(),
                        order,
                    },
                );
            }
        }
        es.commit(&route.vl, start_a);
        true
    }
    pub fn poll(&mut self, until_us: u64) -> Vec<Delivery> {
        let until_ns = until_us * 1000;
        let mut raw: Vec<(u64, u16, u8, u8, u32, Vec<u8>)> = Vec::new();
        for sw in &mut self.switches {
            for (port, f, end) in sw.run(until_ns) {
                raw.push((end.div_ceil(1000), f.vl, f.seq, f.net, port, f.payload));
            }
        }
        raw.sort_by_key(|a| (a.0, a.1, a.2, a.3, a.4));
        let mut out = Vec::new();
        for (t, vl, seq, net, dest, payload) in raw {
            if self.rm.accept(dest, vl, seq) {
                out.push(Delivery {
                    time_us: t,
                    vl,
                    dest,
                    net,
                    seq,
                    payload,
                });
            }
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionalStatus {
    NormalOperation,
    NoComputedData,
    FunctionalTest,
    FailureWarning,
}

impl FunctionalStatus {
    pub fn code(self) -> u8 {
        match self {
            FunctionalStatus::NormalOperation => 0x03,
            FunctionalStatus::NoComputedData => 0x0C,
            FunctionalStatus::FunctionalTest => 0x30,
            FunctionalStatus::FailureWarning => 0x00,
        }
    }
    pub fn from_code(c: u8) -> Self {
        match c {
            0x03 => FunctionalStatus::NormalOperation,
            0x0C => FunctionalStatus::NoComputedData,
            0x30 => FunctionalStatus::FunctionalTest,
            _ => FunctionalStatus::FailureWarning,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParamEncoding {
    Bool,
    Unsigned,
    Signed,
    Float32,
    Float64,
    Scaled { signed: bool, resolution: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParamDef {
    pub name: String,
    pub bit_offset: u32,
    pub bit_len: u32,
    pub encoding: ParamEncoding,
    pub fs_index: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MessageLayout {
    pub size_bytes: usize,
    pub fs_base_byte: usize,
    pub params: Vec<ParamDef>,
}

pub fn write_bits(buf: &mut [u8], offset: u32, len: u32, value: u64) {
    for i in 0..len {
        let bit = (value >> (len - 1 - i)) & 1;
        let pos = offset + i;
        let byte = (pos / 8) as usize;
        let sh = 7 - (pos % 8);
        if byte < buf.len() {
            buf[byte] = (buf[byte] & !(1 << sh)) | ((bit as u8) << sh);
        }
    }
}

pub fn read_bits(buf: &[u8], offset: u32, len: u32) -> u64 {
    let mut v = 0u64;
    for i in 0..len {
        let pos = offset + i;
        let byte = (pos / 8) as usize;
        let sh = 7 - (pos % 8);
        let b = buf.get(byte).map_or(0, |x| (x >> sh) & 1);
        v = (v << 1) | u64::from(b);
    }
    v
}

fn sign_extend(v: u64, len: u32) -> i64 {
    if len == 0 || len >= 64 {
        return v as i64;
    }
    let s = 1u64 << (len - 1);
    if v & s != 0 {
        (v | !((1u64 << len) - 1)) as i64
    } else {
        v as i64
    }
}

impl ParamDef {
    pub fn new(name: &str, bit_offset: u32, bit_len: u32, encoding: ParamEncoding) -> Self {
        ParamDef {
            name: name.to_string(),
            bit_offset,
            bit_len,
            encoding,
            fs_index: None,
        }
    }
    pub fn with_fs(mut self, idx: u32) -> Self {
        self.fs_index = Some(idx);
        self
    }
    fn mask(&self) -> u64 {
        if self.bit_len >= 64 {
            u64::MAX
        } else {
            (1u64 << self.bit_len) - 1
        }
    }
    fn encode_raw(&self, v: f64) -> u64 {
        match self.encoding {
            ParamEncoding::Bool => u64::from(v != 0.0),
            ParamEncoding::Unsigned => (v.max(0.0).round() as u64).min(self.mask()),
            ParamEncoding::Signed => {
                let half = 1i64 << (self.bit_len.min(63) - 1);
                ((v.round() as i64).clamp(-half, half - 1) as u64) & self.mask()
            }
            ParamEncoding::Float32 => u64::from((v as f32).to_bits()),
            ParamEncoding::Float64 => v.to_bits(),
            ParamEncoding::Scaled { signed, resolution } => {
                let c = (v / resolution).round();
                if signed {
                    let half = 1i64 << (self.bit_len.min(63) - 1);
                    ((c as i64).clamp(-half, half - 1) as u64) & self.mask()
                } else {
                    (c.max(0.0) as u64).min(self.mask())
                }
            }
        }
    }
    fn decode_raw(&self, raw: u64) -> f64 {
        match self.encoding {
            ParamEncoding::Bool => f64::from(u8::from(raw != 0)),
            ParamEncoding::Unsigned => raw as f64,
            ParamEncoding::Signed => sign_extend(raw, self.bit_len) as f64,
            ParamEncoding::Float32 => f64::from(f32::from_bits(raw as u32)),
            ParamEncoding::Float64 => f64::from_bits(raw),
            ParamEncoding::Scaled { signed, resolution } => {
                if signed {
                    sign_extend(raw, self.bit_len) as f64 * resolution
                } else {
                    raw as f64 * resolution
                }
            }
        }
    }
}

impl MessageLayout {
    pub fn new(size_bytes: usize, fs_base_byte: usize) -> Self {
        MessageLayout {
            size_bytes,
            fs_base_byte,
            params: Vec::new(),
        }
    }
    pub fn add(&mut self, p: ParamDef) {
        self.params.push(p);
    }
    pub fn encode(&self, values: &[(f64, FunctionalStatus)]) -> Vec<u8> {
        let mut buf = vec![0u8; self.size_bytes];
        for (p, (v, fs)) in self.params.iter().zip(values.iter()) {
            write_bits(&mut buf, p.bit_offset, p.bit_len, p.encode_raw(*v));
            if let Some(i) = p.fs_index {
                let idx = self.fs_base_byte + i as usize;
                if idx < buf.len() {
                    buf[idx] = fs.code();
                }
            }
        }
        buf
    }
    pub fn decode(&self, payload: &[u8]) -> Vec<(f64, FunctionalStatus)> {
        self.params
            .iter()
            .map(|p| {
                let raw = read_bits(payload, p.bit_offset, p.bit_len);
                let fs = match p.fs_index {
                    Some(i) => payload
                        .get(self.fs_base_byte + i as usize)
                        .map_or(FunctionalStatus::FailureWarning, |c| {
                            FunctionalStatus::from_code(*c)
                        }),
                    None => FunctionalStatus::NormalOperation,
                };
                (p.decode_raw(raw), fs)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_sizes() {
        assert_eq!(frame_bytes(0), 64);
        assert_eq!(frame_bytes(18), 64);
        assert_eq!(frame_bytes(100), 146);
        assert_eq!(wire_bytes(100), 166);
        assert_eq!(wire_time_ns(18), 84 * 80);
        assert_eq!(wire_time_ns(1471), (1471 + 46 + 20) as u64 * 80);
        assert_eq!(wire_time_ns(1471) / 1000, 122);
    }

    #[test]
    fn vl_properties() {
        let vl = VirtualLink::new(1, 4_000, 1518, Priority::High);
        assert_eq!(vl.max_payload(), 1472);
        assert_eq!(vl.max_bandwidth_bps(), 1518 * 8 * 250);
    }

    #[test]
    fn end_system_shaping() {
        let vl = VirtualLink::new(1, 2_000, 200, Priority::High);
        let mut es = EndSystem::new();
        let (s1, e1) = es.schedule(&vl, 0, 100, 0);
        es.commit(&vl, s1);
        assert_eq!(s1, 0);
        assert_eq!(e1, 166 * 80);
        let (s2, _) = es.schedule(&vl, 0, 100, 500_000);
        assert_eq!(s2, 2_000_000);
        es.commit(&vl, s2);
        let (s3, _) = es.schedule(&vl, 0, 100, 10_000_000);
        assert_eq!(s3, 10_000_000);
    }

    #[test]
    fn end_system_port_serialises_vls() {
        let a = VirtualLink::new(1, 1_000, 200, Priority::High);
        let b = VirtualLink::new(2, 1_000, 200, Priority::High);
        let mut es = EndSystem::new();
        let (s, e) = es.schedule(&a, 0, 100, 0);
        es.commit(&a, s);
        let (s2, _) = es.schedule(&b, 0, 100, 0);
        assert_eq!(s2, e);
    }

    fn net() -> AfdxNetwork {
        let mut n = AfdxNetwork::new(10_000);
        n.add_virtual_link(VirtualLink::new(1, 1_000, 200, Priority::High), 1, &[10]);
        n.add_virtual_link(VirtualLink::new(2, 1_000, 1518, Priority::Low), 2, &[10]);
        n
    }

    #[test]
    fn single_frame_latency_numbers() {
        let mut n = net();
        assert!(n.send(1, &[0u8; 100], 0));
        let d = n.poll(10_000);
        assert_eq!(d.len(), 1);
        let t = 166 * 80 + 10_000 + 166 * 80;
        assert_eq!(d[0].time_us, (t as u64).div_ceil(1000));
        assert_eq!(d[0].dest, 10);
        assert_eq!(d[0].net, 0);
    }

    #[test]
    fn jitter_from_queueing() {
        let mut n = net();
        n.send(2, &[0u8; 1472], 0);
        n.send(1, &[0u8; 100], 0);
        let d = n.poll(100_000);
        assert_eq!(d.len(), 2);
        let small = d.iter().find(|x| x.vl == 1).unwrap();
        let mut alone = net();
        alone.send(1, &[0u8; 100], 0);
        let base = alone.poll(100_000)[0].time_us;
        assert!(small.time_us >= base);
        let mut n2 = net();
        n2.send(1, &[0u8; 100], 0);
        n2.send(2, &[0u8; 1472], 0);
        let d2 = n2.poll(100_000);
        let big = d2.iter().find(|x| x.vl == 2).unwrap();
        assert!(big.time_us > base);
    }

    #[test]
    fn priority_wins_output_queue() {
        let mut sw = Switch::new(0);
        let lo = Frame {
            vl: 2,
            seq: 1,
            net: 0,
            priority: Priority::Low,
            payload: vec![0; 100],
            order: 1,
        };
        let hi = Frame {
            vl: 1,
            seq: 1,
            net: 0,
            priority: Priority::High,
            payload: vec![0; 100],
            order: 2,
        };
        sw.enqueue(5, 0, lo);
        sw.enqueue(5, 0, hi);
        let out = sw.run(1_000_000);
        assert_eq!(out[0].1.vl, 1);
        assert_eq!(out[1].1.vl, 2);
        assert_eq!(out[1].2, out[0].2 + wire_time_ns(100));
        assert_eq!(sw.queue_len(), 0);
    }

    #[test]
    fn redundancy_first_valid_wins() {
        let mut n = net();
        n.switches[1].fabric_latency_ns = 500_000;
        n.send(1, &[1, 2, 3], 0);
        n.send(1, &[4, 5, 6], 1_000);
        let d = n.poll(100_000);
        assert_eq!(d.len(), 2);
        assert!(d.iter().all(|x| x.net == 0));
        assert_eq!(d[0].seq, 1);
        assert_eq!(d[1].seq, 2);
        assert_eq!(n.redundancy_discards(), 2);
    }

    #[test]
    fn redundancy_survives_net_failure() {
        let mut n = net();
        n.set_network_failed(0, true);
        n.send(1, &[1], 0);
        n.send(1, &[2], 1_000);
        let d = n.poll(100_000);
        assert_eq!(d.len(), 2);
        assert!(d.iter().all(|x| x.net == 1));
        n.set_network_failed(1, true);
        n.send(1, &[3], 2_000);
        assert!(n.poll(200_000).is_empty());
    }

    #[test]
    fn rm_sequence_rules() {
        let mut rm = RedundancyManager::new(10);
        assert!(rm.accept(1, 1, 250));
        assert!(!rm.accept(1, 1, 250));
        assert!(rm.accept(1, 1, 251));
        assert!(!rm.accept(1, 1, 249));
        assert!(rm.accept(1, 1, 2));
        assert!(!rm.accept(1, 1, 100));
        assert!(rm.accept(1, 2, 5));
        assert_eq!(rm.discarded, 3);
    }

    #[test]
    fn oversize_and_unknown_rejected() {
        let mut n = net();
        assert!(!n.send(1, &[0u8; 200], 0));
        assert_eq!(n.rejected_oversize, 1);
        assert!(!n.send(99, &[0u8; 2], 0));
    }

    #[test]
    fn bag_enforced_on_wire() {
        let mut n = net();
        n.send(1, &[0u8; 10], 0);
        n.send(1, &[0u8; 10], 0);
        let d = n.poll(100_000);
        assert_eq!(d.len(), 2);
        assert!(d[1].time_us - d[0].time_us >= 1_000);
    }

    #[test]
    fn bit_io() {
        let mut b = [0u8; 4];
        write_bits(&mut b, 3, 10, 0b10_1101_0111);
        assert_eq!(read_bits(&b, 3, 10), 0b10_1101_0111);
        assert_eq!(b[0], 0b0001_0110);
        write_bits(&mut b, 3, 10, 0);
        assert_eq!(b, [0; 4]);
    }

    #[test]
    fn packing_round_trip() {
        let mut l = MessageLayout::new(16, 12);
        l.add(ParamDef::new("a", 0, 1, ParamEncoding::Bool).with_fs(0));
        l.add(ParamDef::new("b", 3, 13, ParamEncoding::Signed).with_fs(1));
        l.add(ParamDef::new("c", 32, 32, ParamEncoding::Float32).with_fs(2));
        l.add(ParamDef::new(
            "d",
            64,
            16,
            ParamEncoding::Scaled {
                signed: true,
                resolution: 0.25,
            },
        ));
        l.add(ParamDef::new("e", 80, 12, ParamEncoding::Unsigned));
        let vals = [
            (1.0, FunctionalStatus::NormalOperation),
            (-1234.0, FunctionalStatus::NoComputedData),
            (3.5, FunctionalStatus::FailureWarning),
            (-100.25, FunctionalStatus::NormalOperation),
            (4095.0, FunctionalStatus::NormalOperation),
        ];
        let p = l.encode(&vals);
        assert_eq!(p.len(), 16);
        assert_eq!(p[12], 0x03);
        assert_eq!(p[13], 0x0C);
        assert_eq!(p[14], 0x00);
        let d = l.decode(&p);
        assert_eq!(d[0].0, 1.0);
        assert_eq!(d[1], (-1234.0, FunctionalStatus::NoComputedData));
        assert_eq!(d[2].0, 3.5);
        assert_eq!(d[2].1, FunctionalStatus::FailureWarning);
        assert_eq!(d[3].0, -100.25);
        assert_eq!(d[4].0, 4095.0);
        assert_eq!(d[4].1, FunctionalStatus::NormalOperation);
    }

    #[test]
    fn packing_saturates() {
        let mut l = MessageLayout::new(4, 3);
        l.add(ParamDef::new("a", 0, 8, ParamEncoding::Unsigned));
        l.add(ParamDef::new("b", 8, 8, ParamEncoding::Signed));
        let d = l.decode(&l.encode(&[
            (999.0, FunctionalStatus::NormalOperation),
            (-999.0, FunctionalStatus::NormalOperation),
        ]));
        assert_eq!(d[0].0, 255.0);
        assert_eq!(d[1].0, -128.0);
    }

    #[test]
    fn fs_codes() {
        for fs in [
            FunctionalStatus::NormalOperation,
            FunctionalStatus::NoComputedData,
            FunctionalStatus::FunctionalTest,
            FunctionalStatus::FailureWarning,
        ] {
            assert_eq!(FunctionalStatus::from_code(fs.code()), fs);
        }
    }

    #[test]
    fn network_deterministic() {
        let run = || {
            let mut n = net();
            for i in 0..20u64 {
                n.send(1 + (i % 2) as u16, &[i as u8; 50], i * 300);
            }
            n.poll(1_000_000)
        };
        assert_eq!(run(), run());
    }
}
