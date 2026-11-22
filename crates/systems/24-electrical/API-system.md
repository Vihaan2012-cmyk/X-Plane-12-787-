# API: system (b789-sys-electrical, feature `system`)

Closed-loop electrical system. `system::Electrical` owns the net, GCUs, ext power monitors, contactor bank (one `Contactor` per contactors.csv row), BPCU pair, load manager, CBIC cluster, BMUs, chargers, panel, EICAS, synoptic.

## Public API
- `Electrical::new(data_dir) -> Result<Electrical, String>`; `step(dt_us, &ElectricalInputs)` (1 ms sub-steps); `step_idle(dt_us)` reuses last inputs.
- `ElectricalInputs {crew: Vec<CrewCommand>, engine_rpm[2] (VFSG shaft rpm, 20 rpm per Hz; use >= 7500), engine_running[2], apu_rpm, apu_running, ext[3] (ExtSupply {connected, v_ll (monitored phase volts, 115), hz, kva, phase_seq_abc, pin_ef_ok}), on_ground, ground_speed_kt, radio_alt_ft, rat_deployed, rat_rpm, ambient_c, doors_open}`.
- `CrewCommand`: Battery(on), ApuGen(i,on), GenCtrl(i,on), GenCtrlCycle(i), ExtPwrFwd(i), ExtPwrAft, DriveDisc(i), GroundService, ApuStart, CabinUtil(on), IfeSeats(on), LampTest(on).
- `value(name) -> Option<SignalValue {Bool,Num,Text}>`, `signal(name) -> Option<f64>`, `flag(name) -> Option<bool>`, `text(name)`, `snapshot() -> Vec<(String, SignalValue)>`, `eicas_messages() -> Vec<String>`, `mode()`.
- `action(actor, action, target, value) -> Result<(), String>`.
- Accessors: panel/panel_mut, lamps, synoptic, eicas, bpcu_output, load_output, cbic/cbic_mut, gcu_output(m), ext_output(k), contactor_closed(name), contactor_names, bus_volts(bus), faults_mut (`Faults {gen_loss[4], asg_loss[2], bpcu_fail[2], bus_fail[4]}`).

## Loop per sub-step
1. panel update (switch pulses, ext switch toggle, drive disconnect, APU start).
2. sensing from previous solve: ext monitors (receptacle volts/Hz, EPC current), GCUs (machine terminal V/I, PMG from rpm), BPCU (aux contacts, bus volts, GCU/ext status), load manager, CBIC.
3. commands -> contactor coils (28 V from hot battery/DC bus) -> `Contactor` dynamics -> net switch state.
4. GCU field-on -> `set_machine_field` (1.0 / 0.0 feed-forward), shaft rpm, RAT.
5. net step. 6. BMU/charger, `CrewInputs` rebuilt from the solution, EICAS and synoptic updated.

## Actions (runner)
- crew: press/select/rotate/move/cycle/start on BATTERY, APU_GEN_L/R, GEN_CTRL_<L1|L2|R1|R2> (value OFF_ON = cycle), EXT_PWR_FWD_L/FWD_R/AFT, DRIVE_DISC_<gen>, GROUND_SERVICE, APU_SELECTOR (START), CABIN_UTIL, IFE_SEATS.
- ground: connect EXT_PWR_FWD_L|FWD_R|AFT (value "v_ll=115 hz=400 kva=90"), set <target>.hz|v_ll|kva, disconnect.
- fault: gen_loss/vfsg_fail (L1_VFSG.. or ALL_VFSG), asg_fail, bpcu_fail, bus_fail (L1/L2/R1/R2: contactors of that bus forced open), all_engines_flameout.
- sim: advance clock_s (steps with last inputs), set on_ground|oat_c|rat_deployed|apu_running.

## Signals
- elec.mode (cold_dark, on_ground_battery_only, ground_handling, external_power_on, generator, in_air_rat_only, in_air_battery_only)
- elec.bus.<L1_235|L2_235|R1_235|R2_235|L_115|R_115|backup|hbb_main|ltb|rtb|dc28_l|dc28_r>.<energized|voltage>; elec.bus.<L1_235..R2_235>.source (vfsg_l1, asg_l, ext_fwd_l, none)
- elec.other_ac_buses.energized_count
- elec.epc.<fwd_l|fwd_r|aft>.<closed|locked_out|avail|on>; elec.ext.<fwd_l|fwd_r|aft>.<avail|quality_good|tripped|switch_on>
- elec.gcb.<l1|l2|r1|r2>.closed; elec.vfsg.<l1..r2>.<mode|freq_hz|load_kva>; elec.gcu.<l1..r2>.fault_failsafe; elec.asg.<l|r>.gen_on_bus
- elec.mbr.closed; elec.cit.closed; elec.contactor.<name with _ for space>
- elec.rat.deployed; elec.rat.deploy_requested
- elec.load_shed.active|count; elec.ccs.powered; elec.cbic.mode
- elec.msg.<EICAS_NAME_WITH_UNDERSCORES> (e.g. ELEC_AC_BUS_L1)
- elec.battery.<main|apu>.<voltage|current|temp_c|bmu_disconnect|charge_a|diode_forward_v>

## PDU link (`system::pdu_link`)
- `PduLink` runs 17 RPDUs (4 gateways 71 72 81 82, clusters APPROX) and 2 SPDUs in the loop after each net solve (`stage_pdu`); writes channel truth back with `set_sspc`; I2t / instant / arc trips really open the net channel; tripped needs reset; locked ignores close.
- Mapping: 128 RPDU channels (loads.csv order = `RPDUnn.kk`) onto DC19/AC12 modules sized from documented channel mixes; 23 feed SSPCs (breaker ids) on SPDU DC2/DC4 cards from breakers.csv; other SPDU ECB rows with a card slot < 16 become dormant devices. Rating from breakers.csv where it matches the card, else module rating (APPROX); `summary()` counts both.
- Inputs: controller power = RPDU DC node V + backup RPDU node V; comms OK unless forced loss, gateway loss (cluster) or both BPCUs failed (CDN stand-in); load-shed outputs map by load-name keywords (APPROX, window heat mostly) with source LoadShed; CBIC open/close/lock/unlock/reset commands via `queue`; defaults: standby/backup supplies On, shed groups Off, rest LastCommanded; 3 s comms default, immediate on controller power loss.
- Outputs: `breaker_truth(id)` feeds `elec.cb.*`; `rpdu_views()` rows (id, location, input V/A, powered, channels with rating, load, state, current, V out, I2t %, arc events, history); `inspector_data()` (feature inspector) fills `InspectorData.rpdus`.
- Test hooks: `pdu_mut().{set_comms_loss, set_controller_power_loss, inject_overload, inject_arc, set_channel_open, set_channel_lock, reset_channel}`.
- Not modelled: ELCU AC feeds (still gear contactors), SPDU comms/controller faults.

## Gaps
- No battery charger current into the net; machine field is a 0/1 gate (net excitation is feed-forward); ext cart voltage/Hz only drive the monitor (the net source is fixed 115 V 400 Hz while connected and in range); BPCU ground-handling mode is treated as ON for feed ranks so BSB/ATUC are gated to ExtPowerOn here; RAT deploy/spin is input-driven (BPCU only requests); ELCU AC feeds not in the PDU link.
- GCU PMG under-frequency trips below ~7500 rpm VFSG shaft speed (1083 Hz limit vs 3x POR).
