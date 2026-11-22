use super::Electrical;

const POR_BUSES: [&str; 6] = ["L1", "L2", "R1", "R2", "LTB", "RTB"];
const RAT_POR_BUS: &str = "BB";
const RAT_MACHINE: usize = 6;

impl Electrical {
    pub(crate) fn por_voltage(&self, m: usize) -> f64 {
        let (terminal, _) = self.plant.machine_sense(m);
        let (bus, breaker) = if m < 4 {
            (POR_BUSES[m], super::gcb_name(m))
        } else if m < 6 {
            (POR_BUSES[m], super::apb_name(m - 4))
        } else {
            (RAT_POR_BUS, "RCB".to_string())
        };
        if self.gear_out(&breaker).contacts_closed {
            let v = self.plant.bus_v(bus);
            if v > 0.0 {
                return v;
            }
        }
        terminal
    }

    pub(crate) fn drive_machine_exciter(&mut self, m: usize) {
        let lost = if m < 4 {
            self.faults.gen_loss[m]
        } else {
            self.faults.asg_loss[m - 4]
        };
        let g = &self.gcu_out[m];
        let driven = if self.fault_link.field_driver_short[m] {
            self.gcus[m].params.exciter_max_a
        } else {
            g.exciter_current_a
        };
        let exciter_a = if lost || g.field_off {
            0.0
        } else {
            driven * self.fault_link.exciter_scale[m]
        };
        self.exciter_feedback_a[m] = if lost || g.field_off { 0.0 } else { driven };
        self.plant.set_machine_exciter(m, exciter_a);
    }

    pub(crate) fn drive_rat_exciter(&mut self, exciter_a: f64, field_on: bool) {
        self.plant
            .set_machine_exciter(RAT_MACHINE, if field_on { exciter_a } else { 0.0 });
    }
}

#[cfg(test)]
mod tests {
    use crate::net::ElectricalNet;
    use crate::sources::gcu::UV_FAST_V;
    use crate::sources::{Gcu, GcuInput, GcuParams, MachineInput, MachineParams, SyncMachine};

    const STEP_AT_S: f64 = 4.0;
    const END_S: f64 = 9.0;
    const BAND_HI_V: f64 = 3.0;
    const BAND_LO_V: f64 = 2.0;
    const LOAD_PF_ANGLE_RAD: f64 = 0.451;

    struct Trace {
        pre_v: f64,
        post_min_v: f64,
        final_v: f64,
        settle_s: f64,
        max_v: f64,
        start_max_v: f64,
        below_uv_s: f64,
    }

    fn run(params: MachineParams, gcu: GcuParams, rpm: f64, load_a: f64, dt_us: u64) -> Trace {
        let mut m = SyncMachine::new(params, 60.0);
        let mut g = Gcu::new(gcu);
        let dt = dt_us as f64 * 1.0e-6;
        let steps = (END_S / dt) as usize;
        let nominal = gcu.regulated_v;
        let mut exc = 0.0;
        let mut v = 0.0;
        let mut pre_v = 0.0;
        let mut post_min_v = f64::MAX;
        let mut max_v = 0.0_f64;
        let mut start_max_v = 0.0_f64;
        let mut below_uv_s = 0.0_f64;
        let mut last_out_of_band = STEP_AT_S;
        for n in 0..steps {
            let t = n as f64 * dt;
            let i = if t >= STEP_AT_S { load_a } else { 0.0 };
            let mo = m.step(&MachineInput {
                dt_us,
                shaft_rpm: rpm,
                exciter_current_cmd_a: exc,
                load_current_a: i,
                load_pf_angle_rad: LOAD_PF_ANGLE_RAD,
                oil_inlet_c: 60.0,
                ..MachineInput::default()
            });
            v = mo.terminal_phase_v;
            let go = g.step(&GcuInput {
                dt_us,
                pmg_available: mo.gcu_power_available,
                pmg_freq_hz: mo.pmg_freq_hz,
                gcs_on: true,
                phase_v: [v; 3],
                gen_i: [i; 3],
                feeder_i: [i; 3],
                gcb_closed: true,
                idle_valid: true,
                phase_seq_abc: true,
                ..GcuInput::default()
            });
            exc = go.exciter_current_a;
            max_v = max_v.max(v);
            if t < STEP_AT_S {
                pre_v = v;
                start_max_v = start_max_v.max(v);
            } else {
                post_min_v = post_min_v.min(v);
                if v < UV_FAST_V {
                    below_uv_s += dt;
                }
                if v > nominal + BAND_HI_V || v < nominal - BAND_LO_V {
                    last_out_of_band = t;
                }
            }
        }
        Trace {
            pre_v,
            post_min_v,
            final_v: v,
            settle_s: last_out_of_band - STEP_AT_S,
            max_v,
            start_max_v,
            below_uv_s,
        }
    }

    fn cases() -> [(MachineParams, GcuParams, f64); 6] {
        [
            (MachineParams::vfsg(), GcuParams::vfsg(), 7300.0),
            (MachineParams::vfsg(), GcuParams::vfsg(), 15900.0),
            (MachineParams::asg(), GcuParams::asg(), 10_950.0),
            (MachineParams::asg(), GcuParams::asg(), 13_200.0),
            (MachineParams::rat(), GcuParams::rat(), 7600.0),
            (MachineParams::rat(), GcuParams::rat(), 9500.0),
        ]
    }

    #[test]
    fn every_machine_regulates_in_band_over_speed_with_rated_load_step() {
        for (p, g, rpm) in cases() {
            let rated = p.rated_phase_current_a();
            for dt in [1_000u64, 10_000, 50_000] {
                let t = run(p, g, rpm, rated, dt);
                let tag = format!("{:?} rpm {rpm} dt {dt}", p.kind);
                let nominal = g.regulated_v;
                assert!((t.pre_v - nominal).abs() < 0.5, "{tag} pre {}", t.pre_v);
                assert!(
                    t.start_max_v < nominal + BAND_HI_V,
                    "{tag} start overshoot {}",
                    t.start_max_v
                );
                assert!((t.final_v - nominal).abs() < 0.5, "{tag} final {}", t.final_v);
                assert!(t.settle_s < 1.0, "{tag} settle {}", t.settle_s);
                assert!(t.post_min_v > 150.0, "{tag} dip {}", t.post_min_v);
                assert!(t.below_uv_s < 0.3, "{tag} uv {}", t.below_uv_s);
                assert!(t.max_v < nominal + 8.0, "{tag} max {}", t.max_v);
            }
        }
    }

    #[test]
    fn regulation_is_bit_deterministic() {
        let (p, g, rpm) = cases()[1];
        let a = run(p, g, rpm, p.rated_phase_current_a(), 1_000);
        let b = run(p, g, rpm, p.rated_phase_current_a(), 1_000);
        assert_eq!(a.final_v.to_bits(), b.final_v.to_bits());
        assert_eq!(a.post_min_v.to_bits(), b.post_min_v.to_bits());
        assert_eq!(a.max_v.to_bits(), b.max_v.to_bits());
    }

    #[test]
    fn rat_stays_in_its_documented_band() {
        for (p, g, rpm) in [cases()[4], cases()[5]] {
            let t = run(p, g, rpm, p.rated_phase_current_a(), 10_000);
            assert!((t.pre_v - 230.0).abs() < crate::sources::gcu::RAT_REGULATION_BAND_V);
            assert!((t.final_v - 230.0).abs() < crate::sources::gcu::RAT_REGULATION_BAND_V);
        }
    }

    fn data_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/24-electrical")
    }

    struct Rig {
        net: ElectricalNet,
        gcu: Gcu,
        last_v: f64,
    }

    impl Rig {
        fn new() -> Self {
            let mut net = ElectricalNet::from_data(&data_dir()).unwrap();
            net.set_machine_rpm("ASG_L", 10_950.0);
            for c in [
                "L APB",
                "L2 BTB",
                "L3 BTB",
                "L TRU RLY",
                "L1 ATRUC",
                "L2 ATRUC",
                "L1 BTB",
            ] {
                net.set_contactor(c, true);
            }
            Self {
                net,
                gcu: Gcu::new(GcuParams::asg()),
                last_v: 0.0,
            }
        }

        fn run(&mut self, secs: f64) -> crate::net::Snapshot {
            let dt_us = 10_000;
            let mut snap = self.net.step(dt_us).unwrap();
            for _ in 0..(secs * 100.0) as usize {
                let mo = snap.machine("ASG_L").unwrap();
                self.last_v = mo.terminal_v;
                let out = self.gcu.step(&GcuInput {
                    dt_us,
                    pmg_available: true,
                    pmg_freq_hz: 13.5 * 7300.0 / 60.0,
                    gcs_on: true,
                    phase_v: [mo.terminal_v; 3],
                    gen_i: [mo.current_a; 3],
                    feeder_i: [mo.current_a; 3],
                    gcb_closed: true,
                    idle_valid: true,
                    phase_seq_abc: true,
                    ..GcuInput::default()
                });
                self.net.set_machine_exciter("ASG_L", out.exciter_current_a);
                snap = self.net.step(dt_us).unwrap();
            }
            snap
        }
    }

    #[test]
    fn atru_outputs_hold_documented_levels_at_no_load_and_rated_load() {
        let mut r = Rig::new();
        let s = r.run(5.0);
        let h = s.bus("HVDC_L1").unwrap();
        assert!(h.live);
        assert!((250.0..=320.0).contains(&h.voltage_v), "{}", h.voltage_v);
        let ac = s.bus("L1").unwrap().voltage_v;
        let no_load = h.voltage_v;
        assert!((ac - 235.0).abs() < 2.0, "ac {ac}");
        assert!((no_load - 280.0 * ac / 235.0).abs() < 2.0, "no load {no_load}");
        assert!(r.net.set_load("CMSC_CAC_L1", 150_000.0));
        let s = r.run(3.0);
        let h = s.bus("HVDC_L1").unwrap();
        let ac = s.bus("L1").unwrap().voltage_v;
        assert!((ac - 235.0).abs() < 4.0, "ac loaded {ac}");
        let sag = no_load - h.voltage_v;
        assert!((4.0..14.0).contains(&sag), "sag {sag}");
        assert!(
            (h.voltage_v - 270.0 * ac / 235.0).abs() < 4.0,
            "rated {}",
            h.voltage_v
        );
        assert!(s.load("CMSC_CAC_L1").unwrap().current_a > 400.0);
    }

    #[test]
    fn tru_outputs_hold_28v_at_no_load_and_rated_load() {
        let mut r = Rig::new();
        let s = r.run(5.0);
        let dc = s.bus("DC28_L").unwrap().voltage_v;
        assert!((27.0..=29.5).contains(&dc), "base load {dc}");
        assert!(r.net.set_load("LOAD_DC28_L", 2_000.0));
        let s = r.run(2.0);
        let dc = s.bus("DC28_L").unwrap().voltage_v;
        assert!((26.5..=29.0).contains(&dc), "rated {dc}");
    }

    #[test]
    fn engine_driven_buses_hold_band_and_battery_floats_off_main_dc() {
        use crate::system::{Electrical, ElectricalInputs};
        for rpm in [7550.0, 15900.0] {
            let mut e = Electrical::new(&data_dir()).unwrap();
            let idle = ElectricalInputs::default();
            e.step(1_000_000, &idle);
            e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
            e.step(2_000_000, &idle);
            for i in 0..4 {
                e.panel_mut().gen_ctrl[i].set_latched(true);
            }
            let run = ElectricalInputs {
                engine_rpm: [rpm; 2],
                engine_running: [true; 2],
                on_ground: false,
                radio_alt_ft: 20000.0,
                ..ElectricalInputs::default()
            };
            e.step(10_000_000, &run);
            for bus in ["L1", "L2", "R1", "R2"] {
                let v = e.bus_volts(bus);
                assert!((233.0..=238.0).contains(&v), "{rpm} {bus} {v}");
            }
            for bus in ["HVDC_L1", "HVDC_R2"] {
                let v = e.bus_volts(bus);
                assert!((265.0..=290.0).contains(&v), "{rpm} {bus} {v}");
            }
            for bus in ["DC28_L", "DC28_R", "CIB", "FIB", "HBB_MAIN"] {
                let v = e.bus_volts(bus);
                assert!((27.0..=29.5).contains(&v), "{rpm} {bus} {v}");
            }
            assert!(e.plant.battery_a(0).abs() < 1.0, "{rpm} {}", e.plant.battery_a(0));
        }
    }
}
