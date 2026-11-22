use super::channel::{Channel, ChannelAddr, ChannelCommand, ChannelLoad, ChannelStatus};
use crate::protection::sspc::SspcConfig;

pub const SPDU_SLOTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpduCardKind {
    Ac1,
    Ac2,
    Dc2,
    Dc4,
    Dc19,
}

impl SpduCardKind {
    pub fn ratings_a(self) -> Vec<f64> {
        match self {
            SpduCardKind::Ac1 => vec![45.0],
            SpduCardKind::Ac2 => vec![22.5, 22.5],
            SpduCardKind::Dc2 => vec![50.0, 50.0],
            SpduCardKind::Dc4 => vec![35.0, 35.0, 35.0, 15.0],
            SpduCardKind::Dc19 => {
                let mut v = vec![10.0; 2];
                v.extend([5.0; 5]);
                v.extend([2.5; 12]);
                v
            }
        }
    }

    pub fn is_ac(self) -> bool {
        matches!(self, SpduCardKind::Ac1 | SpduCardKind::Ac2)
    }
}

#[derive(Clone, Debug)]
pub struct SpduCard {
    pub kind: SpduCardKind,
    pub channels: Vec<Channel>,
}

impl SpduCard {
    pub fn new(kind: SpduCardKind) -> Self {
        let channels = kind
            .ratings_a()
            .iter()
            .map(|r| {
                if kind.is_ac() {
                    Channel::new(SspcConfig::ac(*r, 1))
                } else {
                    Channel::new(SspcConfig::dc(*r))
                }
            })
            .collect();
        Self { kind, channels }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpduInput {
    pub dt_us: u64,
    pub ac_v: f64,
    pub dc_v: f64,
    pub commands: Vec<ChannelCommand>,
    pub loads: Vec<ChannelLoad>,
}

#[derive(Clone, Debug)]
pub struct Spdu {
    pub slots: Vec<Option<SpduCard>>,
}

impl Default for Spdu {
    fn default() -> Self {
        Self::new()
    }
}

impl Spdu {
    pub fn new() -> Self {
        Self {
            slots: vec![None; SPDU_SLOTS],
        }
    }

    pub fn install(&mut self, slot: usize, kind: SpduCardKind) -> bool {
        match self.slots.get_mut(slot) {
            Some(s) if s.is_none() => {
                *s = Some(SpduCard::new(kind));
                true
            }
            _ => false,
        }
    }

    pub fn channel_mut(&mut self, addr: ChannelAddr) -> Option<&mut Channel> {
        self.slots
            .get_mut(addr.slot as usize)?
            .as_mut()?
            .channels
            .get_mut(addr.channel as usize)
    }

    pub fn step(&mut self, input: &SpduInput) -> Vec<ChannelStatus> {
        for cmd in &input.commands {
            if let Some(c) = self.channel_mut(cmd.addr) {
                c.apply(cmd);
            }
        }
        let mut out = Vec::new();
        for (si, slot) in self.slots.iter_mut().enumerate() {
            let Some(card) = slot else { continue };
            let v = if card.kind.is_ac() { input.ac_v } else { input.dc_v };
            for (ci, ch) in card.channels.iter_mut().enumerate() {
                let addr = ChannelAddr {
                    slot: si as u8,
                    channel: ci as u8,
                };
                let load = input.loads.iter().find(|l| l.addr == addr);
                out.push(ch.step(addr, input.dt_us, v, load, false));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protection::sspc::{CommandSource, SspcState};

    #[test]
    fn card_contents_match_documentation() {
        assert_eq!(SpduCardKind::Ac1.ratings_a(), vec![45.0]);
        assert_eq!(SpduCardKind::Ac2.ratings_a(), vec![22.5, 22.5]);
        assert_eq!(SpduCardKind::Dc2.ratings_a(), vec![50.0, 50.0]);
        assert_eq!(SpduCardKind::Dc4.ratings_a(), vec![35.0, 35.0, 35.0, 15.0]);
        let d = SpduCardKind::Dc19.ratings_a();
        assert_eq!(d.len(), 19);
        assert_eq!(d.iter().filter(|r| **r == 2.5).count(), 12);
    }

    #[test]
    fn install_and_command() {
        let mut s = Spdu::new();
        assert!(s.install(3, SpduCardKind::Dc4));
        assert!(!s.install(3, SpduCardKind::Dc2));
        assert!(!s.install(16, SpduCardKind::Dc2));
        assert!(s.install(5, SpduCardKind::Ac1));
        let a = ChannelAddr { slot: 3, channel: 3 };
        let inp = SpduInput {
            dt_us: 10_000,
            ac_v: 115.0,
            dc_v: 28.0,
            commands: vec![ChannelCommand {
                addr: a,
                close: true,
                source: CommandSource::System,
                reset: false,
                lock: None,
            }],
            loads: vec![],
        };
        s.step(&inp);
        let out = s.step(&SpduInput {
            commands: vec![],
            ..inp
        });
        assert_eq!(out.len(), 5);
        let st = out.iter().find(|x| x.addr == a).unwrap();
        assert_eq!(st.state, SspcState::Closed);
        assert_eq!(st.rating_a, 15.0);
        let ac = out.iter().find(|x| x.addr.slot == 5).unwrap();
        assert_eq!(ac.state, SspcState::Open);
    }
}
