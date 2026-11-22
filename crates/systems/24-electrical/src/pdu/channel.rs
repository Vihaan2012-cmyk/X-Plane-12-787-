use crate::protection::sspc::{
    CommandSource, Sspc, SspcConfig, SspcInput, SspcKind, SspcState, SspcTelemetry,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelAddr {
    pub slot: u8,
    pub channel: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefaultState {
    Off,
    On,
    LastCommanded,
}

#[derive(Clone, Copy, Debug)]
pub struct ChannelCommand {
    pub addr: ChannelAddr,
    pub close: bool,
    pub source: CommandSource,
    pub reset: bool,
    pub lock: Option<bool>,
}

#[derive(Clone, Copy, Debug)]
pub struct ChannelLoad {
    pub addr: ChannelAddr,
    pub currents_a: [f64; 3],
    pub noise_a: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ChannelStatus {
    pub addr: ChannelAddr,
    pub rating_a: f64,
    pub state: SspcState,
    pub v_out: f64,
    pub current_a: f64,
    pub telemetry: SspcTelemetry,
    pub default_active: bool,
    pub inop_tag: bool,
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub sspc: Sspc,
    pub commanded: bool,
    pub default: DefaultState,
    pub locked: bool,
    pub reset_pending: bool,
    pub source: CommandSource,
}

impl Channel {
    pub fn new(config: SspcConfig) -> Self {
        Self {
            sspc: Sspc::new(config),
            commanded: false,
            default: DefaultState::Off,
            locked: false,
            reset_pending: false,
            source: CommandSource::None,
        }
    }

    pub fn apply(&mut self, cmd: &ChannelCommand) {
        self.commanded = cmd.close;
        self.source = cmd.source;
        if cmd.reset {
            self.reset_pending = true;
        }
        if let Some(l) = cmd.lock {
            self.locked = l;
        }
    }

    pub fn effective_close(&self, default_active: bool) -> bool {
        if !default_active {
            return self.commanded;
        }
        match self.default {
            DefaultState::Off => false,
            DefaultState::On => true,
            DefaultState::LastCommanded => self.commanded,
        }
    }

    pub fn step(
        &mut self,
        addr: ChannelAddr,
        dt_us: u64,
        v_in: f64,
        load: Option<&ChannelLoad>,
        default_active: bool,
    ) -> ChannelStatus {
        let close = self.effective_close(default_active);
        let source = if default_active {
            CommandSource::System
        } else {
            self.source
        };
        let o = self.sspc.step(&SspcInput {
            dt_us,
            v_in,
            currents_a: load.map(|l| l.currents_a).unwrap_or([0.0; 3]),
            noise_a: load.map(|l| l.noise_a).unwrap_or(0.0),
            close_cmd: close,
            source,
            reset: self.reset_pending,
            lock: self.locked,
            v_zero_cross: true,
            i_zero_cross: true,
        });
        self.reset_pending = false;
        ChannelStatus {
            addr,
            rating_a: self.sspc.rating_a(),
            state: o.state,
            v_out: o.v_out,
            current_a: o.telemetry.current_a,
            telemetry: o.telemetry,
            default_active,
            inop_tag: o.inop_tag,
        }
    }

    pub fn is_ac(&self) -> bool {
        self.sspc.config.kind == SspcKind::AcZeroCross
    }
}
