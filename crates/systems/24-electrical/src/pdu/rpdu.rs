use super::channel::{Channel, ChannelAddr, ChannelCommand, ChannelLoad, ChannelStatus, DefaultState};
use crate::protection::sspc::SspcConfig;

pub const FIRST_PM_SLOT: u8 = 2;
pub const LAST_PM_SLOT: u8 = 7;
pub const COMMS_LOSS_DEFAULT_US: u64 = 3_000_000;
pub const CONTROLLER_HOLDUP_US: u64 = 50_000;
pub const CONTROLLER_MIN_V: f64 = 18.0;
pub const DC_SPLIT_SLOT: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpduKind {
    Gateway,
    Standard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerModuleKind {
    Ac12,
    Dc19,
    Spare,
}

pub const AC12_RATINGS_A: [f64; 12] = [2.5, 2.5, 2.5, 5.0, 5.0, 5.0, 7.5, 7.5, 7.5, 10.0, 10.0, 10.0];
pub const DC19_RATINGS_A: [f64; 19] = [
    10.0, 10.0, 5.0, 5.0, 5.0, 5.0, 5.0, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5,
];

#[derive(Clone, Debug)]
pub struct PowerModule {
    pub kind: PowerModuleKind,
    pub channels: Vec<Channel>,
}

impl PowerModule {
    pub fn new(kind: PowerModuleKind) -> Self {
        let channels = match kind {
            PowerModuleKind::Ac12 => AC12_RATINGS_A
                .iter()
                .map(|r| Channel::new(SspcConfig::ac(*r, 1)))
                .collect(),
            PowerModuleKind::Dc19 => DC19_RATINGS_A
                .iter()
                .map(|r| Channel::new(SspcConfig::dc(*r)))
                .collect(),
            PowerModuleKind::Spare => Vec::new(),
        };
        Self { kind, channels }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RpduInput {
    pub dt_us: u64,
    pub ac_v: f64,
    pub dc1_v: f64,
    pub dc2_v: f64,
    pub backup_28v: f64,
    pub comms_ok: bool,
    pub commands: Vec<ChannelCommand>,
    pub loads: Vec<ChannelLoad>,
}

#[derive(Clone, Debug)]
pub struct RpduOutput {
    pub controller_powered: bool,
    pub default_active: bool,
    pub statuses: Vec<ChannelStatus>,
}

#[derive(Clone, Debug)]
pub struct Rpdu {
    pub kind: RpduKind,
    pub modules: Vec<PowerModule>,
    holdup_us: u64,
    comms_lost_us: u64,
    last: Vec<ChannelStatus>,
}

impl Rpdu {
    pub fn new(kind: RpduKind, slots: [PowerModuleKind; 6]) -> Self {
        Self {
            kind,
            modules: slots.iter().map(|k| PowerModule::new(*k)).collect(),
            holdup_us: 0,
            comms_lost_us: 0,
            last: Vec::new(),
        }
    }

    fn module_index(slot: u8) -> Option<usize> {
        if (FIRST_PM_SLOT..=LAST_PM_SLOT).contains(&slot) {
            Some((slot - FIRST_PM_SLOT) as usize)
        } else {
            None
        }
    }

    pub fn channel_mut(&mut self, addr: ChannelAddr) -> Option<&mut Channel> {
        let m = Self::module_index(addr.slot)?;
        self.modules.get_mut(m)?.channels.get_mut(addr.channel as usize)
    }

    pub fn set_default(&mut self, addr: ChannelAddr, d: DefaultState) -> bool {
        match self.channel_mut(addr) {
            Some(c) => {
                c.default = d;
                true
            }
            None => false,
        }
    }

    pub fn program_phases(&mut self, addr: ChannelAddr, phases: u8) -> bool {
        match self.channel_mut(addr) {
            Some(c) if c.is_ac() => {
                let rating = c.sspc.rating_a();
                let (commanded, default, locked) = (c.commanded, c.default, c.locked);
                *c = Channel::new(SspcConfig::ac(rating, phases));
                c.commanded = commanded;
                c.default = default;
                c.locked = locked;
                true
            }
            _ => false,
        }
    }

    pub fn backup_output_v(&self, addr: ChannelAddr) -> f64 {
        self.last
            .iter()
            .find(|s| s.addr == addr)
            .map(|s| s.v_out)
            .unwrap_or(0.0)
    }

    pub fn controller_powered(&self) -> bool {
        self.holdup_us > 0
    }

    pub fn step(&mut self, input: &RpduInput) -> RpduOutput {
        let supplied = input.dc1_v >= CONTROLLER_MIN_V
            || input.dc2_v >= CONTROLLER_MIN_V
            || input.backup_28v >= CONTROLLER_MIN_V;
        if supplied {
            self.holdup_us = CONTROLLER_HOLDUP_US;
        } else {
            self.holdup_us = self.holdup_us.saturating_sub(input.dt_us);
        }
        let powered = self.holdup_us > 0;
        if input.comms_ok && powered {
            self.comms_lost_us = 0;
            for cmd in &input.commands {
                if let Some(c) = self.channel_mut(cmd.addr) {
                    c.apply(cmd);
                }
            }
        } else {
            self.comms_lost_us = self.comms_lost_us.saturating_add(input.dt_us);
        }
        let default_active = !powered || self.comms_lost_us >= COMMS_LOSS_DEFAULT_US;
        let mut statuses = Vec::new();
        for (mi, module) in self.modules.iter_mut().enumerate() {
            let slot = FIRST_PM_SLOT + mi as u8;
            let v_in = match module.kind {
                PowerModuleKind::Ac12 => input.ac_v,
                PowerModuleKind::Dc19 => {
                    if slot < DC_SPLIT_SLOT + 1 {
                        input.dc1_v
                    } else {
                        input.dc2_v
                    }
                }
                PowerModuleKind::Spare => 0.0,
            };
            for (ci, ch) in module.channels.iter_mut().enumerate() {
                let addr = ChannelAddr {
                    slot,
                    channel: ci as u8,
                };
                let load = input.loads.iter().find(|l| l.addr == addr);
                statuses.push(ch.step(addr, input.dt_us, v_in, load, default_active));
            }
        }
        self.last = statuses.clone();
        RpduOutput {
            controller_powered: powered,
            default_active,
            statuses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protection::sspc::{CommandSource, SspcState};

    fn rpdu() -> Rpdu {
        use PowerModuleKind::*;
        Rpdu::new(RpduKind::Standard, [Dc19, Dc19, Ac12, Dc19, Ac12, Spare])
    }

    fn addr(slot: u8, channel: u8) -> ChannelAddr {
        ChannelAddr { slot, channel }
    }

    fn cmd(a: ChannelAddr, close: bool) -> ChannelCommand {
        ChannelCommand {
            addr: a,
            close,
            source: CommandSource::System,
            reset: false,
            lock: None,
        }
    }

    fn base() -> RpduInput {
        RpduInput {
            dt_us: 10_000,
            ac_v: 115.0,
            dc1_v: 28.0,
            dc2_v: 28.0,
            backup_28v: 0.0,
            comms_ok: true,
            ..Default::default()
        }
    }

    fn state_of(o: &RpduOutput, a: ChannelAddr) -> SspcState {
        o.statuses.iter().find(|s| s.addr == a).unwrap().state
    }

    #[test]
    fn channel_mix_matches_documentation() {
        let ac = PowerModule::new(PowerModuleKind::Ac12);
        assert_eq!(ac.channels.len(), 12);
        for r in [2.5, 5.0, 7.5, 10.0] {
            assert_eq!(ac.channels.iter().filter(|c| c.sspc.rating_a() == r).count(), 3);
        }
        let dc = PowerModule::new(PowerModuleKind::Dc19);
        assert_eq!(dc.channels.len(), 19);
        assert_eq!(
            dc.channels.iter().filter(|c| c.sspc.rating_a() == 10.0).count(),
            2
        );
        assert_eq!(dc.channels.iter().filter(|c| c.sspc.rating_a() == 5.0).count(), 5);
        assert_eq!(
            dc.channels.iter().filter(|c| c.sspc.rating_a() == 2.5).count(),
            12
        );
        assert!(PowerModule::new(PowerModuleKind::Spare).channels.is_empty());
    }

    #[test]
    fn command_closes_addressed_channel_only() {
        let mut r = rpdu();
        let mut inp = base();
        inp.commands = vec![cmd(addr(2, 0), true)];
        let o = r.step(&inp);
        inp.commands.clear();
        let o2 = r.step(&inp);
        assert_eq!(state_of(&o2, addr(2, 0)), SspcState::Closed);
        assert_eq!(state_of(&o2, addr(2, 1)), SspcState::Open);
        assert!(o.controller_powered);
        assert_eq!(o2.statuses.len(), 19 * 3 + 12 * 2);
    }

    #[test]
    fn dc_source_split_between_slots() {
        let mut r = rpdu();
        let mut inp = base();
        inp.dc2_v = 0.0;
        inp.commands = vec![cmd(addr(2, 0), true), cmd(addr(5, 0), true)];
        r.step(&inp);
        inp.commands.clear();
        let o = r.step(&inp);
        assert_eq!(state_of(&o, addr(2, 0)), SspcState::Closed);
        assert_eq!(state_of(&o, addr(5, 0)), SspcState::Open);
    }

    #[test]
    fn data_loss_applies_default_after_three_seconds() {
        let mut r = rpdu();
        r.set_default(addr(2, 0), DefaultState::Off);
        r.set_default(addr(2, 1), DefaultState::On);
        r.set_default(addr(2, 2), DefaultState::LastCommanded);
        let mut inp = base();
        inp.commands = vec![
            cmd(addr(2, 0), true),
            cmd(addr(2, 1), false),
            cmd(addr(2, 2), true),
        ];
        r.step(&inp);
        inp.commands.clear();
        let o = r.step(&inp);
        assert_eq!(state_of(&o, addr(2, 0)), SspcState::Closed);
        assert_eq!(state_of(&o, addr(2, 1)), SspcState::Open);
        inp.comms_ok = false;
        let mut o = r.step(&inp);
        for _ in 0..290 {
            o = r.step(&inp);
        }
        assert!(!o.default_active);
        assert_eq!(state_of(&o, addr(2, 0)), SspcState::Closed);
        for _ in 0..20 {
            o = r.step(&inp);
        }
        assert!(o.default_active);
        assert_eq!(state_of(&o, addr(2, 0)), SspcState::Open);
        assert_eq!(state_of(&o, addr(2, 1)), SspcState::Closed);
        assert_eq!(state_of(&o, addr(2, 2)), SspcState::Closed);
        inp.comms_ok = true;
        let o = r.step(&inp);
        assert!(!o.default_active);
        assert_eq!(state_of(&o, addr(2, 0)), SspcState::Closed);
    }

    #[test]
    fn controller_power_loss_uses_holdup_then_defaults() {
        let mut r = rpdu();
        r.set_default(addr(2, 0), DefaultState::Off);
        let mut inp = base();
        inp.commands = vec![cmd(addr(2, 0), true)];
        r.step(&inp);
        inp.commands.clear();
        r.step(&inp);
        inp.dc1_v = 0.0;
        inp.dc2_v = 0.0;
        let o = r.step(&inp);
        assert!(o.controller_powered && !o.default_active);
        let mut o = o;
        for _ in 0..5 {
            o = r.step(&inp);
        }
        assert!(!o.controller_powered && o.default_active);
        assert_eq!(state_of(&o, addr(2, 0)), SspcState::Open);
    }

    #[test]
    fn backup_28v_keeps_controller_alive() {
        let mut r = rpdu();
        let mut inp = base();
        inp.dc1_v = 0.0;
        inp.dc2_v = 0.0;
        inp.backup_28v = 28.0;
        for _ in 0..100 {
            assert!(r.step(&inp).controller_powered);
        }
        inp.backup_28v = 0.0;
        for _ in 0..10 {
            r.step(&inp);
        }
        assert!(!r.controller_powered());
    }

    #[test]
    fn locked_channel_ignores_close() {
        let mut r = rpdu();
        let mut inp = base();
        inp.commands = vec![ChannelCommand {
            lock: Some(true),
            ..cmd(addr(2, 3), true)
        }];
        r.step(&inp);
        inp.commands = vec![cmd(addr(2, 3), true)];
        let o = r.step(&inp);
        let s = o.statuses.iter().find(|s| s.addr == addr(2, 3)).unwrap();
        assert_eq!(s.state, SspcState::LockedOpen);
        assert!(s.inop_tag);
        assert_eq!(s.v_out, 0.0);
    }

    #[test]
    fn overload_trips_and_reports() {
        let mut r = rpdu();
        let mut inp = base();
        inp.commands = vec![cmd(addr(2, 0), true)];
        inp.loads = vec![ChannelLoad {
            addr: addr(2, 0),
            currents_a: [200.0, 0.0, 0.0],
            noise_a: 0.0,
        }];
        r.step(&inp);
        inp.commands.clear();
        let o = r.step(&inp);
        assert_eq!(state_of(&o, addr(2, 0)), SspcState::Tripped);
    }

    #[test]
    fn phase_programming_and_backup_output() {
        let mut r = rpdu();
        assert!(r.program_phases(addr(4, 0), 3));
        assert!(!r.program_phases(addr(2, 0), 3));
        assert_eq!(r.channel_mut(addr(4, 0)).unwrap().sspc.config.phases, 3);
        let mut inp = base();
        inp.commands = vec![cmd(addr(3, 0), true)];
        r.step(&inp);
        inp.commands.clear();
        for _ in 0..2 {
            r.step(&inp);
        }
        assert!(r.backup_output_v(addr(3, 0)) > 27.0);
        assert!(r.channel_mut(addr(1, 0)).is_none());
        assert!(r.channel_mut(addr(8, 0)).is_none());
    }
}
