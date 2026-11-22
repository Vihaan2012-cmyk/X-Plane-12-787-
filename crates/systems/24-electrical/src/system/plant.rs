use super::inputs::ExtSupply;
use crate::net::{ElectricalNet, Snapshot};
use crate::sources::MachineParams;
use crate::storage::MAX_CELLS;

pub(crate) const MACHINE_NAMES: [&str; 7] = [
    "VFSG_L1", "VFSG_L2", "VFSG_R1", "VFSG_R2", "ASG_L", "ASG_R", "RAT",
];
const RECEPTACLES: [&str; 3] = ["L FWD", "R FWD", "AFT"];
const BATTERY_NAMES: [&str; 2] = ["BATT_MAIN", "BATT_APU"];

pub(crate) struct Plant {
    net: ElectricalNet,
    snap: Option<Snapshot>,
    machine_rpm: [f64; 7],
    machine_params: Vec<MachineParams>,
    ext_connected: [bool; 3],
}

impl Plant {
    pub(crate) fn new(data_dir: &std::path::Path) -> Result<Self, String> {
        Ok(Self {
            net: ElectricalNet::from_data(data_dir)?,
            snap: None,
            machine_rpm: [0.0; 7],
            machine_params: vec![
                MachineParams::vfsg(),
                MachineParams::vfsg(),
                MachineParams::vfsg(),
                MachineParams::vfsg(),
                MachineParams::asg(),
                MachineParams::asg(),
                MachineParams::rat(),
            ],
            ext_connected: [false; 3],
        })
    }

    pub(crate) fn snapshot(&self) -> Option<&Snapshot> {
        self.snap.as_ref()
    }

    pub(crate) fn net(&self) -> &ElectricalNet {
        &self.net
    }

    pub(crate) fn net_mut(&mut self) -> &mut ElectricalNet {
        &mut self.net
    }

    pub(crate) fn bus_v(&self, name: &str) -> f64 {
        self.snap
            .as_ref()
            .and_then(|s| s.bus(name))
            .map(|b| b.voltage_v)
            .unwrap_or(0.0)
    }

    pub(crate) fn contactor_a(&self, name: &str) -> f64 {
        self.snap
            .as_ref()
            .and_then(|s| s.contactor(name))
            .map(|c| c.current_a)
            .unwrap_or(0.0)
    }

    pub(crate) fn machine_sense(&self, m: usize) -> (f64, f64) {
        self.snap
            .as_ref()
            .and_then(|s| s.machine(MACHINE_NAMES[m]))
            .map(|r| (r.terminal_v, r.current_a))
            .unwrap_or((0.0, 0.0))
    }

    pub(crate) fn feeder_zone_residual_a(&self, m: usize, breaker: &str) -> f64 {
        let Some(s) = self.snap.as_ref() else {
            return 0.0;
        };
        let g = s
            .machine(MACHINE_NAMES[m])
            .map(|r| r.residual)
            .unwrap_or((0.0, 0.0));
        let b = s.contactor(breaker).map(|c| c.residual).unwrap_or((0.0, 0.0));
        let (re, im) = (g.0 - b.0, g.1 - b.1);
        (re * re + im * im).sqrt()
    }

    pub(crate) fn machine_phases(&self, m: usize) -> ([f64; 3], [f64; 3], f64) {
        self.snap
            .as_ref()
            .and_then(|s| s.machine(MACHINE_NAMES[m]))
            .map(|r| (r.phase_v, r.phase_a, r.residual_a))
            .unwrap_or(([0.0; 3], [0.0; 3], 0.0))
    }

    pub(crate) fn contactor_phase_a(&self, name: &str) -> [f64; 3] {
        self.snap
            .as_ref()
            .and_then(|s| s.contactor(name))
            .map(|c| c.phase_a)
            .unwrap_or([0.0; 3])
    }

    pub(crate) fn bus_hz(&self, name: &str) -> f64 {
        self.snap
            .as_ref()
            .and_then(|s| s.bus(name))
            .map(|b| b.freq_hz)
            .unwrap_or(0.0)
    }

    pub(crate) fn machine_hz(&self, m: usize) -> f64 {
        self.machine_params[m].electrical_freq_hz(self.machine_rpm[m])
    }

    pub(crate) fn battery_v(&self, b: usize) -> f64 {
        self.snap
            .as_ref()
            .and_then(|s| s.battery(BATTERY_NAMES[b]))
            .map(|r| r.voltage_v)
            .unwrap_or(0.0)
    }

    pub(crate) fn battery_a(&self, b: usize) -> f64 {
        self.snap
            .as_ref()
            .and_then(|s| s.battery(BATTERY_NAMES[b]))
            .map(|r| r.current_a)
            .unwrap_or(0.0)
    }

    pub(crate) fn battery_temp_c(&self, b: usize) -> f64 {
        self.snap
            .as_ref()
            .and_then(|s| s.battery(BATTERY_NAMES[b]))
            .map(|r| r.temp_k - 273.15)
            .unwrap_or(15.0)
    }

    fn pack(&self, b: usize) -> &crate::storage::BatteryPack {
        if b == 0 {
            self.net.battery()
        } else {
            self.net.battery_apu()
        }
    }

    pub(crate) fn battery_thevenin(&self, b: usize) -> (f64, f64) {
        self.pack(b).thevenin()
    }

    pub(crate) fn battery_cells(&self, b: usize, out: &mut [f64; MAX_CELLS]) -> usize {
        let pack = self.pack(b);
        let n = pack.params.cells;
        let i = self.battery_a(b);
        for (k, slot) in out.iter_mut().enumerate().take(n) {
            *slot = pack.cell_ocv(k) - i * pack.cell_r0(k);
        }
        n
    }

    pub(crate) fn set_contactor(&mut self, name: &str, closed: bool) {
        self.net.set_contactor(name, closed);
    }

    pub(crate) fn set_machine_rpm(&mut self, m: usize, rpm: f64) {
        self.machine_rpm[m] = rpm;
        self.net.set_machine_rpm(MACHINE_NAMES[m], rpm);
    }

    pub(crate) fn set_machine_exciter(&mut self, m: usize, exciter_a: f64) {
        self.net.set_machine_exciter(MACHINE_NAMES[m], exciter_a);
    }

    pub(crate) fn rat_volts(&self) -> f64 {
        self.machine_sense(6).0
    }

    pub(crate) fn set_ext(&mut self, k: usize, s: ExtSupply) {
        let ok = s.connected && s.v_ll > 80.0 && s.hz > 300.0;
        if self.ext_connected[k] != ok {
            self.ext_connected[k] = ok;
            self.net.set_external_power(RECEPTACLES[k], ok);
        }
    }

    pub(crate) fn set_load_shed(&mut self, _shed: &[bool; 14]) {}

    pub(crate) fn set_load(&mut self, name: &str, demand_w: f64) {
        self.net.set_load(name, demand_w);
    }

    pub(crate) fn set_sspc(&mut self, id: &str, closed: bool) -> bool {
        self.net.set_sspc(id, closed)
    }

    pub(crate) fn load_power(&self, name: &str) -> f64 {
        self.snap
            .as_ref()
            .and_then(|s| s.load(name))
            .map(|l| l.power_w)
            .unwrap_or(0.0)
    }

    pub(crate) fn set_charger(&mut self, b: usize, a: f64) {
        self.net.set_battery_charge(b, a);
    }

    pub(crate) fn step(&mut self, dt_us: u64, _ambient_c: f64) {
        if let Ok(s) = self.net.step(dt_us) {
            self.snap = Some(s);
        }
    }
}

impl Plant {
    pub(crate) fn analog_samples(&self, out: &mut Vec<f64>) {
        out.clear();
        if let Some(s) = &self.snap {
            for b in &s.buses {
                out.push(b.voltage_v);
            }
            for b in &s.batteries {
                out.push(b.voltage_v);
                out.push(b.current_a);
            }
            for m in &s.machines {
                out.push(m.terminal_v);
                out.push(m.current_a);
            }
        }
    }
}
