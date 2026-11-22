use super::{CrewCommand, Electrical, ElectricalInputs, ExtSupply, MACHINES, SUBSTEP_US};
use crate::control::cbic::{BreakerKind, CbAction, CbState, CmdOutcome, LockTag, UserId, UserKind};
use crate::crew::ApuSelector;
use crate::sources::GcuInput;

fn gen_index(target: &str) -> Option<usize> {
    ["L1", "L2", "R1", "R2"].iter().position(|n| target.contains(n))
}

fn side_index(target: &str) -> Option<usize> {
    if target.ends_with("_L") || target.ends_with(" L") {
        Some(0)
    } else if target.ends_with("_R") || target.ends_with(" R") {
        Some(1)
    } else {
        None
    }
}

fn parse_num(text: &str, key: &str) -> Option<f64> {
    let at = text.find(key)?;
    let rest = &text[at + key.len()..];
    let rest = rest.trim_start_matches([' ', '=', ':']);
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn ext_index(target: &str) -> Option<usize> {
    if target.starts_with("EXT_PWR_FWD_L") {
        Some(0)
    } else if target.starts_with("EXT_PWR_FWD_R") {
        Some(1)
    } else if target.starts_with("EXT_PWR_AFT") {
        Some(2)
    } else {
        None
    }
}

fn ext_preset_index(key: &str) -> Option<usize> {
    match key {
        "ext_pwr_fwd_l" => Some(0),
        "ext_pwr_fwd_r" => Some(1),
        "ext_pwr_aft" => Some(2),
        _ => None,
    }
}

fn clean(v: &str) -> &str {
    v.trim().trim_matches('"')
}

fn truthy(v: Option<&str>) -> bool {
    matches!(
        v.map(|s| clean(s).to_ascii_lowercase()).as_deref(),
        Some("true")
            | Some("on")
            | Some("1")
            | Some("yes")
            | Some("run")
            | Some("plugged")
            | Some("connected")
    )
}

fn parse_bool_pair(v: &str) -> [bool; 2] {
    let inner = v.trim().trim_start_matches('[').trim_end_matches(']');
    let mut it = inner.split(',').map(|s| s.trim() == "true");
    let a = it.next().unwrap_or(false);
    let b = it.next().unwrap_or(a);
    [a, b]
}

fn cbic_id(name: &str) -> Option<u32> {
    name.get(2..)?.parse().ok()
}

fn cbic_kind(name: &str) -> BreakerKind {
    match name.get(..2) {
        Some("CK") => BreakerKind::Elcc,
        Some("CT") => BreakerKind::Tcb,
        _ => BreakerKind::Ecb,
    }
}

fn gcu_target(path: &str) -> Option<usize> {
    let rest = path.strip_prefix("gcu.")?;
    ["l1", "l2", "r1", "r2"].iter().position(|n| rest.starts_with(n))
}

impl Electrical {
    pub(crate) fn apply_command(&mut self, c: CrewCommand) {
        match c {
            CrewCommand::Battery(on) => self.panel.battery.set_latched(on),
            CrewCommand::ApuGen(i, on) => self.panel.apu_gen[i.min(1)].set_latched(on),
            CrewCommand::GenCtrl(i, on) => {
                let i = i.min(3);
                if on && !self.panel.gen_ctrl[i].is_on() {
                    self.panel.press_gen_ctrl(i);
                } else {
                    self.panel.gen_ctrl[i].set_latched(on);
                }
            }
            CrewCommand::GenCtrlCycle(i) => {
                let i = i.min(3);
                self.panel.gen_ctrl[i].set_latched(false);
                self.panel.press_gen_ctrl(i);
            }
            CrewCommand::ExtPwrFwd(i) => {
                self.panel.ext_pwr_fwd[i.min(1)].press();
                self.panel.ext_pwr_fwd[i.min(1)].release();
            }
            CrewCommand::ExtPwrAft => {
                self.panel.ext_pwr_aft.press();
                self.panel.ext_pwr_aft.release();
            }
            CrewCommand::DriveDisc(i) => {
                self.press_drive_disconnect_for(i.min(3), super::drive_disc::APPROX_CREW_PRESS_US);
            }
            CrewCommand::GroundService => {
                self.panel.ground_service.press();
                self.panel.ground_service.release();
            }
            CrewCommand::ApuStart => {
                self.panel.set_apu_selector(ApuSelector::Start);
            }
            CrewCommand::CabinUtil(on) => self.panel.cabin_util.set_latched(on),
            CrewCommand::IfeSeats(on) => self.panel.ife_seats.set_latched(on),
            CrewCommand::LampTest(on) => self.crew_in.lamp_test = on,
        }
    }

    pub fn action(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        value: Option<&str>,
    ) -> Result<(), String> {
        let on = |v: Option<&str>| {
            !matches!(
                v.map(|s| clean(s).to_ascii_lowercase()).as_deref(),
                Some("off") | Some("0") | Some("false") | Some("unplugged") | Some("norm") | Some("cutoff")
            )
        };
        match actor {
            "crew" => self.crew_action(action, target, value, on(value)),
            "ground" => self.ground_action(action, target, value),
            "fault" => self.fault_action(action, target, value),
            "sim" => self.sim_action(action, target, value),
            _ => Err(format!("unknown actor {actor}")),
        }
    }

    fn ext_switch(&mut self, k: usize, value: Option<&str>, cycle: bool) {
        let pulse = |s: &mut Self| {
            if k == 2 {
                s.apply_command(CrewCommand::ExtPwrAft);
            } else {
                s.apply_command(CrewCommand::ExtPwrFwd(k));
            }
        };
        if cycle || value.is_some_and(|v| clean(v).eq_ignore_ascii_case("OFF_ON")) {
            self.ext_switch_on[k] = false;
            let held = self.inputs.clone();
            self.step(2 * SUBSTEP_US, &held);
            pulse(self);
            return;
        }
        match value {
            Some(v) => {
                let want = on_text(v);
                if want != self.ext_switch_on[k] {
                    pulse(self);
                }
            }
            None => pulse(self),
        }
    }

    fn crew_action(
        &mut self,
        action: &str,
        target: &str,
        value: Option<&str>,
        on: bool,
    ) -> Result<(), String> {
        if action.starts_with("cbic_") {
            return self.cbic_action(action, target);
        }
        if action == "open_page" {
            return self.open_page(target);
        }
        if action == "cycle_power" {
            return self.cycle_power(target);
        }
        let cmd = match target {
            "BATTERY" => {
                if action == "press" && value.is_none() {
                    CrewCommand::Battery(!self.panel.battery.is_on())
                } else {
                    CrewCommand::Battery(on)
                }
            }
            "GROUND_SERVICE" => CrewCommand::GroundService,
            "APU_SELECTOR" | "APU" | "APU_SWITCH" => {
                if value.is_some_and(|v| clean(v) == "START") || action == "start" {
                    CrewCommand::ApuStart
                } else {
                    self.panel
                        .set_apu_selector(if on { ApuSelector::On } else { ApuSelector::Off });
                    return Ok(());
                }
            }
            "CABIN_UTIL" => CrewCommand::CabinUtil(on),
            "IFE_SEATS" => CrewCommand::IfeSeats(on),
            t if t.starts_with("ENG_START") => {
                let i = side_index(t).ok_or("engine side")?;
                let want = value.is_some_and(|v| clean(v) == "START");
                self.crew_in.engine_start_in_progress[i] = want && !self.inputs.engine_running[i];
                if want && !self.inputs.engine_running[i] {
                    self.rig.engine[i].owned = true;
                    self.rig.engine[i].crank_s = 0.0;
                }
                return Ok(());
            }
            t if t.starts_with("ENG_MOTOR") => {
                let i = side_index(t).ok_or("engine side")?;
                self.rig.motor_request[i] = on;
                return Ok(());
            }
            t if t.starts_with("FUEL_CONTROL") => {
                let i = side_index(t).ok_or("engine side")?;
                let run = value.is_some_and(|v| clean(v).eq_ignore_ascii_case("RUN"));
                self.rig.engine[i].fuel = run;
                return Ok(());
            }
            t if t.starts_with("APU_GEN") => {
                let i = side_index(t).ok_or("apu gen side")?;
                if action == "press" && value.is_none() {
                    CrewCommand::ApuGen(i, !self.panel.apu_gen[i].is_on())
                } else {
                    CrewCommand::ApuGen(i, on)
                }
            }
            t if t.starts_with("GEN_CTRL") => {
                let i = gen_index(t).ok_or("gen ctrl index")?;
                if value.is_some_and(|v| clean(v) == "OFF_ON") || action == "cycle" {
                    CrewCommand::GenCtrlCycle(i)
                } else if action == "press" && value.is_none() {
                    CrewCommand::GenCtrl(i, !self.panel.gen_ctrl[i].is_on())
                } else {
                    CrewCommand::GenCtrl(i, on)
                }
            }
            t if t.starts_with("DRIVE_DISC") => CrewCommand::DriveDisc(gen_index(t).ok_or("drive index")?),
            t => match ext_index(t) {
                Some(k) => {
                    self.ext_switch(k, value, action == "cycle");
                    return Ok(());
                }
                None => return Err(format!("unknown crew target {t}")),
            },
        };
        self.apply_command(cmd);
        Ok(())
    }

    fn open_page(&mut self, target: &str) -> Result<(), String> {
        match target {
            "CBIC" => {
                self.cbic_session()?;
                Ok(())
            }
            t => Err(format!("unknown page {t}")),
        }
    }

    fn cbic_session(&mut self) -> Result<UserId, String> {
        if let Some(s) = self.inputs.cbic_session.take() {
            self.cbic.app.disconnect(UserId(s));
        }
        let user = self
            .cbic
            .app
            .connect(UserKind::Mfd)
            .map_err(|r| format!("cbic session refused {r:?}"))?;
        self.inputs.cbic_session = Some(user.0);
        Ok(user)
    }

    fn cbic_action(&mut self, action: &str, target: &str) -> Result<(), String> {
        let id = cbic_id(target).ok_or_else(|| format!("bad breaker {target}"))?;
        let kind = cbic_kind(target);
        if self.cbic.app.breaker(id).is_none() {
            self.cbic.app.add_breaker(id, target, kind, CbState::Closed);
        }
        let act = match action {
            "cbic_open" => CbAction::Open,
            "cbic_close" => CbAction::Close,
            "cbic_lock" => CbAction::Lock(LockTag::DoNotClose),
            "cbic_unlock" => CbAction::Unlock,
            "cbic_reset" => CbAction::RequestReset,
            a => return Err(format!("unknown cbic action {a}")),
        };
        let user = self.cbic_session()?;
        let mut out = self.cbic.app.command(user, id, act, true);
        if out == CmdOutcome::PendingEnter {
            out = self.cbic.app.press_enter(user);
        }
        if matches!(out, CmdOutcome::Refused(_)) && kind == BreakerKind::Tcb {
            self.faults.cbic_tcb_rejected = true;
        }
        let taken = self.cbic.app.take_commands();
        for c in self.pdu.queue(taken, &mut self.cbic.app) {
            let reported = match c.action {
                CbAction::Open => CbState::Open,
                CbAction::Close | CbAction::RequestReset => CbState::Closed,
                _ => continue,
            };
            self.cbic.app.report_status(c.id, reported, 0.0);
        }
        Ok(())
    }

    fn cycle_power(&mut self, target: &str) -> Result<(), String> {
        let range = match target {
            "ALL_GCU" => 0..6,
            t => {
                let i = gen_index(t).ok_or_else(|| format!("unknown power target {t}"))?;
                i..i + 1
            }
        };
        for m in range {
            self.gcus[m].step(&GcuInput::default());
            self.gcu_out[m] = Default::default();
        }
        Ok(())
    }

    fn ground_action(&mut self, action: &str, target: &str, value: Option<&str>) -> Result<(), String> {
        let (base, field) = match target.split_once('.') {
            Some((b, f)) => (b, Some(f)),
            None => (target, None),
        };
        let k = ext_index(base).ok_or_else(|| format!("unknown ground target {target}"))?;
        let mut e: ExtSupply = self.inputs.ext[k];
        match (action, field) {
            ("connect", _) => {
                e.connected = true;
                if let Some(v) = value {
                    e.v_ll = parse_num(v, "v_ll").unwrap_or(e.v_ll);
                    e.hz = parse_num(v, "hz").unwrap_or(e.hz);
                    e.kva = parse_num(v, "kva").unwrap_or(e.kva);
                }
            }
            ("set", Some(f)) => {
                let n: f64 = value
                    .and_then(|v| clean(v).parse().ok())
                    .ok_or("numeric value required")?;
                match f {
                    "hz" => e.hz = n,
                    "v_ll" | "v" | "v_worst_phase_rms" => e.v_ll = n,
                    "kva" => e.kva = n,
                    _ => return Err(format!("unknown ext field {f}")),
                }
            }
            ("disconnect", _) | ("unplug", _) => e.connected = false,
            _ => return Err(format!("unknown ground action {action}")),
        }
        self.inputs.ext[k] = e;
        Ok(())
    }

    fn fault_action(&mut self, action: &str, target: &str, value: Option<&str>) -> Result<(), String> {
        match action {
            "arm" => {
                let spec = value.map(clean).ok_or("mode required")?;
                let mut it = spec.split_whitespace();
                let mode = it.next().ok_or("mode required")?;
                let magnitude = match it.next() {
                    Some(m) => m.parse::<f64>().map_err(|_| format!("bad magnitude {m}"))?,
                    None => 1.0,
                };
                return self.arm_fault(clean(target), mode, magnitude);
            }
            "clear" => {
                let mode = value.map(clean).filter(|m| !m.is_empty());
                return self.clear_fault(clean(target), mode);
            }
            "gen_loss" | "vfsg_fail" => {
                if target.starts_with("ALL") {
                    self.faults.gen_loss = [true; 4];
                    for m in MACHINES.iter().take(4) {
                        let _ = self.arm_fault(m, "pmg-failure", 1.0);
                    }
                } else {
                    let i = gen_index(target).ok_or("generator index")?;
                    self.faults.gen_loss[i] = true;
                    let _ = self.arm_fault(MACHINES[i], "pmg-failure", 1.0);
                    if value.is_some_and(|v| clean(v) == "drive_disconnect") {
                        self.drive_disconnected[i] = true;
                        self.crew_in.vfsg_lop[i] = true;
                    }
                }
            }
            "asg_fail" => {
                let i = side_index(target).unwrap_or(0);
                self.faults.asg_loss[i] = true;
                let _ = self.arm_fault(MACHINES[4 + i], "pmg-failure", 1.0);
            }
            "bpcu_fail" => {
                let left = target.contains('L') || target == "BOTH";
                let right = target.contains('R') || target == "BOTH";
                if left {
                    self.faults.bpcu_fail[0] = true;
                    let _ = self.arm_fault("BPCU_L", "cpu-fail", 1.0);
                }
                if right {
                    self.faults.bpcu_fail[1] = true;
                    let _ = self.arm_fault("BPCU_R", "cpu-fail", 1.0);
                }
                if !left && !right {
                    return Err(format!("unknown bpcu {target}"));
                }
            }
            "bus_fail" => {
                let i = gen_index(target).ok_or("bus index")?;
                self.faults.bus_fail[i] = true;
                let bus = super::BUS_NAMES[i];
                for g in self.gear.iter_mut() {
                    if g.name.starts_with(bus) || g.upstream == bus {
                        g.forced_open = true;
                    }
                }
            }
            "all_engines_flameout" => {
                self.rig_set_engine_running(0, false);
                self.rig_set_engine_running(1, false);
            }
            _ => return Err(format!("unknown fault {action}")),
        }
        Ok(())
    }

    fn sim_action(&mut self, action: &str, target: &str, value: Option<&str>) -> Result<(), String> {
        let n: Option<f64> = value.and_then(|v| clean(v).parse().ok());
        match (action, target) {
            ("advance", "clock_s") => {
                let s = n.ok_or("seconds required")?;
                let inputs: ElectricalInputs = self.inputs.clone();
                self.step((s * 1.0e6) as u64, &inputs);
                Ok(())
            }
            ("set", _) => self.set_state(target, value, n),
            _ => Err(format!("unknown sim action {action}")),
        }
    }

    fn set_state(&mut self, key: &str, value: Option<&str>, n: Option<f64>) -> Result<(), String> {
        let text = value.map(clean).unwrap_or("");
        match key {
            "on_ground" => self.inputs.on_ground = truthy(value),
            "oat_c" => self.inputs.ambient_c = n.ok_or("number required")?,
            "alt_ft" => {
                let a = n.ok_or("number required")?;
                self.inputs.alt_ft = a;
                self.inputs.radio_alt_ft = a;
            }
            "airplane_state" => match text {
                "cruise" => {
                    self.inputs.on_ground = false;
                    self.inputs.ground_speed_kt = 480.0;
                    self.inputs.galley_power = true;
                    self.inputs.cargo_heat = true;
                }
                "cold_dark" | "powered_continuously" => {
                    self.inputs.on_ground = true;
                    self.inputs.ground_speed_kt = 0.0;
                }
                s => return Err(format!("unsupported airplane state {s}")),
            },
            "battery_switch" => self.apply_command(CrewCommand::Battery(truthy(value))),
            "apu_battery_connected" => self.inputs.apu_battery_connected = truthy(value),
            "apu_gens_online" => {
                let on = truthy(value);
                self.apply_command(CrewCommand::ApuGen(0, on));
                self.apply_command(CrewCommand::ApuGen(1, on));
            }
            "apu_running" => {
                let run = truthy(value);
                self.rig_set_apu_running(run);
            }
            "engines_running" => {
                let pair = parse_bool_pair(text);
                for i in 0..2 {
                    self.rig_set_engine_running(i, pair[i]);
                }
            }
            "gen_ctrl_switches" => {
                let on = truthy(value);
                for i in 0..4 {
                    self.apply_command(CrewCommand::GenCtrl(i, on));
                }
            }
            "ext_power_on" => {
                if truthy(value) {
                    self.inputs.ext[0] = ExtSupply::cart();
                    self.ext_switch_on[0] = true;
                }
            }
            "gcu_power_cycled" => {
                if truthy(value) {
                    self.cycle_power("ALL_GCU")?;
                }
            }
            "rat_deployed" => self.inputs.rat_deployed = truthy(value),
            k if ext_preset_index(k).is_some() => {
                let i = ext_preset_index(k).unwrap_or(0);
                if truthy(value) {
                    self.inputs.ext[i] = ExtSupply::cart();
                } else if !(i == 0 && self.ext_switch_on[0]) {
                    self.inputs.ext[i].connected = false;
                }
            }
            k if k.starts_with("gcu.") && k.ends_with(".uptime_days") => {
                let m = gcu_target(k).ok_or_else(|| format!("unknown gcu {k}"))?;
                let days = n.ok_or("days required")?;
                self.gcus[m].set_powered_time_us((days * 86_400.0e6) as u64);
            }
            k if k.starts_with("load.total_kva_on_ext_pwr_") => {
                let which = k.trim_start_matches("load.total_kva_on_").to_ascii_uppercase();
                let i = ext_index(&which).ok_or_else(|| format!("unknown ext load {k}"))?;
                self.inputs.ext_load_kva[i] = n.ok_or("kva required")?;
            }
            t => return Err(format!("unsupported state path {t}")),
        }
        Ok(())
    }
}

fn on_text(v: &str) -> bool {
    !matches!(
        clean(v).to_ascii_lowercase().as_str(),
        "off" | "0" | "false" | "unplugged"
    )
}
