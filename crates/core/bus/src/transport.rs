use crate::a429::Speed;
use crate::can::CanFrame;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub link: u32,
    pub seq: u64,
    pub sent_us: u64,
    pub deliver_us: u64,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct SerialLink {
    bit_time_ns: u64,
    free_us: u64,
}

impl SerialLink {
    pub fn a429(speed: Speed) -> Self {
        SerialLink {
            bit_time_ns: speed.bit_time_us() * 1000,
            free_us: 0,
        }
    }
    pub fn can(bitrate: u32) -> Self {
        SerialLink {
            bit_time_ns: 1_000_000_000 / u64::from(bitrate),
            free_us: 0,
        }
    }
    pub fn schedule(&mut self, sent_us: u64, bits: u64, gap_bits: u64) -> (u64, u64) {
        let start = sent_us.max(self.free_us);
        let end = start + (bits * self.bit_time_ns).div_ceil(1000);
        self.free_us = end + (gap_bits * self.bit_time_ns).div_ceil(1000);
        (start, end)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Transport {
    queue: BTreeMap<(u64, u32, u64), Message>,
    serial: BTreeMap<u32, SerialLink>,
    seq: u64,
}

impl Transport {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register_serial(&mut self, link: u32, l: SerialLink) {
        self.serial.insert(link, l);
    }
    pub fn post(&mut self, link: u32, sent_us: u64, deliver_us: u64, payload: Vec<u8>) -> u64 {
        self.seq += 1;
        let seq = self.seq;
        self.queue.insert(
            (deliver_us.max(sent_us), link, seq),
            Message {
                link,
                seq,
                sent_us,
                deliver_us: deliver_us.max(sent_us),
                payload,
            },
        );
        seq
    }
    pub fn post_a429(&mut self, link: u32, sent_us: u64, word: u32) -> Option<u64> {
        let l = self.serial.get_mut(&link)?;
        let (_, end) = l.schedule(sent_us, 32, 4);
        Some(self.post(link, sent_us, end, word.to_be_bytes().to_vec()))
    }
    pub fn post_can(&mut self, link: u32, sent_us: u64, frame: &CanFrame) -> Option<u64> {
        let l = self.serial.get_mut(&link)?;
        let (_, end) = l.schedule(sent_us, u64::from(frame.bits_exact()), 3);
        Some(self.post(link, sent_us, end, frame.payload().to_vec()))
    }
    pub fn pending(&self) -> usize {
        self.queue.len()
    }
    pub fn next_delivery_us(&self) -> Option<u64> {
        self.queue.keys().next().map(|k| k.0)
    }
    pub fn deliver_due(&mut self, now_us: u64) -> Vec<Message> {
        let keys: Vec<(u64, u32, u64)> = self
            .queue
            .range(..(now_us.saturating_add(1), 0, 0))
            .map(|(k, _)| *k)
            .collect();
        keys.into_iter().filter_map(|k| self.queue.remove(&k)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::can::CanId;

    #[test]
    fn ordered_by_time_link_seq() {
        let mut t = Transport::new();
        t.post(5, 0, 100, vec![1]);
        t.post(2, 0, 100, vec![2]);
        t.post(2, 0, 100, vec![3]);
        t.post(9, 0, 50, vec![4]);
        let d = t.deliver_due(100);
        let p: Vec<u8> = d.iter().map(|m| m.payload[0]).collect();
        assert_eq!(p, vec![4, 2, 3, 1]);
        assert_eq!(t.pending(), 0);
    }

    #[test]
    fn not_delivered_early() {
        let mut t = Transport::new();
        t.post(1, 0, 100, vec![1]);
        assert!(t.deliver_due(99).is_empty());
        assert_eq!(t.next_delivery_us(), Some(100));
        assert_eq!(t.deliver_due(100).len(), 1);
    }

    #[test]
    fn a429_link_serialises() {
        let mut t = Transport::new();
        t.register_serial(1, SerialLink::a429(Speed::Low));
        t.post_a429(1, 0, 0xDEAD_BEEF).unwrap();
        t.post_a429(1, 0, 1).unwrap();
        let d = t.deliver_due(1_000_000);
        assert_eq!(d[0].deliver_us, 2560);
        assert_eq!(d[1].deliver_us, 2880 + 2560);
        assert_eq!(d[0].payload, 0xDEAD_BEEFu32.to_be_bytes().to_vec());
        assert!(t.post_a429(77, 0, 1).is_none());
    }

    #[test]
    fn can_link_timing() {
        let mut t = Transport::new();
        t.register_serial(3, SerialLink::can(1_000_000));
        let f = CanFrame::new(CanId::Standard(0x10), &[1, 2, 3]);
        t.post_can(3, 100, &f).unwrap();
        let d = t.deliver_due(10_000);
        assert_eq!(d[0].deliver_us, 100 + u64::from(f.bits_exact()));
        assert_eq!(d[0].payload, vec![1, 2, 3]);
    }

    #[test]
    fn deterministic() {
        let run = || {
            let mut t = Transport::new();
            for i in 0..50u64 {
                t.post((i % 4) as u32, i, 10 + (i * 7) % 13, vec![i as u8]);
            }
            t.deliver_due(1000)
        };
        assert_eq!(run(), run());
    }
}
