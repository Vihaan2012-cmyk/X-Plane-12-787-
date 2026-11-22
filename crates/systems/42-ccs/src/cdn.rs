use crate::icd::crc32;
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

pub const FRAME_OVERHEAD_BYTES: usize = 67;
pub const SEQ_BYTES: usize = 1;
pub const APPROX_SWITCH_LATENCY_US: u64 = 20;
pub const APPROX_FOX_LATENCY_US: u64 = 1;
pub const MAX_TX_JITTER_US: u64 = 500;
pub const APPROX_QUEUE_FRAMES: usize = 64;
pub const APPROX_MAX_SKEW_US: u64 = 2_000;
pub const RATE_100M: f64 = 100.0e6;
pub const RATE_10M: f64 = 10.0e6;
pub const CHANNELS: usize = 2;
pub const SETTLE_US: u64 = 500_000;
pub const CORRUPT_BYTE: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevKind {
    EndSystem,
    Switch,
    Fox,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub name: String,
    pub kind: DevKind,
    pub powered: bool,
    pub failed: bool,
    pub replay: bool,
    pub babbling: bool,
    pub corrupt_before_crc: bool,
    pub ports: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub a: usize,
    pub b: usize,
    pub rate_bps: f64,
    pub fibre: bool,
    pub channel: usize,
    pub failed: bool,
    pub ber: f64,
    pub crc_errors: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vl {
    pub id: u16,
    pub name: String,
    pub src: usize,
    pub dests: Vec<usize>,
    pub bag_us: u64,
    pub max_bytes: usize,
    pub high_priority: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub vl: u16,
    pub seq: u8,
    pub channel: usize,
    pub payload: Vec<u8>,
    pub crc: u32,
    pub sent_us: u64,
}

impl Frame {
    pub fn wire_bits(&self) -> f64 {
        ((self.payload.len() + SEQ_BYTES + FRAME_OVERHEAD_BYTES) * 8) as f64
    }

    pub fn crc_ok(&self) -> bool {
        crc32(&self.payload) == self.crc
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PortQueue {
    hi: VecDeque<Frame>,
    lo: VecDeque<Frame>,
    busy: bool,
    pub drops: u64,
    pub sent: u64,
    pub max_depth: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Police {
    credit_us: i64,
    last_us: u64,
    pub drops: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RxState {
    pub last_seq: Option<u8>,
    pub data: Vec<u8>,
    pub rx_us: Option<u64>,
    pub accepted: [u64; CHANNELS],
    pub duplicates: u64,
    pub seq_errors: u64,
    pub max_skew_us: u64,
    pub max_latency_us: u64,
    last_seq_time: Option<(u8, u64)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TxState {
    next_us: u64,
    seq: u8,
    data: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq)]
enum EvKind {
    Arrive { dev: usize, link: usize, frame: Frame },
    TxDone { link: usize, dir: usize },
    Send { vl: u16 },
}

#[derive(Clone, Debug, PartialEq)]
struct Ev {
    t: u64,
    seq: u64,
    kind: EvKind,
}

impl Eq for Ev {}
impl PartialOrd for Ev {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Ev {
    fn cmp(&self, o: &Self) -> Ordering {
        (self.t, self.seq).cmp(&(o.t, o.seq))
    }
}

#[derive(Clone, Debug)]
pub struct Cdn {
    pub devices: Vec<Device>,
    pub links: Vec<Link>,
    pub vls: Vec<Vl>,
    fwd: BTreeMap<(usize, u16), Vec<usize>>,
    queues: BTreeMap<(usize, usize), PortQueue>,
    police: BTreeMap<(usize, usize, u16), Police>,
    rx: BTreeMap<(usize, u16), RxState>,
    tx: BTreeMap<u16, TxState>,
    events: BinaryHeap<Reverse<Ev>>,
    ev_seq: u64,
    rng: u64,
    pub now_us: u64,
    pub unknown_vl_drops: u64,
    pub delivered: u64,
    frozen: BTreeMap<(usize, u16), Frame>,
    paths: BTreeMap<(u16, usize, usize), Vec<(usize, usize)>>,
    last_key: Vec<u8>,
    settle_until: u64,
    pub full_steps: u64,
    pub fast_steps: u64,
}

fn next_rand(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

impl Cdn {
    pub fn new(seed: u64) -> Cdn {
        Cdn {
            devices: Vec::new(),
            links: Vec::new(),
            vls: Vec::new(),
            fwd: BTreeMap::new(),
            queues: BTreeMap::new(),
            police: BTreeMap::new(),
            rx: BTreeMap::new(),
            tx: BTreeMap::new(),
            events: BinaryHeap::new(),
            ev_seq: 0,
            rng: seed | 1,
            now_us: 0,
            unknown_vl_drops: 0,
            delivered: 0,
            frozen: BTreeMap::new(),
            paths: BTreeMap::new(),
            last_key: Vec::new(),
            settle_until: 0,
            full_steps: 0,
            fast_steps: 0,
        }
    }

    pub fn add_device(&mut self, name: &str, kind: DevKind) -> usize {
        self.devices.push(Device {
            name: name.to_string(),
            kind,
            powered: false,
            failed: false,
            replay: false,
            babbling: false,
            corrupt_before_crc: false,
            ports: Vec::new(),
        });
        self.devices.len() - 1
    }

    pub fn device(&self, name: &str) -> Option<usize> {
        self.devices.iter().position(|d| d.name == name)
    }

    pub fn connect(&mut self, a: usize, b: usize, rate_bps: f64, fibre: bool, channel: usize) -> usize {
        self.links.push(Link {
            a,
            b,
            rate_bps,
            fibre,
            channel,
            failed: false,
            ber: 0.0,
            crc_errors: 0,
        });
        let l = self.links.len() - 1;
        self.devices[a].ports.push(l);
        self.devices[b].ports.push(l);
        l
    }

    fn other(&self, link: usize, dev: usize) -> (usize, usize) {
        let l = &self.links[link];
        if l.a == dev { (l.b, 0) } else { (l.a, 1) }
    }

    pub fn add_vl(
        &mut self,
        name: &str,
        src: usize,
        dests: Vec<usize>,
        bag_us: u64,
        max_bytes: usize,
        hi: bool,
    ) -> u16 {
        let id = self.vls.len() as u16 + 1;
        let phase = next_rand(&mut self.rng) % bag_us.max(1);
        self.vls.push(Vl {
            id,
            name: name.to_string(),
            src,
            dests,
            bag_us,
            max_bytes,
            high_priority: hi,
        });
        self.tx.insert(
            id,
            TxState {
                next_us: phase,
                seq: 0,
                data: None,
            },
        );
        id
    }

    pub fn vl_by_name(&self, name: &str) -> Option<u16> {
        self.vls.iter().find(|v| v.name == name).map(|v| v.id)
    }

    pub fn build_routes(&mut self) {
        self.fwd.clear();
        self.paths.clear();
        for vi in 0..self.vls.len() {
            let vl = self.vls[vi].clone();
            for ch in 0..CHANNELS {
                let mut prev: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
                let mut seen = vec![false; self.devices.len()];
                let mut q = VecDeque::new();
                seen[vl.src] = true;
                q.push_back(vl.src);
                while let Some(d) = q.pop_front() {
                    if d != vl.src && self.devices[d].kind == DevKind::EndSystem {
                        continue;
                    }
                    for &l in &self.devices[d].ports {
                        if self.links[l].channel != ch {
                            continue;
                        }
                        let (n, _) = self.other(l, d);
                        if !seen[n] {
                            seen[n] = true;
                            prev.insert(n, (d, l));
                            q.push_back(n);
                        }
                    }
                }
                for &dst in &vl.dests {
                    let mut cur = dst;
                    let mut hops = Vec::new();
                    while let Some(&(p, l)) = prev.get(&cur) {
                        let e = self.fwd.entry((p, vl.id)).or_default();
                        if !e.contains(&l) {
                            e.push(l);
                        }
                        hops.push((p, l));
                        cur = p;
                    }
                    if cur == vl.src && !hops.is_empty() {
                        self.paths.insert((vl.id, dst, ch), hops);
                    }
                }
            }
        }
    }

    pub fn publish(&mut self, vl: u16, payload: Vec<u8>) {
        if let Some(t) = self.tx.get_mut(&vl) {
            t.data = Some(payload);
        }
    }

    pub fn unpublish(&mut self, vl: u16) {
        if let Some(t) = self.tx.get_mut(&vl) {
            t.data = None;
        }
    }

    pub fn rx(&self, dev: usize, vl: u16) -> Option<&RxState> {
        self.rx.get(&(dev, vl))
    }

    pub fn queue(&self, dev: usize, link: usize) -> Option<&PortQueue> {
        self.queues.get(&(dev, link))
    }

    pub fn police_drops(&self) -> u64 {
        self.police.values().map(|p| p.drops).sum()
    }

    pub fn queue_drops(&self) -> u64 {
        self.queues.values().map(|q| q.drops).sum()
    }

    fn push(&mut self, t: u64, kind: EvKind) {
        self.ev_seq += 1;
        self.events.push(Reverse(Ev {
            t,
            seq: self.ev_seq,
            kind,
        }));
    }

    fn up(&self, dev: usize) -> bool {
        let d = &self.devices[dev];
        d.powered && !d.failed
    }

    fn enqueue(&mut self, t: u64, dev: usize, link: usize, frame: Frame, hi: bool) {
        if self.links[link].failed || !self.up(dev) {
            return;
        }
        let latency = match self.devices[dev].kind {
            DevKind::Switch => APPROX_SWITCH_LATENCY_US,
            DevKind::Fox => APPROX_FOX_LATENCY_US,
            DevKind::EndSystem => 0,
        };
        let q = self.queues.entry((dev, link)).or_default();
        let depth = q.hi.len() + q.lo.len();
        if depth >= APPROX_QUEUE_FRAMES {
            q.drops += 1;
            if !(hi && q.lo.pop_back().is_some()) {
                return;
            }
        }
        if hi {
            q.hi.push_back(frame);
        } else {
            q.lo.push_back(frame);
        }
        q.max_depth = q.max_depth.max(q.hi.len() + q.lo.len());
        if !q.busy {
            q.busy = true;
            let dir = if self.links[link].a == dev { 0 } else { 1 };
            self.start_tx(t + latency, dev, link, dir);
        }
    }

    fn start_tx(&mut self, t: u64, dev: usize, link: usize, dir: usize) {
        let frame = {
            let q = self.queues.entry((dev, link)).or_default();
            match q.hi.pop_front().or_else(|| q.lo.pop_front()) {
                Some(f) => f,
                None => {
                    q.busy = false;
                    return;
                }
            }
        };
        let rate = self.links[link].rate_bps;
        let ser = (frame.wire_bits() / rate * 1.0e6).ceil() as u64;
        let done = t + ser.max(1);
        let mut frame = frame;
        let ber = self.links[link].ber;
        if ber > 0.0 {
            let p = 1.0 - (1.0 - ber).powf(frame.wire_bits());
            let r = (next_rand(&mut self.rng) >> 11) as f64 / (1u64 << 53) as f64;
            if r < p && !frame.payload.is_empty() {
                let bit = (next_rand(&mut self.rng) as usize) % (frame.payload.len() * 8);
                frame.payload[bit / 8] ^= 1 << (bit % 8);
            }
        }
        let (to, _) = self.other(link, dev);
        self.push(done, EvKind::Arrive { dev: to, link, frame });
        self.push(done, EvKind::TxDone { link, dir });
        if let Some(q) = self.queues.get_mut(&(dev, link)) {
            q.sent += 1;
        }
    }

    fn police_ok(&mut self, t: u64, dev: usize, link: usize, vl: &Vl, bytes: usize) -> bool {
        if bytes > vl.max_bytes {
            self.police.entry((dev, link, vl.id)).or_default().drops += 1;
            return false;
        }
        let cap = (vl.bag_us + MAX_TX_JITTER_US) as i64;
        let p = self.police.entry((dev, link, vl.id)).or_default();
        if p.last_us == 0 && p.credit_us == 0 {
            p.credit_us = cap;
        }
        p.credit_us = (p.credit_us + (t.saturating_sub(p.last_us)) as i64).min(cap);
        p.last_us = t;
        if p.credit_us >= vl.bag_us as i64 - MAX_TX_JITTER_US as i64 {
            p.credit_us -= vl.bag_us as i64;
            true
        } else {
            p.drops += 1;
            false
        }
    }

    fn arrive(&mut self, t: u64, dev: usize, link: usize, frame: Frame) {
        if self.links[link].failed || !self.up(dev) {
            return;
        }
        if !frame.crc_ok() {
            self.links[link].crc_errors += 1;
            return;
        }
        let Some(vl) = self.vls.get(frame.vl as usize - 1).cloned() else {
            self.unknown_vl_drops += 1;
            return;
        };
        match self.devices[dev].kind {
            DevKind::EndSystem => {
                if !vl.dests.contains(&dev) {
                    return;
                }
                self.receive(t, dev, &vl, frame);
            }
            DevKind::Switch | DevKind::Fox => {
                if self.devices[dev].kind == DevKind::Switch
                    && !self.police_ok(t, dev, link, &vl, frame.payload.len())
                {
                    return;
                }
                let Some(outs) = self.fwd.get(&(dev, vl.id)).cloned() else {
                    self.unknown_vl_drops += 1;
                    return;
                };
                for out in outs {
                    if out != link {
                        self.enqueue(t, dev, out, frame.clone(), vl.high_priority);
                    }
                }
            }
        }
    }

    fn receive(&mut self, t: u64, dev: usize, vl: &Vl, frame: Frame) {
        let r = self.rx.entry((dev, vl.id)).or_default();
        if let Some((s, at)) = r.last_seq_time
            && s == frame.seq
        {
            r.duplicates += 1;
            r.max_skew_us = r.max_skew_us.max(t.saturating_sub(at));
            return;
        }
        if let Some(last) = r.last_seq {
            let d = frame.seq.wrapping_sub(last);
            if d == 0 {
                r.seq_errors += 1;
                return;
            }
            if d > 2 {
                r.seq_errors += 1;
                let gap = r.rx_us.map(|x| t.saturating_sub(x)).unwrap_or(u64::MAX);
                if gap < APPROX_MAX_SKEW_US {
                    return;
                }
            }
        }
        r.last_seq = Some(frame.seq);
        r.last_seq_time = Some((frame.seq, t));
        r.data = frame.payload;
        r.rx_us = Some(t);
        r.accepted[frame.channel] += 1;
        r.max_latency_us = r.max_latency_us.max(t.saturating_sub(frame.sent_us));
        self.delivered += 1;
    }

    fn transmit_due(&mut self, until: u64) {
        let ids: Vec<u16> = self.tx.keys().copied().collect();
        for id in ids {
            loop {
                let bag = self.vls[id as usize - 1].bag_us;
                let src = self.vls[id as usize - 1].src;
                let slot = self.tx.get(&id).map(|s| s.next_us).unwrap_or(u64::MAX);
                if slot > until {
                    break;
                }
                let babble = self.devices[src].babbling;
                let step = if babble { (bag / 8).max(1) } else { bag };
                let jitter = if babble {
                    0
                } else {
                    next_rand(&mut self.rng) % MAX_TX_JITTER_US
                };
                if let Some(s) = self.tx.get_mut(&id) {
                    s.next_us = slot + step;
                }
                self.push(slot + jitter, EvKind::Send { vl: id });
            }
        }
    }

    fn send(&mut self, t: u64, id: u16) {
        let vl = self.vls[id as usize - 1].clone();
        let src = vl.src;
        if !self.up(src) {
            return;
        }
        let Some(data) = self.tx.get(&id).and_then(|s| s.data.clone()) else {
            return;
        };
        let seq = self
            .tx
            .get(&id)
            .map(|s| s.seq)
            .unwrap_or(0)
            .wrapping_add(1)
            .max(1);
        let mut payload = data;
        let crc = crc32(&payload);
        let crc = if self.devices[src].corrupt_before_crc && !payload.is_empty() {
            let k = CORRUPT_BYTE.min(payload.len() - 1);
            payload[k] ^= 0x01;
            crc32(&payload)
        } else {
            crc
        };
        if let Some(s) = self.tx.get_mut(&id) {
            s.seq = seq;
        }
        for ch in 0..CHANNELS {
            let frame = Frame {
                vl: id,
                seq,
                channel: ch,
                payload: payload.clone(),
                crc,
                sent_us: t,
            };
            let ports: Vec<usize> = self.devices[src]
                .ports
                .iter()
                .copied()
                .filter(|l| self.links[*l].channel == ch)
                .collect();
            for l in ports {
                self.enqueue(t, src, l, frame.clone(), vl.high_priority);
            }
        }
    }

    fn key(&self) -> (Vec<u8>, bool) {
        let mut k = Vec::with_capacity(self.devices.len() + self.links.len());
        let mut faulty = false;
        for d in &self.devices {
            k.push(u8::from(d.powered) | (u8::from(d.failed) << 1));
            faulty |= d.replay || d.babbling || d.corrupt_before_crc;
        }
        for l in &self.links {
            k.push(u8::from(l.failed));
            faulty |= l.ber > 0.0;
        }
        (k, faulty)
    }

    fn path_up(&self, vl: u16, dest: usize, ch: usize) -> bool {
        self.paths.get(&(vl, dest, ch)).is_some_and(|hops| {
            hops.iter().all(|&(d, l)| self.up(d) && !self.links[l].failed) && self.up(dest)
        })
    }

    pub fn is_fast(&self) -> bool {
        self.events.is_empty() && self.fast_steps > 0
    }

    fn fast_forward(&mut self, until: u64) {
        for vi in 0..self.vls.len() {
            let id = self.vls[vi].id;
            let bag = self.vls[vi].bag_us;
            let src = self.vls[vi].src;
            let (next, data, seq) = match self.tx.get(&id) {
                Some(t) => (t.next_us, t.data.clone(), t.seq),
                None => continue,
            };
            if next > until {
                continue;
            }
            let n = (until - next) / bag + 1;
            let last_slot = next + (n - 1) * bag;
            let mut new_seq = seq;
            for _ in 0..n.min(512) {
                new_seq = new_seq.wrapping_add(1).max(1);
            }
            if let Some(t) = self.tx.get_mut(&id) {
                t.next_us = next + n * bag;
                if self.devices[src].powered && !self.devices[src].failed && data.is_some() {
                    t.seq = new_seq;
                }
            }
            let Some(data) = data else {
                continue;
            };
            if !self.up(src) {
                continue;
            }
            let dests = self.vls[vi].dests.clone();
            for dest in dests {
                let chans: Vec<usize> = (0..CHANNELS).filter(|ch| self.path_up(id, dest, *ch)).collect();
                if chans.is_empty() {
                    continue;
                }
                let r = self.rx.entry((dest, id)).or_default();
                r.data = data.clone();
                r.rx_us = Some(last_slot);
                r.last_seq = Some(new_seq);
                r.last_seq_time = Some((new_seq, last_slot));
                r.accepted[chans[0]] += n;
                if chans.len() > 1 {
                    r.duplicates += n;
                }
                self.delivered += n;
            }
        }
    }

    fn drain_events(&mut self) {
        while let Some(Reverse(ev)) = self.events.pop() {
            self.now_us = self.now_us.max(ev.t);
            self.handle(ev);
        }
    }

    fn handle(&mut self, ev: Ev) {
        match ev.kind {
            EvKind::Arrive { dev, link, frame } => {
                let frame = if self.devices[dev].replay && self.devices[dev].kind == DevKind::Switch {
                    self.frozen.entry((dev, frame.vl)).or_insert(frame).clone()
                } else {
                    self.frozen.remove(&(dev, frame.vl));
                    frame
                };
                self.arrive(ev.t, dev, link, frame)
            }
            EvKind::Send { vl } => self.send(ev.t, vl),
            EvKind::TxDone { link, dir } => {
                let dev = if dir == 0 {
                    self.links[link].a
                } else {
                    self.links[link].b
                };
                self.start_tx(ev.t, dev, link, dir);
            }
        }
    }

    pub fn run_until(&mut self, until: u64) {
        let (key, faulty) = self.key();
        if key != self.last_key {
            self.last_key = key;
            self.settle_until = until + SETTLE_US;
        }
        if !faulty && until >= self.settle_until {
            self.drain_events();
            self.fast_forward(until);
            self.fast_steps += 1;
            self.now_us = until;
            return;
        }
        self.full_steps += 1;
        self.transmit_due(until);
        while let Some(Reverse(ev)) = self.events.peek().cloned() {
            if ev.t > until {
                break;
            }
            self.events.pop();
            self.now_us = ev.t;
            self.handle(ev);
        }
        self.now_us = until;
    }
}
