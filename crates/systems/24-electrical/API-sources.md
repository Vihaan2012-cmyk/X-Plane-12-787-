# API: sources and storage (package b789-sys-electrical, crate sys_electrical)

All plain f64 SI, time in u64 microseconds. No comments, no unsafe, deterministic.

## sources::machine
- `MachineKind {Vfsg, Asg, RatGen}`, `MachineParams::{vfsg(), asg(), rat()}` (pole pairs 3, PMG 9 / 13.5 pole pairs, POR 360-800 Hz = 20 rpm per Hz, PMG = 3x POR for VFSG, 4.5x for ASG; saturation tanh; Xs scales with speed).
- `MachineParams` methods: `rated_phase_current_a, z_base_ohm, r_ohm, xs_ohm_at_ref, saturation, emf_per_rpm, open_circuit_emf(rpm, field_a), electrical_freq_hz(rpm), pmg_freq_hz(rpm)`.
- `SyncMachine::new(params, oil_c)`; `step(&MachineInput) -> MachineOutput`; accessors `disconnected, oil_temp_c, main_field_a`.
- `MachineInput {dt_us, shaft_rpm, exciter_current_cmd_a, disconnect_cmd (one-way latch), start_mode, motor_shaft_power_w, load_current_a (per-phase rms), load_pf_angle_rad (lagging +), oil_inlet_c}`.
- `MachineOutput {elec_freq_hz, pmg_freq_hz, pmg_phase_v, gcu_power_available, emf_phase_v, terminal_phase_v, thevenin, exciter_field_a, main_field_a, loss_w, shaft_power_w, oil_temp_c, disconnected, motoring_elec_power_w}`.
- `Thevenin {emf_v, angle_rad, r_ohm, x_ohm, freq_hz}` per-phase source for the network solver.

## sources::gcu
- `Gcu::new(GcuParams::{vfsg(), asg(), rat()})`; `step(&GcuInput) -> GcuOutput`; `powered_time_us, set_powered_time_us, latched_mask`.
- `GcuInput {dt_us, pmg_available, pmg_freq_hz, gcs_on, reset_cmd, phase_v[3], gen_i[3], feeder_i[3], gcb_closed, engine_start_mode, idle_valid, engine_at_idle, phase_seq_abc}`.
- `GcuOutput {exciter_current_a, vr_enabled, field_off, gcb_trip_cmd, gcb_close_permit, trip_latched, trip_mask, bite_code, phase_seq_fault, regulated_v, powered_ticks, voltage_established}`.
- `GcuFault` (BITE codes 1 OV, 2 UV, 3 OF, 4 UF, 5 differential, 6 overcurrent, 7 open phase, 8 phase sequence, 9 powered-time overflow); `.code()`, `.mask()`.
- Protections: OV 9-tier inverse time `OV_TIERS` 320/0.03, 310/0.05, 300/0.1, 290/0.2, 280/0.4, 270/0.6, 260/1.0, 250/1.75, 240/2.8 (reset 239), UV 216 V/4.5 s and 214.5 V/0.5 s (reset 222, inhibited by overcurrent 620/399 A and before voltage established), UF PMG 1083 Hz/4 s (inhibited in start mode or idle invalid), OF PMG 2397 Hz/100 ms (VFSG); ASG UF1 <1624.5 Hz/1.5 s (reset 1638), UF2 <1579.5 Hz/0.2 s (reset 1539), OF1 >1998 Hz/2.1 s (reset 1984.5), OF2 >2065.5 Hz/0.1 s (reset 2052), differential 30/45 A, 30 ms (needs GCB closed), overcurrent 375 A (ASG 340) inverse 330 s..5.1 s reaching the fast limit at 620 A (VFSG, ratio 620/375) or 399 A (ASG, ratio 399/340), open phase <11 A & spread >55 A / 4 s, phase sequence 100 ms (inhibits GCB close, not a trip).
- Latch: trips clear on GCS off/on or `reset_cmd`; PMG loss clears (power cycle). 248-day: 2^31 ticks of 10 ms (248.55 d) of powered time drops the generator, only cleared by a power cycle.
- PI voltage regulation with soft start and speed gain scheduling. `rat_breaker_close_permit(freq, &GcuOutput)` (>= 380 Hz); `ov_delay_s(v)`.

## sources::extpower
- `ExtPowerMonitor::new(Receptacle::{LeftFwd, RightFwd, Aft})`; `step(&ExtPowerInput) -> ExtPowerOutput {avail, power_ready, power_quality_good, epc_close_permit, tripped, fault: EpFault, source: Thevenin}`; `ExtPowerInput {dt_us, connected, pin_ef_ok, phase_v[3], phase_i[3], freq_hz, phase_seq_abc, engine_start_mode_active, atru_saturation}`.
- Documented forward-receptacle BPCU values (elec2_r2_sources): OV > 130 V inverse time (max 10 s; intermediate tiers are APPROX_EP_OV_TIERS), reset < 128 V; UV < 104.5 V for 9 s, reset > 107.5 V; OF > 429 Hz for 1 s, reset < 427 Hz; UF < 371 Hz for 1 s, reset > 373 Hz (OF/UF inhibited during UV); phase sequence not A-B-C for 100 ms, reset on A-B-C; power quality good after 150 ms with no UV/OV/UF/OF; OC1 270 A / 300 s (timer reset < 250 A), OC2 326 A / 5.2 s (timer reset < 306 A).
- Trips latch (lockout) until the reset condition is met; unplugging clears all lockouts. Pin E/F interlock, start-mode inhibit (forward only) and ATRU saturation (aft only) gate power ready.
- The aft receptacle uses the forward values (aft thresholds are not in the fact files). Open phase is current based: trip when lowest phase current < 11 A and next-lowest minus lowest > 55 A for 4.0 s; reset when lowest > 20 A or spread < 45 A (`EP_OPEN_PHASE_*`).

## storage::battery
- `BatteryParams::new(cells, capacity_ah)` (cells <= MAX_CELLS = 16; NO hard-coded capacity/cell count).
- `BatteryPack::new(params, serial_seed, soc, temp_c)`; `step(dt_us, discharge_a, ambient_c, bleed_mask) -> BatteryOutput {terminal_v, cell_v[16], soc_mean, temp_c, heat_w, current_a}`; `thevenin() -> (emf, r)`, `terminal_v(i)`, `open_circuit_v`, `soc_mean`, `cell_r0`, `cell_ocv`, `set_temp_c`, `ocv_from_soc`.
- Per-cell OCV table, R0 vs temperature and SOC, RC polarisation, pack thermal node, seeded SplitMix64 tolerances (capacity, resistance, initial SOC).

## storage::bmu
- `BatteryMonitoringUnit::step(&BmuInput) -> BmuOutput {balance[16], disconnect, over_voltage, under_voltage, over_temp, imbalance_fault, bmu_fail, charge_inhibit, min_cell_v, max_cell_v, spread_v}`; latched cutoffs cleared by `reset` when the condition is gone.

## storage::charger
- `BatteryCharger::new(ChargerParams::default())` (46 A, 32.2 V per eps24; 75 A is breaker rating only); `step(&ChargerInput) -> ChargerOutput {mode: ChargerMode (Off, ConstantCurrent, ConstantVoltage, Float, Fault), charge_current_a, output_v, input_power_w, failed}`; internal fault sensed 1.0 s -> Fault.

## storage::diode
- `BatteryDiodeModule::{conducts, evaluate(battery_v, battery_r, other_source_v, load_a), forward_v}`.

## Fact conflicts and APPROX
- Battery capacity: eps24/ata24 say 50 Ah, stm says 65 Ah, NTSB AIR-14/01 Table 1 says 75 Ah, 8 cells, 61.8 lb. Both are constructor parameters.
- Cell OCV table tops at 4.18 V / bottoms at 2.8 V; NTSB gives a 2.5-4.025 V operating range. Table not changed.
- VERIFIED (refs/facts/elec3_approx.md): `OV_TIERS` (eps24.txt:16981-16996), ASG OF/UF values (eps24.txt:17679-17689), `UV_ARM_V` 222 (eps24.txt:17110), EP open-phase rule (eps24.txt:23370), `CELL_OV_V` 4.2 and `CELL_UV_CUTOFF_V` 2.5 (NTSB AIR-14/01 p.10, p.33), `CHARGE_TEMP_MIN_C` -18 and `CHARGE_TEMP_MAX_C` 70 (NTSB p.9, p.66), `OVER_TEMP_C` 70 (NTSB p.66; cells self-heat from 62 C, so 65 is also defensible).
- DERIVED: `OC_FAST_RATIO_VFSG` 620/375 and `OC_FAST_RATIO_ASG` 399/340 (eps24.txt:17187, 17198, 9372), `R0_CELL_25C_OHM` 0.6 mOhm (R0+R1 0.9 mOhm from NTSB Thales comment ntsb.txt:3957-3961, at `APPROX_R1_RATIO` 0.5), `PACK_THERMAL_MASS_J_PER_K` 25 000 (28 kg x ~0.9 kJ/kg/K, NTSB Table 1).
- Still APPROX: EP OV inner tiers (only 130 V / 10 s sourced), EP source R/X, GCU kp/ki/soft start, PMG min freq, exciter/field taus, machine losses, oil thermal, battery R0 temperature coef / low-SOC factor / RC tau / cooling / tolerances / bleed, BMU delays, imbalance, balance thresholds, over-temp reset, charger efficiency/float/input min, diode Vf/R_on, `APPROX_R1_RATIO`.
