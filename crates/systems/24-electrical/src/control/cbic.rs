#![allow(clippy::needless_range_loop, clippy::collapsible_if)]

use std::collections::BTreeMap;

pub const MAX_MFD_USERS: usize = 3;
pub const MAX_LAPTOP_USERS: usize = 1;
pub const MAX_RECENT: usize = 30;
pub const LIMITED_SPEED_KT: f64 = 2.0;
pub const CONFIRM_TIMEOUT_US: u64 = 10_000_000;
pub const CB_STATUS_PAGE_SIZE: usize = 15;
pub const HOST_COUNT: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerKind {
    Ecb,
    Tcb,
    Elcc,
    FcSsr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockTag {
    Inop,
    DoNotClose,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CbState {
    Open,
    Closed,
    Tripped,
    Locked(LockTag),
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Breaker {
    pub id: u32,
    pub name: String,
    pub kind: BreakerKind,
    pub state: CbState,
    pub output_override: bool,
    pub load_amps: f64,
    pub lock_set_from_laptop: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Ground,
    Maintenance,
    Limited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserKind {
    Mfd,
    Laptop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CbAction {
    Open,
    Close,
    Lock(LockTag),
    Unlock,
    RequestReset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    MaxUsers,
    NoSession,
    NotFound,
    IndicationOnly,
    LaptopControlOnGroundOnly,
    LimitedModeNoLockControl,
    LimitedModeResetOnly,
    FlightControlBlockedInFlight,
    NoLockOnFcSsr,
    InopLockEcbOnly,
    LockedUnlockFirst,
    TrippedResetFirst,
    MustBeOpenToLock,
    NotLocked,
    NotTripped,
    CommunicationUnknown,
    CtrlRequired,
    NoPendingCommand,
    ConfirmationExpired,
    CustomListMaintenanceOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmdOutcome {
    Executed,
    PendingEnter,
    Refused(Refusal),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BreakerCommand {
    pub id: u32,
    pub action: CbAction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CbicInput {
    pub now_us: u64,
    pub on_ground: bool,
    pub ground_speed_kt: f64,
    pub doors_open: bool,
    pub ext_power_avail: bool,
    pub gnd_test_enable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nvm {
    pub airplane_seq: u32,
    pub locks: Vec<(u32, LockTag, bool)>,
}

#[derive(Clone, Copy, Debug)]
struct Pending {
    id: u32,
    action: CbAction,
    at_us: u64,
}

#[derive(Clone, Debug)]
pub struct Cbic {
    breakers: BTreeMap<u32, Breaker>,
    users: BTreeMap<UserId, UserKind>,
    pending: BTreeMap<UserId, Pending>,
    next_user: u32,
    outgoing: Vec<BreakerCommand>,
    recent: Vec<u32>,
    custom: Vec<u32>,
    mode: Mode,
    on_ground: bool,
    now_us: u64,
    pub nvm_discarded: bool,
}

impl Default for Cbic {
    fn default() -> Self {
        Self::new()
    }
}

impl Cbic {
    pub fn new() -> Self {
        Self {
            breakers: BTreeMap::new(),
            users: BTreeMap::new(),
            pending: BTreeMap::new(),
            next_user: 1,
            outgoing: Vec::new(),
            recent: Vec::new(),
            custom: Vec::new(),
            mode: Mode::Limited,
            on_ground: false,
            now_us: 0,
            nvm_discarded: false,
        }
    }

    pub fn add_breaker(&mut self, id: u32, name: &str, kind: BreakerKind, state: CbState) {
        let state = if kind == BreakerKind::Tcb {
            match state {
                CbState::Open | CbState::Closed | CbState::Unknown => state,
                _ => CbState::Unknown,
            }
        } else {
            state
        };
        self.breakers.insert(
            id,
            Breaker {
                id,
                name: name.to_string(),
                kind,
                state,
                output_override: false,
                load_amps: 0.0,
                lock_set_from_laptop: false,
            },
        );
    }

    pub fn breaker(&self, id: u32) -> Option<&Breaker> {
        self.breakers.get(&id)
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    pub fn update(&mut self, inp: &CbicInput) {
        self.now_us = inp.now_us;
        self.on_ground = inp.on_ground;
        let ground_cond = inp.on_ground
            && inp.ground_speed_kt <= LIMITED_SPEED_KT
            && (inp.doors_open || inp.ext_power_avail);
        self.mode = if !ground_cond {
            Mode::Limited
        } else if inp.gnd_test_enable {
            Mode::Maintenance
        } else {
            Mode::Ground
        };
        let now = self.now_us;
        let expired: Vec<UserId> = self
            .pending
            .iter()
            .filter(|(_, p)| now.saturating_sub(p.at_us) > CONFIRM_TIMEOUT_US)
            .map(|(u, _)| *u)
            .collect();
        for u in expired {
            self.pending.remove(&u);
        }
        if self.mode != Mode::Maintenance {
            self.custom.clear();
        }
    }

    pub fn connect(&mut self, kind: UserKind) -> Result<UserId, Refusal> {
        let mfd = self.users.values().filter(|&&k| k == UserKind::Mfd).count();
        let lap = self.users.len() - mfd;
        let full = match kind {
            UserKind::Mfd => mfd >= MAX_MFD_USERS,
            UserKind::Laptop => lap >= MAX_LAPTOP_USERS,
        };
        if full || self.users.len() >= MAX_MFD_USERS + MAX_LAPTOP_USERS {
            return Err(Refusal::MaxUsers);
        }
        let id = UserId(self.next_user);
        self.next_user += 1;
        self.users.insert(id, kind);
        Ok(id)
    }

    pub fn disconnect(&mut self, user: UserId) {
        self.users.remove(&user);
        self.pending.remove(&user);
    }

    fn validate(&self, user: UserId, id: u32, action: CbAction) -> Result<(), Refusal> {
        let kind = *self.users.get(&user).ok_or(Refusal::NoSession)?;
        let b = self.breakers.get(&id).ok_or(Refusal::NotFound)?;
        if b.kind == BreakerKind::Tcb {
            return Err(Refusal::IndicationOnly);
        }
        if kind == UserKind::Laptop && !self.on_ground {
            return Err(Refusal::LaptopControlOnGroundOnly);
        }
        if b.kind == BreakerKind::FcSsr {
            if !self.on_ground {
                return Err(Refusal::FlightControlBlockedInFlight);
            }
            if matches!(action, CbAction::Lock(_) | CbAction::Unlock) {
                return Err(Refusal::NoLockOnFcSsr);
            }
        }
        if self.mode == Mode::Limited {
            if matches!(action, CbAction::Lock(_) | CbAction::Unlock) {
                return Err(Refusal::LimitedModeNoLockControl);
            }
            if b.kind == BreakerKind::Elcc && action != CbAction::RequestReset {
                return Err(Refusal::LimitedModeResetOnly);
            }
        }
        if b.state == CbState::Unknown {
            return Err(Refusal::CommunicationUnknown);
        }
        match action {
            CbAction::Open | CbAction::Close => {
                if matches!(b.state, CbState::Locked(_)) {
                    return Err(Refusal::LockedUnlockFirst);
                }
                if action == CbAction::Close && b.state == CbState::Tripped {
                    return Err(Refusal::TrippedResetFirst);
                }
            }
            CbAction::Lock(tag) => {
                if tag == LockTag::Inop && b.kind != BreakerKind::Ecb {
                    return Err(Refusal::InopLockEcbOnly);
                }
                if matches!(b.state, CbState::Locked(_)) {
                    return Err(Refusal::LockedUnlockFirst);
                }
                if b.state != CbState::Open {
                    return Err(Refusal::MustBeOpenToLock);
                }
            }
            CbAction::Unlock => {
                if !matches!(b.state, CbState::Locked(_)) {
                    return Err(Refusal::NotLocked);
                }
            }
            CbAction::RequestReset => {
                if b.state != CbState::Tripped {
                    return Err(Refusal::NotTripped);
                }
            }
        }
        Ok(())
    }

    fn execute(&mut self, user: UserId, id: u32, action: CbAction) {
        let from_laptop = self.users.get(&user) == Some(&UserKind::Laptop);
        if let Some(b) = self.breakers.get_mut(&id) {
            match action {
                CbAction::Lock(tag) => {
                    b.state = CbState::Locked(tag);
                    b.lock_set_from_laptop = from_laptop;
                }
                CbAction::Unlock => {
                    b.state = CbState::Open;
                    b.lock_set_from_laptop = false;
                }
                _ => {}
            }
        }
        self.outgoing.push(BreakerCommand { id, action });
        self.recent.retain(|&r| r != id);
        self.recent.insert(0, id);
        self.recent.truncate(MAX_RECENT);
    }

    pub fn command(&mut self, user: UserId, id: u32, action: CbAction, ctrl_armed: bool) -> CmdOutcome {
        if let Err(r) = self.validate(user, id, action) {
            return CmdOutcome::Refused(r);
        }
        if self.on_ground {
            if !ctrl_armed {
                return CmdOutcome::Refused(Refusal::CtrlRequired);
            }
            self.execute(user, id, action);
            CmdOutcome::Executed
        } else {
            self.pending.insert(
                user,
                Pending {
                    id,
                    action,
                    at_us: self.now_us,
                },
            );
            CmdOutcome::PendingEnter
        }
    }

    pub fn press_enter(&mut self, user: UserId) -> CmdOutcome {
        let Some(p) = self.pending.remove(&user) else {
            return CmdOutcome::Refused(Refusal::NoPendingCommand);
        };
        if self.now_us.saturating_sub(p.at_us) > CONFIRM_TIMEOUT_US {
            return CmdOutcome::Refused(Refusal::ConfirmationExpired);
        }
        if let Err(r) = self.validate(user, p.id, p.action) {
            return CmdOutcome::Refused(r);
        }
        self.execute(user, p.id, p.action);
        CmdOutcome::Executed
    }

    pub fn take_commands(&mut self) -> Vec<BreakerCommand> {
        std::mem::take(&mut self.outgoing)
    }

    pub fn report_status(&mut self, id: u32, reported: CbState, load_amps: f64) {
        if let Some(b) = self.breakers.get_mut(&id) {
            let keep_lock = matches!(b.state, CbState::Locked(_))
                && matches!(reported, CbState::Open | CbState::Locked(_));
            if keep_lock {
                b.load_amps = load_amps;
                return;
            }
            b.state = if b.kind == BreakerKind::Tcb {
                match reported {
                    CbState::Open | CbState::Closed => reported,
                    _ => CbState::Unknown,
                }
            } else {
                reported
            };
            b.load_amps = load_amps;
        }
    }

    pub fn report_comm_loss(&mut self, id: u32) {
        if let Some(b) = self.breakers.get_mut(&id) {
            if !matches!(b.state, CbState::Locked(_)) {
                b.state = CbState::Unknown;
            }
        }
    }

    pub fn report_override(&mut self, id: u32, shed: bool) {
        if let Some(b) = self.breakers.get_mut(&id) {
            b.output_override = shed;
        }
    }

    pub fn output_status(&self, id: u32) -> Option<&'static str> {
        let b = self.breakers.get(&id)?;
        if b.output_override {
            Some("OVERRIDE")
        } else if b.state == CbState::Closed {
            Some("ON")
        } else {
            Some("OFF")
        }
    }

    pub fn open_trip_page(&self) -> Vec<u32> {
        self.breakers
            .values()
            .filter(|b| {
                b.kind != BreakerKind::Tcb
                    && matches!(b.state, CbState::Open | CbState::Tripped | CbState::Locked(_))
            })
            .map(|b| b.id)
            .collect()
    }

    fn state_rank(s: CbState) -> Option<u8> {
        match s {
            CbState::Tripped => Some(0),
            CbState::Open => Some(1),
            CbState::Locked(_) => Some(2),
            CbState::Unknown => Some(3),
            CbState::Closed => None,
        }
    }

    pub fn by_state(&self) -> Vec<u32> {
        let mut v: Vec<(u8, u32)> = self
            .breakers
            .values()
            .filter(|b| b.kind != BreakerKind::Tcb)
            .filter_map(|b| Self::state_rank(b.state).map(|r| (r, b.id)))
            .collect();
        v.sort();
        v.into_iter().map(|(_, id)| id).collect()
    }

    pub fn status_pages(&self, ids: &[u32]) -> Vec<Vec<u32>> {
        ids.chunks(CB_STATUS_PAGE_SIZE).map(<[u32]>::to_vec).collect()
    }

    pub fn recent_used(&self) -> &[u32] {
        &self.recent
    }

    pub fn search(&self, name: &str) -> Result<Vec<u32>, &'static str> {
        let v: Vec<u32> = self
            .breakers
            .values()
            .filter(|b| b.name.contains(name))
            .map(|b| b.id)
            .collect();
        if v.is_empty() {
            Err("CB WAS NOT FOUND")
        } else {
            Ok(v)
        }
    }

    pub fn set_custom_list(&mut self, ids: &[u32]) -> Result<(), Refusal> {
        if self.mode != Mode::Maintenance {
            return Err(Refusal::CustomListMaintenanceOnly);
        }
        self.custom = ids
            .iter()
            .copied()
            .filter(|i| self.breakers.contains_key(i))
            .collect();
        Ok(())
    }

    pub fn custom_list(&self) -> &[u32] {
        &self.custom
    }

    pub fn export_nvm(&self, airplane_seq: u32) -> Nvm {
        let locks = self
            .breakers
            .values()
            .filter_map(|b| match b.state {
                CbState::Locked(t) => Some((b.id, t, b.lock_set_from_laptop)),
                _ => None,
            })
            .collect();
        Nvm { airplane_seq, locks }
    }

    pub fn power_cycle(&mut self) {
        self.users.clear();
        self.pending.clear();
        self.recent.clear();
        self.custom.clear();
        self.outgoing.clear();
        for b in self.breakers.values_mut() {
            b.state = CbState::Unknown;
            b.output_override = false;
            b.load_amps = 0.0;
        }
    }

    pub fn restore_nvm(&mut self, nvm: &Nvm, airplane_seq: u32) {
        if nvm.airplane_seq != airplane_seq {
            self.nvm_discarded = true;
            return;
        }
        self.nvm_discarded = false;
        for &(id, tag, laptop) in &nvm.locks {
            if let Some(b) = self.breakers.get_mut(&id) {
                if b.kind != BreakerKind::Tcb {
                    b.state = CbState::Locked(tag);
                    b.lock_set_from_laptop = laptop;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CbicCluster {
    pub app: Cbic,
    host_up: [bool; HOST_COUNT],
    nvm: Nvm,
    airplane_seq: u32,
    powered: bool,
}

impl CbicCluster {
    pub fn new(app: Cbic, airplane_seq: u32) -> Self {
        let nvm = app.export_nvm(airplane_seq);
        Self {
            app,
            host_up: [true; HOST_COUNT],
            nvm,
            airplane_seq,
            powered: true,
        }
    }

    pub fn active_host(&self) -> Option<usize> {
        self.host_up.iter().position(|&u| u)
    }

    pub fn is_powered(&self) -> bool {
        self.powered
    }

    pub fn sync_nvm(&mut self) {
        if self.powered {
            self.nvm = self.app.export_nvm(self.airplane_seq);
        }
    }

    pub fn set_host_power(&mut self, host: usize, up: bool) {
        self.sync_nvm();
        self.host_up[host] = up;
        let any = self.host_up.iter().any(|&u| u);
        if self.powered && !any {
            self.app.power_cycle();
            self.powered = false;
        } else if !self.powered && any {
            let nvm = self.nvm.clone();
            self.app.restore_nvm(&nvm, self.airplane_seq);
            self.powered = true;
        }
    }

    pub fn set_airplane_seq(&mut self, seq: u32) {
        self.airplane_seq = seq;
    }
}
