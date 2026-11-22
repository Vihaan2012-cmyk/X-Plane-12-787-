# API: control (b789-sys-electrical, feature `control`)

Re-exported from `control` (mod.rs). All plain f64 SI / u64 microseconds, no cross-crate deps.

## control::bpcu
- `BpcuInput` (state sensed: now_us, on_ground, gen_ok/gen_ctrl_on/gcb_aux_closed [4], asg_ok/asg_ctrl_on/asg_cb_aux_closed (APB aux) [2], ext_plugged/quality_ok/switch_on/epc_aux_closed [3: L FWD, R FWD, AFT], btb_aux_closed [6], bus_volts [4], atu_ok [2], dc_main_ok [2], batt_switch_on, apu_batt_switch_on, apu_start_requested, refuel_panel_active, rat_gen_ok, backup_bus_ok, ground_service_switch, engines_running); `BpcuInput::cold_and_dark(now_us)`.
- `BpcuPair::new()`, `step(&mut self, &BpcuInput) -> BpcuOutput`, `rat_deployed()`, pub `left_powered`/`right_powered` (a powered peer takes over all commands of an unpowered one; both unpowered: all open).
- `BpcuOutput`: gcb_close[4], asg_cb_close[2] (L/R APB), epc_close[3], btb_close[6] (L1 BTB, L2 BTB, R1 BTB, R2 BTB, L3 BTB, R3 BTB), bsb_close[2], btb115_close[2], bus_feed[4] (`Feed::{None, Src(Src)}`, `Src::{Vfsg(k), Asg(k), Ext(k)}`), bus_powered, ext_avail, ext_on, source_kva, source_change, transfer_pending, instr_parallel, main_batt_relay_close, apu_batt_relay_close, ground_config (`GroundConfig::{FlightOrNormal,BatteryOnly,GroundHandling,ExtPowerOn}`), only_apu_start_loads, sspc_elcc_loads_enabled, refuel_on_battery, ground_service_mode, ext_full_power, capt_inst/fo_inst/standby (`InstFeed`), rat_deploy, rat_cb_close, bb_iso_open, c1_tru_iso_open, controlling[2], all_ac_lost.
- Tables: `RANK_GROUND`, `RANK_AIR` (source order per bus), `BTB_NODES`, `BTB_NAMES`, `NODE_NAMES` (L1 L2 R1 R2 LTB RTB), `APB_NODE`, `EPC_BUS`; `loop_plant` (test helper).
- Behaviour: planner picks, per dead bus, the best-ranked available source with a free path over the BTB graph (a tie bus or bus is owned by one source, so no paralleling); break-before-make (a contactor closes only after every aux-closed contactor sharing a node that the plan opens is open).

### VERIFIED-sourced (elec2_r4_topology_bpcu.md, 24eps figures / eps24 / ata24)
- Topology: LTB joins L APB (ASG L), L2 BTB (L2), R1 BTB (R1); RTB joins R APB (ASG R), L1 BTB (L1), R2 BTB (R2); L3 BTB L1-L2; R3 BTB R1-R2; BTBs normally open.
- Per-bus source ranks: L1, L2, R1, R2 tables incl. ground/air swap of rank 4/5 on L1 and R1 (L2/R2 air ranks used for ground too); L FWD EP then R FWD EP last.
- BPCU owns EPC, BSB, 115 BTB, CIT/FIT, MBR; 115 VAC BSB/BTB rule; ground handling (fwd EP connected, not ON, no APU/engines): first EPC closes; EP ON mode gives full-airplane power; aft EP powers only L2 ATRU (never a main 235 bus); fwd EP 90 kVA limit.
- Ext power quality true after 150 ms; CIT/instrument buses paralleled before any transfer and 15 s after; CIT closed 120 s on ground-to-air; MBR closes on transfer pending or RAT deployed and opens 15 s after command lost; RAT request 15 s after L1 235 VAC and backup bus lost; refuel on battery 15 min.

## control::load_mgmt
- `DEFAULT_LOADS: [LoadDef; 14]` (name, group, kw, ground_priority, flight_priority, shed_on_engine_start, sheddable), `LoadGroup`.
- `LoadManager::new()`, `step(&mut self, &LmInput) -> LmOutput`, `power_up_reset(now_us)`, `LoadManager::priority(k, on_ground)`.
- `LmInput`: now_us, on_ground, available_kw, demand_on[14], engine_start, source_change, ext_in_use, ext_kva, overload_hint_kw. `LmOutput`: shed[14], override_status[14] (shed with breaker closed), breaker_closed[14], total_kw, shed_count, ext_overload_5s/5min.
- Timing: min shed 5 s, restore delay 10 s stable, 0.5 s between actions, 5 kW restore margin, 120 s engine-start hold, EP overload >112.5 kVA 5 s / >90 kVA 5 min.

## control::cbic
- `Cbic`: `new`, `add_breaker(id, name, BreakerKind, CbState)`, `breaker`, `update(&CbicInput)` (mode: Limited if >2 kt or not on ground or no door/EP; Maintenance with GND TEST; else Ground), `mode`, `connect(UserKind) -> Result<UserId, Refusal>` (3 MFD + 1 laptop, fifth refused), `disconnect`, `command(user, id, CbAction, ctrl_armed) -> CmdOutcome` (ground: CTRL armed executes; flight: PendingEnter then `press_enter`, 10 s timeout), `take_commands`, `report_status`, `report_comm_loss`, `report_override`, `output_status` ("OVERRIDE"/"ON"/"OFF"), `open_trip_page`, `by_state`, `status_pages`, `recent_used` (30), `search`, `set_custom_list`/`custom_list` (maintenance only), `export_nvm`, `restore_nvm`, `power_cycle`.
- `CbicCluster` (hosts L1, L4, R1): `new(app, airplane_seq)`, pub `app`, `set_host_power`, `active_host`, `is_powered`, `sync_nvm`, `set_airplane_seq`; locks persist via NVM across total power-down, discarded on airplane-sequence mismatch.
- Types: `BreakerKind::{Ecb,Tcb,Elcc,FcSsr}`, `CbState::{Open,Closed,Tripped,Locked(LockTag),Unknown}`, `LockTag::{Inop,DoNotClose}`, `CbAction::{Open,Close,Lock(tag),Unlock,RequestReset}`, `Mode`, `UserKind`, `UserId`, `Refusal`, `CmdOutcome`, `BreakerCommand`, `CbicInput`, `Nvm`.

## APPROX
- Ext source attach: L FWD EP enters at L1 and R FWD EP at R1 (via ATU/ATUC); L2/R2 ground ranks reuse the air ranks (only air ranks were read); RTB ranks are mirrored implicitly through the per-bus tables.
- 300 ms EPC close hold after quality is interpreted as a contactor-change hold; GH-mode EP ranks 9/10 not distinguished from GO 7/8.
- VFSG/ASG kVA ratings; load table, all load-management timings and the 5 kW margin; engine-start shed set; 10 s CBIC confirm timeout; CBIC limited when on ground with no door open and no EP AVAIL.
