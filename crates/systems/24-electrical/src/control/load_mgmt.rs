#![allow(clippy::needless_range_loop, clippy::collapsible_if)]

pub const MIN_SHED_US: u64 = 5_000_000;
pub const RESTORE_DELAY_US: u64 = 10_000_000;
pub const SHED_SPACING_US: u64 = 500_000;
pub const RESTORE_MARGIN_KW: f64 = 5.0;
pub const EP_5MIN_KVA: f64 = 90.0;
pub const EP_5MIN_US: u64 = 5 * 60 * 1_000_000;
pub const EP_5S_KVA: f64 = 112.5;
pub const EP_5S_US: u64 = 5_000_000;
pub const EP_SHED_INTERVAL_US: u64 = 5_000_000;
pub const ENGINE_START_RESTORE_HOLD_US: u64 = 120_000_000;
pub const VFSG_CONTINUOUS_KVA: f64 = 250.0;
pub const VFSG_5MIN_KVA: f64 = 312.5;
pub const GEN_5MIN_RATIO: f64 = VFSG_5MIN_KVA / VFSG_CONTINUOUS_KVA;
pub const VFSG_5S_KVA: f64 = 437.5;
pub const GEN_5S_RATIO: f64 = VFSG_5S_KVA / VFSG_CONTINUOUS_KVA;
pub const GEN_5MIN_US: u64 = 5 * 60 * 1_000_000;
pub const GEN_5S_US: u64 = 5_000_000;
pub const APPROX_CAC_MIN_SPEED: f64 = 0.5;
pub const CAC_LOAD_INDEX: usize = 8;
pub const HPX_REPEAT_WINDOW_US: u64 = 20 * 60 * 1_000_000;
pub const PECS_THERMAL_SHED_F: f64 = 148.0;
pub const PECS_THERMAL_SHED_C: f64 = (PECS_THERMAL_SHED_F - 32.0) / 1.8;
pub const APPROX_PECS_RESTORE_C: f64 = 60.0;
pub const PECS_SHED_ORDER: [usize; 5] = [3, 2, 1, 0, 4];
pub const EP_CMSC_STOP_A: f64 = 260.0;
pub const EP_CMSC_STOP_US: u64 = 4 * 60 * 1_000_000;
pub const EP_CMSC_STOP_FAST_A: f64 = 313.0;
pub const EP_CMSC_STOP_FAST_US: u64 = 5_000_000;
pub const EP_90_CMSC_STOP_A: f64 = 208.0;
pub const EP_90_CMSC_STOP_FAST_A: f64 = 260.0;
pub const HYD_LOAD_INDEX: usize = 10;
pub const BUDGET_ECS_KW: f64 = 320.0;
pub const BUDGET_HYDRAULICS_KW: f64 = 40.0;
pub const BUDGET_ECS_FANS_KW: f64 = 32.0;
pub const BUDGET_GALLEYS_KW: f64 = 120.0;
pub const APPROX_HPX_RESTORE_US: u64 = 60_000_000;
pub const HPX_GROUPS: [LoadGroup; 3] = [LoadGroup::Cabin, LoadGroup::MiscHeaters, LoadGroup::WindowHeat];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadGroup {
    Hyd,
    Fuel,
    Air,
    WindowHeat,
    MiscHeaters,
    ExteriorLights,
    CommNav,
    Cabin,
}

#[derive(Clone, Copy, Debug)]
pub struct LoadDef {
    pub name: &'static str,
    pub group: LoadGroup,
    pub kw: f64,
    pub ground_priority: u8,
    pub flight_priority: u8,
    pub shed_on_engine_start: bool,
    pub sheddable: bool,
}

pub const DEFAULT_LOADS: [LoadDef; 14] = [
    LoadDef {
        name: "GALLEY_FWD",
        group: LoadGroup::Cabin,
        kw: BUDGET_GALLEYS_KW / 2.0,
        ground_priority: 9,
        flight_priority: 9,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "GALLEY_AFT",
        group: LoadGroup::Cabin,
        kw: BUDGET_GALLEYS_KW / 2.0,
        ground_priority: 9,
        flight_priority: 9,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "IFE",
        group: LoadGroup::Cabin,
        kw: 30.0,
        ground_priority: 8,
        flight_priority: 8,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "CABIN_LIGHTING",
        group: LoadGroup::Cabin,
        kw: 10.0,
        ground_priority: 7,
        flight_priority: 7,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "CARGO_HEAT",
        group: LoadGroup::MiscHeaters,
        kw: 15.0,
        ground_priority: 8,
        flight_priority: 6,
        shed_on_engine_start: false,
        sheddable: true,
    },
    LoadDef {
        name: "RECIRC_FANS",
        group: LoadGroup::Air,
        kw: BUDGET_ECS_FANS_KW,
        ground_priority: 6,
        flight_priority: 5,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "RAM_FAN",
        group: LoadGroup::Air,
        kw: 5.0,
        ground_priority: 6,
        flight_priority: 4,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "WINDOW_HEAT",
        group: LoadGroup::WindowHeat,
        kw: 25.0,
        ground_priority: 7,
        flight_priority: 3,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "ECS_PACKS",
        group: LoadGroup::Air,
        kw: BUDGET_ECS_KW,
        ground_priority: 5,
        flight_priority: 2,
        shed_on_engine_start: true,
        sheddable: true,
    },
    LoadDef {
        name: "FUEL_PUMPS_CTR",
        group: LoadGroup::Fuel,
        kw: 10.0,
        ground_priority: 4,
        flight_priority: 4,
        shed_on_engine_start: false,
        sheddable: true,
    },
    LoadDef {
        name: "HYD_DEMAND_PUMPS",
        group: LoadGroup::Hyd,
        kw: BUDGET_HYDRAULICS_KW,
        ground_priority: 3,
        flight_priority: 2,
        shed_on_engine_start: false,
        sheddable: true,
    },
    LoadDef {
        name: "EXTERIOR_LIGHTS",
        group: LoadGroup::ExteriorLights,
        kw: 8.0,
        ground_priority: 5,
        flight_priority: 3,
        shed_on_engine_start: false,
        sheddable: true,
    },
    LoadDef {
        name: "COMM_NAV",
        group: LoadGroup::CommNav,
        kw: 12.0,
        ground_priority: 2,
        flight_priority: 1,
        shed_on_engine_start: false,
        sheddable: true,
    },
    LoadDef {
        name: "FLIGHT_DISPLAYS_ESSENTIAL",
        group: LoadGroup::CommNav,
        kw: 6.0,
        ground_priority: 1,
        flight_priority: 0,
        shed_on_engine_start: false,
        sheddable: false,
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LmInput {
    pub now_us: u64,
    pub on_ground: bool,
    pub available_kw: f64,
    pub demand_on: [bool; 14],
    pub engine_start: bool,
    pub source_change: bool,
    pub ext_in_use: bool,
    pub ext_kva: f64,
    pub overload_hint_kw: f64,
    pub hpx_kw: [f64; 2],
    pub hpx_limit_kw: [f64; 2],
    pub pecs_coolant_c: f64,
    pub ext_phase_a: f64,
    pub ext_single_90: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LmOutput {
    pub shed: [bool; 14],
    pub override_status: [bool; 14],
    pub breaker_closed: [bool; 14],
    pub total_kw: f64,
    pub shed_count: usize,
    pub ext_overload_5s: bool,
    pub ext_overload_5min: bool,
    pub cac_speed: f64,
    pub hpx_groups_shed: usize,
    pub pecs_thermal_shed: usize,
    pub emp_cmsc_stopped: bool,
}

#[derive(Clone, Debug)]
pub struct LoadManager {
    shed: [bool; 14],
    shed_at: [u64; 14],
    last_action_us: u64,
    stable_since: Option<u64>,
    engine_start_until: u64,
    gen5s_since: Option<u64>,
    gen5min_since: Option<u64>,
    ep_forced: bool,
    pub bpcu_reset_shed: bool,
    hpx_active: bool,
    hpx_groups: usize,
    hpx_last_us: Option<u64>,
    hpx_clear_since: Option<u64>,
    pecs_shed: usize,
    pecs_last_us: u64,
    ep_slow_since: Option<u64>,
    ep_fast_since: Option<u64>,
    emp_stopped: bool,
}

impl Default for LoadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadManager {
    pub fn new() -> Self {
        Self {
            shed: [false; 14],
            shed_at: [0; 14],
            last_action_us: 0,
            stable_since: None,
            engine_start_until: 0,
            gen5s_since: None,
            gen5min_since: None,
            ep_forced: false,
            bpcu_reset_shed: false,
            hpx_active: false,
            hpx_groups: 0,
            hpx_last_us: None,
            hpx_clear_since: None,
            pecs_shed: 0,
            pecs_last_us: 0,
            ep_slow_since: None,
            ep_fast_since: None,
            emp_stopped: false,
        }
    }

    pub fn power_up_reset(&mut self, now_us: u64) {
        for (k, d) in DEFAULT_LOADS.iter().enumerate() {
            if d.sheddable {
                self.shed[k] = true;
                self.shed_at[k] = now_us;
            }
        }
        self.bpcu_reset_shed = true;
        self.stable_since = None;
    }

    pub fn priority(k: usize, on_ground: bool) -> u8 {
        if on_ground {
            DEFAULT_LOADS[k].ground_priority
        } else {
            DEFAULT_LOADS[k].flight_priority
        }
    }

    fn demand_kw(&self, inp: &LmInput) -> f64 {
        (0..14)
            .filter(|&k| inp.demand_on[k] && !self.shed[k])
            .map(|k| DEFAULT_LOADS[k].kw)
            .sum()
    }

    fn lowest_priority_unshed(&self, inp: &LmInput) -> Option<usize> {
        (0..14)
            .filter(|&k| inp.demand_on[k] && !self.shed[k] && DEFAULT_LOADS[k].sheddable)
            .max_by_key(|&k| (Self::priority(k, inp.on_ground), k))
    }

    fn highest_priority_shed(&self, inp: &LmInput) -> Option<usize> {
        (0..14)
            .filter(|&k| self.shed[k])
            .min_by_key(|&k| (Self::priority(k, inp.on_ground), k))
    }

    fn hpx_held(&self, k: usize) -> bool {
        self.hpx_groups > 0
            && HPX_GROUPS[..self.hpx_groups.min(HPX_GROUPS.len())].contains(&DEFAULT_LOADS[k].group)
    }

    fn step_hpx(&mut self, inp: &LmInput) {
        let now = inp.now_us;
        let exceed = (0..2).any(|e| inp.hpx_limit_kw[e] > 0.0 && inp.hpx_kw[e] > inp.hpx_limit_kw[e]);
        if exceed && !self.hpx_active {
            let repeat = self
                .hpx_last_us
                .is_some_and(|t| now.saturating_sub(t) < HPX_REPEAT_WINDOW_US);
            self.hpx_groups = if repeat {
                (self.hpx_groups.max(1) + 1).min(HPX_GROUPS.len())
            } else {
                1
            };
            self.hpx_last_us = Some(now);
            for k in 0..DEFAULT_LOADS.len() {
                if self.hpx_held(k) && DEFAULT_LOADS[k].sheddable && !self.shed[k] {
                    self.shed[k] = true;
                    self.shed_at[k] = now;
                }
            }
        }
        self.hpx_active = exceed;
        if exceed {
            self.hpx_clear_since = None;
        } else if self.hpx_groups > 0 {
            let since = *self.hpx_clear_since.get_or_insert(now);
            if now.saturating_sub(since) >= APPROX_HPX_RESTORE_US {
                self.hpx_groups = 0;
                self.hpx_clear_since = None;
            }
        }
    }

    fn step_pecs(&mut self, inp: &LmInput) {
        let now = inp.now_us;
        let due = now.saturating_sub(self.pecs_last_us) >= RESTORE_DELAY_US;
        if inp.pecs_coolant_c > PECS_THERMAL_SHED_C {
            if self.pecs_shed < PECS_SHED_ORDER.len() && (self.pecs_shed == 0 || due) {
                let k = PECS_SHED_ORDER[self.pecs_shed];
                self.shed[k] = true;
                self.shed_at[k] = now;
                self.pecs_shed += 1;
                self.pecs_last_us = now;
            }
        } else if inp.pecs_coolant_c < APPROX_PECS_RESTORE_C && self.pecs_shed > 0 && due {
            self.pecs_shed -= 1;
            self.pecs_last_us = now;
        }
    }

    fn step_ep_cmsc(&mut self, inp: &LmInput) {
        let now = inp.now_us;
        if !inp.ext_in_use {
            self.emp_stopped = false;
            self.ep_slow_since = None;
            self.ep_fast_since = None;
            return;
        }
        let (slow_a, fast_a) = if inp.ext_single_90 {
            (EP_90_CMSC_STOP_A, EP_90_CMSC_STOP_FAST_A)
        } else {
            (EP_CMSC_STOP_A, EP_CMSC_STOP_FAST_A)
        };
        let slow = if inp.ext_phase_a > slow_a {
            now.saturating_sub(*self.ep_slow_since.get_or_insert(now)) >= EP_CMSC_STOP_US
        } else {
            self.ep_slow_since = None;
            false
        };
        let fast = if inp.ext_phase_a > fast_a {
            now.saturating_sub(*self.ep_fast_since.get_or_insert(now)) >= EP_CMSC_STOP_FAST_US
        } else {
            self.ep_fast_since = None;
            false
        };
        if slow || fast {
            self.emp_stopped = true;
            if !self.shed[HYD_LOAD_INDEX] {
                self.shed[HYD_LOAD_INDEX] = true;
                self.shed_at[HYD_LOAD_INDEX] = now;
            }
        }
    }

    fn held(&self, k: usize) -> bool {
        self.hpx_held(k)
            || PECS_SHED_ORDER[..self.pecs_shed].contains(&k)
            || (self.emp_stopped && k == HYD_LOAD_INDEX)
    }

    pub fn step(&mut self, inp: &LmInput) -> LmOutput {
        let now = inp.now_us;
        self.step_hpx(inp);
        self.step_pecs(inp);
        self.step_ep_cmsc(inp);
        let hpx_hold: [bool; 14] = core::array::from_fn(|k| self.held(k));
        if inp.engine_start {
            self.engine_start_until = now + ENGINE_START_RESTORE_HOLD_US;
            for (k, d) in DEFAULT_LOADS.iter().enumerate() {
                if d.shed_on_engine_start && !self.shed[k] {
                    self.shed[k] = true;
                    self.shed_at[k] = now;
                }
            }
        }
        let ep5s = inp.ext_in_use && inp.ext_kva > EP_5S_KVA;
        let ep5m = inp.ext_in_use && inp.ext_kva > EP_5MIN_KVA;
        let ov5s = ep5s;
        let ov5m = ep5m;
        if !ep5m {
            self.ep_forced = false;
        }
        let ep_due = !self.ep_forced || now.saturating_sub(self.last_action_us) >= EP_SHED_INTERVAL_US;
        let ext_trip = (ov5s || ov5m) && ep_due;
        if inp.source_change {
            self.stable_since = None;
        }
        let capacity = inp.available_kw;
        let raw = self.demand_kw(inp);
        let cac_kw = if inp.demand_on[CAC_LOAD_INDEX] && !self.shed[CAC_LOAD_INDEX] {
            DEFAULT_LOADS[CAC_LOAD_INDEX].kw
        } else {
            0.0
        };
        let cac_speed = if !inp.ext_in_use && raw > capacity && cac_kw > 0.0 {
            ((capacity - (raw - cac_kw)) / cac_kw).clamp(APPROX_CAC_MIN_SPEED, 1.0)
        } else {
            1.0
        };
        let demand = raw - cac_kw * (1.0 - cac_speed);
        let spaced = now.saturating_sub(self.last_action_us) >= SHED_SPACING_US;
        let over5m = !inp.ext_in_use && demand > capacity;
        let over5s = !inp.ext_in_use && demand > capacity * GEN_5MIN_RATIO;
        let beyond = !inp.ext_in_use && demand > capacity * GEN_5S_RATIO;
        let g5m = if over5m {
            *self.gen5min_since.get_or_insert(now)
        } else {
            self.gen5min_since = None;
            now
        };
        let g5s = if over5s {
            *self.gen5s_since.get_or_insert(now)
        } else {
            self.gen5s_since = None;
            now
        };
        let gen_trip = beyond
            || (over5m && now.saturating_sub(g5m) >= GEN_5MIN_US)
            || (over5s && now.saturating_sub(g5s) >= GEN_5S_US);
        let overload = gen_trip || inp.overload_hint_kw > 0.0 || ext_trip;
        if overload {
            self.stable_since = None;
            if spaced || ext_trip {
                if let Some(k) = self.lowest_priority_unshed(inp) {
                    self.shed[k] = true;
                    self.shed_at[k] = now;
                    self.last_action_us = now;
                    if ext_trip {
                        self.ep_forced = true;
                    }
                }
            }
        } else if now >= self.engine_start_until {
            let since = *self.stable_since.get_or_insert(now);
            if now.saturating_sub(since) >= RESTORE_DELAY_US && spaced {
                if let Some(k) = self.highest_priority_shed(inp).filter(|k| !hpx_hold[*k]) {
                    let waited = now.saturating_sub(self.shed_at[k]) >= MIN_SHED_US;
                    let kw = if inp.demand_on[k] {
                        DEFAULT_LOADS[k].kw
                    } else {
                        0.0
                    };
                    if waited && demand + kw + RESTORE_MARGIN_KW <= capacity {
                        self.shed[k] = false;
                        self.last_action_us = now;
                        self.bpcu_reset_shed = false;
                    }
                }
            }
        }
        let mut shed = [false; 14];
        let mut ov = [false; 14];
        let mut closed = [false; 14];
        let mut count = 0;
        for k in 0..14 {
            shed[k] = self.shed[k];
            ov[k] = self.shed[k] && inp.demand_on[k];
            closed[k] = inp.demand_on[k];
            if self.shed[k] {
                count += 1;
            }
        }
        LmOutput {
            shed,
            override_status: ov,
            breaker_closed: closed,
            total_kw: demand,
            shed_count: count,
            ext_overload_5s: ov5s,
            ext_overload_5min: ov5m,
            cac_speed,
            hpx_groups_shed: self.hpx_groups,
            pecs_thermal_shed: self.pecs_shed,
            emp_cmsc_stopped: self.emp_stopped,
        }
    }
}
