# crew module API (src/crew)

Feature `crew`. Pure models, plain f64 SI and u64 us. No deps. Index order for generators/buses: `[L1, L2, R1, R2]`; engines `[L, R]` (engine = gen index / 2); APU gen `[L, R]`.

## inputs.rs
- `CrewInputs` (Default, Clone): every sensed signal the crew models read (time `now_us`, `phase`, `on_ground`, `lamp_test`, `dim`, switch states, GCB/APB/EPC states, bus volts, battery values, fault flags and sensed durations in us).
- `FlightPhase { Ground, TakeoffInhibit, Flight, LandingInhibit }`
- `classify_phase(gs_kt, radio_alt_ft, on_ground, takeoff_config) -> FlightPhase` (APPROX 80 kt, 400 ft, 800 ft)

## switch.rs
- `Switch`, `SwitchKind { Alternate, Momentary, GuardedMomentary }`: `new`, `press -> bool`, `release`, `set_guard`, `set_latched`, `is_on`, `take_pulse`, `has_pulse`; fields `latched`, `pressed`, `guard_open`.
- `Lamp { color, lit, brightness }`, `LampColor { White, Amber, Green, Blue }`, `Lamp::drive(color, command, lamp_test, dim)`, `BRIGHT_LEVEL`, `DIM_LEVEL` (APPROX 0.4).

## panel.rs
- `OverheadPanel`: switches `battery`, `apu_gen[2]`, `gen_ctrl[4]`, `apu_selector` (`ApuSelector { Off, On, Start }`, START spring-back via `release_apu_selector`), `ext_pwr_fwd[2]`, `ext_pwr_aft`, `drive_disconnect[4]` (guarded momentary), `ground_service` (momentary, attendant panel), `cabin_util`, `ife_seats`, `towing_power`, `towing_test`. No bus tie switch (facts: none exists).
- `new`, `set_normal`, `press_gen_ctrl(i)` (reports an OFF to ON cycle once), `press_drive_disconnect(i)`, `release_drive_disconnect(i)`, `set_apu_selector`, `release_apu_selector`.
- `update(&mut self, &CrewInputs) -> (PanelOutputs, PanelLamps)`.
- `PanelOutputs`: switch states for controllers (`battery_on`, `gen_ctrl_on`, `gen_ctrl_cycled`, `apu_gen_on`, `ext_fwd_pulse[2]`, `ext_aft_pulse`, `drive_disconnect_request[4]`, `ground_service_on`, `towing_power_on`, ...). Drive request gated: in flight only when DRIVE lamp lit; on ground engine running or fuel switch RUN; never if already disconnected. Hold-1-s (QRH) not modelled.
- `PanelLamps`: `battery_off`, `apu_gen_off/avail/on[2]`, `gen_off[4]`, `drive[4]`, `ext_fwd_avail/on[2]`, `ext_aft_avail/on`, `cabin_util_off`, `ife_off`, `ground_service_on`, `towing_on_bat`, `towing_high/medium/low`. All honour lamp test and dim.

## eicas.rs
- `Level { Warning, Caution, Advisory, Status, Memo }`, `MsgId` (one variant per message, index payload), `MessageDef` (level, on/off delay us, latched, inhibit flags, `delay_approx`, beeper), `EicasLine`.
- `message_catalogue() -> Vec<MessageDef>` (91 messages: 4 caution AC BUS, 26 advisory/memo, rest status).
- `EicasElec`: `new`, `update(&CrewInputs)`, `list() -> Vec<EicasLine>` (level order, newest first within level, inhibited hidden), `names`, `is_active`, `is_shown`, `beeper`, `cancel_aural`, `master_caution`, `defs`, `message_count`.
- Conditions with thresholds and hysteresis from eps24 (AC BUS <130 Vrms 500 ms reset >165; BATTERY OFF 1 s; CABIN/UTIL, IFE/SEATS 500 ms; hot bus <22 V 1.5 s reset 24; 115 V <50/85; 28 V <18/22 1.38 s; GEN SYS APB/GCB disagree >150 ms, 28 V fail 1.5 s; DRIVE LOP >500 ms latched; DEBRIS and GEN SYS APU latched until switch cycled). Advisories inhibited in takeoff and landing phases (APPROX); caution, status, memo not inhibited. Delays marked `delay_approx` are APPROX.

## synoptic.rs
- `build_synoptic(&CrewInputs, gen_valid[4], bus_valid[4]) -> SynopticPage`.
- `SynopticPage { gens[4] (SynGen: source state/colour, drive state, DRIVE text, starter, STARTERS text), apu_gens[2], ext_fwd[2], ext_aft, buses[4], large_motor (L/R/APU START arrows), main_battery, apu_battery (rounded VOLTS/AMPS, CHG/DISCH), lines (FlowLine green/white), load_shed }`.
- `ElementState { On, Available, Off, Failed, Invalid }`, `Colour { Green, White, WhiteLow, Amber, AmberLow, Cyan }`, `DriveState`, `BatteryLabel`.
- Colours from facts: green on, white off, amber failed, low-intensity white invalid. Flow-line routing from APU/ext sources to buses is APPROX (facts give no map).
