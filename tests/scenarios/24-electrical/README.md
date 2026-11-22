# 24-electrical scenarios

Scenario files: `NN_name.toml` in this folder. Fact tests: `tests/facts/24-electrical.toml`.

## Scenario schema

- `[scenario]`: `id`, `title`, `system`, `aircraft`, `engines`, `tick_hz`, `summary`.
- `[initial]`: free key/value preset applied before t=0. Keys used: `airplane_state` (cold_dark, cruise, powered_continuously), `on_ground`, `battery_switch`, `apu_battery_connected`, `ext_pwr_fwd_l|fwd_r|aft`, `gen_ctrl_switches`, `oat_c`, `alt_ft`, `engines_running` (array L,R), `apu_running`, `apu_gens_online`, `ext_power_on`, `gcu_power_cycled`.
- `[[step]]`: timeline action. Fields: `t_s` (seconds from start; runner converts to us), `actor` (crew, ground, fault, sim), `action`, `target`, optional `value` (scalar or inline table).
  - crew actions: press, select, rotate, move, cycle, open_page, cbic_open, cbic_lock, cbic_close, cycle_power, start.
  - ground actions: connect (value = `{v_ll, hz, kva}`), set.
  - fault actions: vfsg_fail, bus_fail, gen_loss, all_engines_flameout, asg_fail, bpcu_fail.
  - sim actions: advance (target `clock_s`, value seconds), set (target = state path).
- `[[expect]]`: observable outcome checked at `t_s`. Fields: `id`, `t_s`, `signal` (dotted sim-cli observable), `op`, optional `value`, `tol`, `unit`, `approx`, `cite`.
  - `op`: `true`, `false`, `eq`, `approx` (value +/- tol), `within` (value +/- tol), `lt`, `gt`, `present`, `absent`.
  - `approx = true` and `cite = "none"`: no documented value; runner reports these as APPROX (warn, never fail the suite).
  - `approx = false`: `cite` is `file:line` under `refs/facts`; failure fails the suite.
  - `eq` with `tol = 0.0` is exact.
- Expectations with `t_s` earlier than the matching step are static parameter checks (read at t=0).
- Signals not yet exposed by the system crate are reported as UNBOUND, not failed.

## Scenarios

- 01 cold and dark, battery ON, APU-start loads only
- 02 battery-only on ground, FCOM load list
- 03 external power L FWD, R FWD, AFT acceptance and transfer
- 04 external power quality trips (OV, UV, OF, UF)
- 05 APU start on APU battery, ASGs online
- 06 engine start, VFSG generator online, APU to engine transfer
- 07 loss of one VFSG in flight, ring tie pickup
- 08 loss of all AC, RAT auto-deploy, backup bus and battery-only items
- 09 L1 AC bus failure, QRH inoperative items
- 10 L2 AC bus failure, QRH inoperative items
- 11 R1 AC bus failure, QRH inoperative items
- 12 R2 AC bus failure, QRH inoperative items
- 13 load shed on external power overload and generator loss
- 14 CBIC open, lock, limited mode
- 15 GCU 248-day counter overflow (all APPROX)
- 16 battery OFF shutdown sequence and MBR hold

## Fact test schema (`tests/facts/24-electrical.toml`)

- `[[fact]]`: `id` (F24-nnn), `quantity` (dotted name), `expected`, `unit`, `tol` (absolute), `condition`, `approx`, `cite` (`file:line`, file under `refs/facts`).
- Runner compares the model parameter or measured value named by `quantity` against `expected` +/- `tol`.

## Corrected expectations

| Scenario | Id | Old | New | Fact quote | Reason |
|---|---|---|---|---|---|
| 01 | e01 | hbb_main.voltage 28 +/- 4 V | 29.5 +/- 5.5 V (24..35 V) | "Hot Battery Bus < 24 VDC (UV Fault)" elec2_r5_batteries_backup.md:61; "> 35 VDC (OV Fault)" :62 | Cite gives only the fault limits; nominal 29.6 V (r5:191) and charger 32.2 V (r5:35) lie inside the documented window |
| 05 | e04 | apu_batt.voltage 28 +/- 4 V, cite r5:12 | 29.5 +/- 5.5 V, cite r5:61 | "Hot Battery Bus < 24 VDC (UV Fault)" elec2_r5_batteries_backup.md:61 | Cited row has no voltage; same 24..35 V window as 01.e01 |
| 03 | e02 | L_115 energized false at 1 s | true | "the 115 Vac system is energized directly." facts_elec_dist.verified.md:57 | Forward EP connected on the ground energizes the 115 V system in Ground Handling mode; cite moved from elec_conditions:57 |
| 03 | e04 | fwd_l EPC closed false at 5.1 s | false at 0.1 s | "Fwd EP power quality must hold 150 +/-10 ms (no UV, OV, UF)" elec2_r8_timing_loads.md:39 | The 150 ms quality hold applies at connection (t=0); Ground Handling closes the EPC before the ON press |
| 03 | e23 | L_115 energized false at 51 s, approx | true, cite dist:57 | "the 115 Vac system is energized directly." facts_elec_dist.verified.md:57 | Both forward receptacles stay connected after FWD OFF, so Ground Handling keeps the 115 V system energized |
| 04 | e07 | fwd_l EPC closed false at 90 s, cite r2:118 | false at 91 s, cite r2:115 | "OV when highest phase > 130 Vrms, trips and locks out within inverse time (10 s max)" elec2_r2_sources.md:115 | Row 118 is the UV row; the 131 V step is at 80 s so 90 s sits exactly on the 10 s bound |
| 04 | e08 | locked_out true at 90 s, cite r2:118 | true at 91 s, cite r2:115 | same as e07 | same as e07 |
| 04 | e15 | fwd_ep_ov_rms cite r2:118 | cite r2:115 | same as e07 | OV threshold lives in row 115, not the UV row |
| 09 | e02 | ELEC_AC_BUS_L1 present at 10.1 s | present at 10.6 s, cite r7_crew_interface.verified.md:195 | "235 VAC bus voltage < 130 Vrms for 500 ms (reset > 165 Vrms for 500 ms)" elec2_r7_crew_interface.verified.md:195 | Documented trigger needs 500 ms below 130 Vrms; fault at 10.0 s |
| 10 | e02 | ELEC_AC_BUS_L2 present at 10.1 s | present at 10.6 s | same as 09.e02 | same as 09.e02 |
| 11 | e02 | ELEC_AC_BUS_R1 present at 10.1 s | present at 10.6 s | same as 09.e02 | same as 09.e02 |
| 12 | e02 | ELEC_AC_BUS_R2 present at 10.1 s | present at 10.6 s | same as 09.e02 | same as 09.e02 |
| 07 | e07 | ELEC_GEN_OFF_VFSG_L1, cite gen:364 | ELEC_GEN_OFF_L1, cite elec2_r7_crew_interface.verified.md:204 | "ELEC GEN OFF L1, L2, R1, R2 | Advisory | Generator control breaker open" elec2_r7_crew_interface.verified.md:204 | The documented message name has no VFSG token |
| 14 | e12 | cbic.mode limited after BPCU L+R fail | ground | "a second BPCU assumes all functions" elec_conditions.verified.md:40; "aircraft moving (>2 kt) or not on the ground" elec2_r6_distribution_units.md:180 | The airplane is stationary on the ground, so limited mode is not entered |
| 15 | preset | uptime preset 247.99 d, advance 1 h | 248.513 d, engines_running L,R added | "2147483648 cs" limit = 248.55 d (own e01) | 247.99 d + 1 h never reaches the 248.55 d limit; overflow now lands at about 3313 s, between e03 (3000 s) and e04 (3601 s); GCUs are PMG-powered so engines must run for the counter to tick |
