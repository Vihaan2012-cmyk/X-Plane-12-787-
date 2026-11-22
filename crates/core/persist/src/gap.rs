use crate::model::{Airframe, BatteryRecord, GapMode, PartRecord, SessionClock};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GapReport {
    pub mode: GapMode,
    pub elapsed_s: f64,
    pub clamped: bool,
    pub first_session: bool,
}

pub fn soak_toward(temp_c: f64, ambient_c: f64, tau_s: f64, dt_s: f64) -> f64 {
    if dt_s <= 0.0 {
        return temp_c;
    }
    if tau_s <= 0.0 {
        return ambient_c;
    }
    ambient_c + (temp_c - ambient_c) * (-dt_s / tau_s).exp()
}

pub fn apply_part_gap(p: &mut PartRecord, elapsed_s: f64, ambient_c: f64) {
    p.unpowered_s += elapsed_s;
    p.soak_c = soak_toward(p.soak_c, ambient_c, p.soak_tau_s, elapsed_s);
    for c in p.counters.values_mut() {
        if !c.powered_only {
            c.value += elapsed_s;
        }
    }
}

pub fn apply_battery_gap(b: &mut BatteryRecord, elapsed_s: f64, ambient_c: f64) {
    if elapsed_s <= 0.0 {
        return;
    }
    let steps = (elapsed_s / 3600.0).ceil().clamp(1.0, 4096.0);
    let dt = elapsed_s / steps;
    let n = steps as u64;
    for _ in 0..n {
        b.soak_c = soak_toward(b.soak_c, ambient_c, b.soak_tau_s, dt);
        let k = b.self_discharge_per_day * ((b.soak_c - 20.0) / 10.0).exp2();
        let f = (-k * dt / 86400.0).exp();
        b.soc = (b.soc * f).clamp(0.0, 1.0);
        for c in b.cell_soc.iter_mut() {
            *c = (*c * f).clamp(0.0, 1.0);
        }
    }
}

impl Airframe {
    pub fn elapsed_since_stamp(&self, now: SessionClock) -> (f64, bool) {
        let Some(stamp) = self.stamp else {
            return (0.0, false);
        };
        let raw = match self.settings.gap_mode {
            GapMode::Sim => now.sim_s - stamp.sim_s,
            GapMode::Real => now.real_s.saturating_sub(stamp.real_s) as f64,
        };
        if !raw.is_finite() || raw < 0.0 {
            return (0.0, true);
        }
        if raw > self.settings.max_gap_s {
            return (self.settings.max_gap_s, true);
        }
        (raw, false)
    }

    pub fn begin_session(&mut self, now: SessionClock, ambient_c: f64) -> GapReport {
        let first_session = self.stamp.is_none();
        let (elapsed_s, clamped) = self.elapsed_since_stamp(now);
        if elapsed_s > 0.0 {
            for p in self.parts.values_mut() {
                apply_part_gap(p, elapsed_s, ambient_c);
            }
            for b in self.batteries.values_mut() {
                apply_battery_gap(b, elapsed_s, ambient_c);
            }
        }
        self.stamp = Some(now);
        GapReport {
            mode: self.settings.gap_mode,
            elapsed_s,
            clamped,
            first_session,
        }
    }

    pub fn end_session(&mut self, now: SessionClock) {
        self.stamp = Some(now);
    }
}
