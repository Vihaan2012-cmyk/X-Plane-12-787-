#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanId {
    Standard(u16),
    Extended(u32),
}

impl CanId {
    pub fn arbitration_key(self) -> u64 {
        match self {
            CanId::Standard(id) => u64::from(id & 0x7FF) << 20,
            CanId::Extended(id) => {
                let base = u64::from((id >> 18) & 0x7FF);
                let ext = u64::from(id & 0x3FFFF);
                (base << 20) | (1 << 19) | ext
            }
        }
    }
    pub fn is_extended(self) -> bool {
        matches!(self, CanId::Extended(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanFrame {
    pub id: CanId,
    pub dlc: u8,
    pub data: [u8; 8],
}

impl CanFrame {
    pub fn new(id: CanId, payload: &[u8]) -> Self {
        let n = payload.len().min(8);
        let mut data = [0u8; 8];
        data[..n].copy_from_slice(&payload[..n]);
        CanFrame {
            id,
            dlc: n as u8,
            data,
        }
    }
    pub fn payload(&self) -> &[u8] {
        &self.data[..usize::from(self.dlc.min(8))]
    }

    fn stuff_region_bits(&self) -> Vec<bool> {
        let mut bits: Vec<bool> = Vec::new();
        let push = |bits: &mut Vec<bool>, v: u32, n: u32| {
            for i in (0..n).rev() {
                bits.push((v >> i) & 1 == 1);
            }
        };
        bits.push(false);
        match self.id {
            CanId::Standard(id) => {
                push(&mut bits, u32::from(id & 0x7FF), 11);
                bits.push(false);
                bits.push(false);
                bits.push(false);
            }
            CanId::Extended(id) => {
                push(&mut bits, (id >> 18) & 0x7FF, 11);
                bits.push(true);
                bits.push(true);
                push(&mut bits, id & 0x3FFFF, 18);
                bits.push(false);
                bits.push(false);
                bits.push(false);
            }
        }
        push(&mut bits, u32::from(self.dlc & 0xF), 4);
        for b in self.payload() {
            push(&mut bits, u32::from(*b), 8);
        }
        let crc = crc15(&bits);
        push(&mut bits, u32::from(crc), 15);
        bits
    }

    pub fn stuffed_bits(&self) -> u32 {
        let bits = self.stuff_region_bits();
        let mut stuffed = 0u32;
        let mut last = bits[0];
        let mut run = 1;
        for &b in &bits[1..] {
            if b == last {
                run += 1;
            } else {
                run = 1;
                last = b;
            }
            if run == 5 {
                stuffed += 1;
                last = !last;
                run = 1;
            }
        }
        stuffed
    }

    pub fn bits_exact(&self) -> u32 {
        self.stuff_region_bits().len() as u32 + self.stuffed_bits() + 10
    }

    pub fn bits_worst_case(&self) -> u32 {
        let n = u32::from(self.dlc.min(8));
        let g = if self.id.is_extended() {
            54 + 8 * n
        } else {
            34 + 8 * n
        };
        g + 13 + (g - 1) / 4
    }

    pub fn time_us_worst_case(&self, bitrate: u32) -> u64 {
        u64::from(self.bits_worst_case()) * 1_000_000 / u64::from(bitrate)
    }
}

pub fn crc15(bits: &[bool]) -> u16 {
    let mut crc: u16 = 0;
    for &b in bits {
        let top = (crc >> 14) & 1 == 1;
        crc = (crc << 1) & 0x7FFF;
        if top != b {
            crc ^= 0x4599;
        }
    }
    crc
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanState {
    ErrorActive,
    ErrorPassive,
    BusOff,
}

#[derive(Clone, Debug)]
pub struct CanNode {
    pub tec: u32,
    pub rec: u32,
    recovery_bits: u32,
}

impl Default for CanNode {
    fn default() -> Self {
        Self::new()
    }
}

impl CanNode {
    pub fn new() -> Self {
        CanNode {
            tec: 0,
            rec: 0,
            recovery_bits: 0,
        }
    }
    pub fn state(&self) -> CanState {
        if self.tec > 255 {
            CanState::BusOff
        } else if self.tec > 127 || self.rec > 127 {
            CanState::ErrorPassive
        } else {
            CanState::ErrorActive
        }
    }
    pub fn tx_error(&mut self) {
        if self.state() != CanState::BusOff {
            self.tec += 8;
        }
    }
    pub fn tx_success(&mut self) {
        self.tec = self.tec.saturating_sub(1);
    }
    pub fn rx_error(&mut self) {
        self.rec = (self.rec + 1).min(255);
    }
    pub fn rx_success(&mut self) {
        if self.rec > 127 {
            self.rec = 120;
        } else {
            self.rec = self.rec.saturating_sub(1);
        }
    }
    pub fn recessive_bits(&mut self, n: u32) {
        if self.state() == CanState::BusOff {
            self.recovery_bits += n;
            if self.recovery_bits >= 128 * 11 {
                self.tec = 0;
                self.rec = 0;
                self.recovery_bits = 0;
            }
        }
    }
    pub fn may_transmit(&self) -> bool {
        self.state() != CanState::BusOff
    }
}

#[derive(Clone, Debug)]
struct Pending {
    ready_us: u64,
    seq: u64,
    node: u32,
    frame: CanFrame,
}

#[derive(Clone, Debug)]
pub struct CanBus {
    bitrate: u32,
    pending: Vec<Pending>,
    free_us: u64,
    seq: u64,
}

impl CanBus {
    pub fn new(bitrate: u32) -> Self {
        CanBus {
            bitrate,
            pending: Vec::new(),
            free_us: 0,
            seq: 0,
        }
    }
    pub fn submit(&mut self, ready_us: u64, node: u32, frame: CanFrame) {
        self.seq += 1;
        self.pending.push(Pending {
            ready_us,
            seq: self.seq,
            node,
            frame,
        });
    }
    pub fn process(&mut self, until_us: u64) -> Vec<(u64, u64, u32, CanFrame)> {
        let mut out = Vec::new();
        loop {
            if self.pending.is_empty() {
                break;
            }
            let earliest = self.pending.iter().map(|p| p.ready_us).min().unwrap_or(0);
            let start = self.free_us.max(earliest);
            if start > until_us {
                break;
            }
            let mut best: Option<usize> = None;
            for (i, p) in self.pending.iter().enumerate() {
                if p.ready_us > start {
                    continue;
                }
                let better = match best {
                    None => true,
                    Some(b) => {
                        let q = &self.pending[b];
                        (p.frame.id.arbitration_key(), p.seq) < (q.frame.id.arbitration_key(), q.seq)
                    }
                };
                if better {
                    best = Some(i);
                }
            }
            let Some(i) = best else { break };
            let p = self.pending.remove(i);
            let bits = u64::from(p.frame.bits_exact()) + 3;
            let end = start + (bits * 1_000_000).div_ceil(u64::from(self.bitrate));
            self.free_us = end;
            out.push((start, end, p.node, p.frame));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worst_case_bits_known() {
        let f = CanFrame::new(CanId::Standard(0x123), &[0; 8]);
        assert_eq!(f.bits_worst_case(), 135);
        let e = CanFrame::new(CanId::Extended(0x1234567), &[0; 8]);
        assert_eq!(e.bits_worst_case(), 160);
        let z = CanFrame::new(CanId::Standard(1), &[]);
        assert_eq!(z.bits_worst_case(), 34 + 13 + 33 / 4);
        assert_eq!(f.time_us_worst_case(500_000), 270);
        assert_eq!(f.time_us_worst_case(1_000_000), 135);
    }

    #[test]
    fn exact_bits_within_bounds() {
        for id in [0u16, 0x7FF, 0x123, 0x555, 0x0F0] {
            for n in 0..=8usize {
                let f = CanFrame::new(CanId::Standard(id), &[0u8; 8][..n]);
                let exact = f.bits_exact();
                assert!(exact <= f.bits_worst_case(), "{id} {n}");
                assert!(exact >= 44 + 8 * n as u32);
                let g = CanFrame::new(CanId::Standard(id), &[0xFF; 8][..n]);
                assert!(g.bits_exact() <= g.bits_worst_case());
            }
        }
        let e = CanFrame::new(CanId::Extended(0), &[0; 8]);
        assert!(e.bits_exact() <= e.bits_worst_case());
        assert!(e.stuffed_bits() > 10);
    }

    #[test]
    fn crc15_known() {
        assert_eq!(crc15(&[]), 0);
        let bits = [true, false, true, true, false, false, true];
        let c = crc15(&bits);
        let mut all = bits.to_vec();
        for i in (0..15).rev() {
            all.push((c >> i) & 1 == 1);
        }
        assert_eq!(crc15(&all), 0);
    }

    #[test]
    fn arbitration_order() {
        assert!(CanId::Standard(0x100).arbitration_key() < CanId::Standard(0x101).arbitration_key());
        assert!(CanId::Standard(0x100).arbitration_key() < CanId::Extended(0x100 << 18).arbitration_key());
        assert!(CanId::Extended(0x100 << 18).arbitration_key() < CanId::Standard(0x101).arbitration_key());
        assert!(
            CanId::Extended((0x100 << 18) | 5).arbitration_key()
                < CanId::Extended((0x100 << 18) | 6).arbitration_key()
        );
    }

    #[test]
    fn bus_arbitrates_and_serialises() {
        let mut bus = CanBus::new(500_000);
        bus.submit(0, 1, CanFrame::new(CanId::Standard(0x300), &[1]));
        bus.submit(0, 2, CanFrame::new(CanId::Standard(0x100), &[2]));
        bus.submit(0, 3, CanFrame::new(CanId::Standard(0x200), &[3]));
        let out = bus.process(1_000_000);
        let order: Vec<u32> = out.iter().map(|o| o.2).collect();
        assert_eq!(order, vec![2, 3, 1]);
        assert_eq!(out[1].0, out[0].1);
        assert_eq!(out[2].0, out[1].1);
    }

    #[test]
    fn bus_later_frame_waits_for_busy() {
        let mut bus = CanBus::new(500_000);
        bus.submit(0, 1, CanFrame::new(CanId::Standard(0x300), &[1; 8]));
        bus.submit(10, 2, CanFrame::new(CanId::Standard(0x001), &[2]));
        let out = bus.process(10_000);
        assert_eq!(out[0].2, 1);
        assert_eq!(out[1].2, 2);
        assert!(out[1].0 >= out[0].1);
        assert!(bus.process(20_000).is_empty());
    }

    #[test]
    fn error_counters_and_states() {
        let mut n = CanNode::new();
        assert_eq!(n.state(), CanState::ErrorActive);
        for _ in 0..16 {
            n.tx_error();
        }
        assert_eq!(n.tec, 128);
        assert_eq!(n.state(), CanState::ErrorPassive);
        n.tx_success();
        assert_eq!(n.tec, 127);
        assert_eq!(n.state(), CanState::ErrorActive);
        for _ in 0..17 {
            n.tx_error();
        }
        assert_eq!(n.state(), CanState::BusOff);
        assert!(!n.may_transmit());
        n.recessive_bits(1407);
        assert_eq!(n.state(), CanState::BusOff);
        n.recessive_bits(1);
        assert_eq!(n.state(), CanState::ErrorActive);
        assert_eq!(n.tec, 0);
    }

    #[test]
    fn receive_errors_go_passive() {
        let mut n = CanNode::new();
        for _ in 0..128 {
            n.rx_error();
        }
        assert_eq!(n.state(), CanState::ErrorPassive);
        n.rx_success();
        assert_eq!(n.rec, 120);
        assert_eq!(n.state(), CanState::ErrorActive);
    }
}
