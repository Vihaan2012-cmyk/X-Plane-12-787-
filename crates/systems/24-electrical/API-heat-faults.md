# API: heat + faults (b789-sys-electrical, features `heat`, `faults`)

Units SI f64 (K, W, A, ohm, J), time u64 microseconds. No comments, no unsafe, deterministic (BTreeMap everywhere).

## heat::ElecThermal
One `solver_thermal::Network` per `Zone` (smaller dense solves), each with an ambient boundary and a cooling-air boundary.
- `Zone {EeBayFwd, EeBayAft, Crown, WheelWell, Nacelle, Cabin}`, `ZONES`, `.index()`, `.default_k()` (298/298/293/288/308/295 K, APPROX).
- `new()`, `from_parts(&[PartSpec]) -> Result<_, HeatError>`; `PartSpec::{Wire, Contactor, Battery, Module, Generator}`; `HeatError::{DuplicateId, UnknownId}`.
- Builders (id is the part instance id, e.g. the net/CBIC id):
  - `add_wire(id, zone, rating_a, length_m) -> Sizing`: one copper node per segment, gauge from `size_wire(rating)`; rating above the largest gauge uses parallel conductors (current split).
  - `add_contactor(id, zone, rating_a)`: contact node + case node (contact R = 50 mV / rating at 20 C).
  - `add_battery(id, zone, cells, cell_mass_kg, cell_r_ohm_20c)`: one pack node (I2R(T) with cold rise, entropic term).
  - `add_module(id, zone, ModuleKind::{Sspc, Rpdu, Spdu, PanelBox}, dissipation_w)`: core + shell, cooling-air eps-NTU link.
  - `add_generator(id, zone, rated_current_a, rated_loss_w)`: winding + oil nodes, oil cooler link to cooling air.
- Inputs: `set_current(id, a)`, `set_dissipation(id, w)` (extra watts: coil, internal, iron/mech), `set_resistance_multiplier(id, m)` (fault high-resistance heats the part), `set_cooling_flow(module, kg_s)`, `set_oil_cooling(gen, 0..1)`, `set_zone_temp(zone, k)`, `set_cooling_air_k(zone, k)`, `cold_soak(zone, k)` (zone and every part node in it set to k).
- `step(dt_us) -> StepReport`: sub-steps at most 1 s, sources re-evaluated from start-of-substep temperature (wire/contact/battery/generator resistance rises with T).
- Outputs (all `Option` for unknown id): `temperature_k` (wire/contact/battery node, module core, generator oil), `secondary_temperature_k` (contact case, module shell, generator winding), `heat_w`, `resistance_factor` (R(T)/R(20 C); battery factor is cold-sensitive), `resistance_ohm` (wire, contact), `ampacity_a` (live, depends on zone ambient), `margin_to_limit_k` (wire, to 473.15 K), `zone_of`, `kind_of`, `part_ids`, `len`, `zone_temp_k`.
- `energy_report() -> EnergyReport {elapsed_s, source_j, boundary_j, stored_delta_j, imbalance_j, stored_j}`.
- Gauge: `GAUGES` (AWG 24..4/0), `size_wire(rating) -> Sizing {gauge, parallel, limit_a}`, `ambient_derate`. Rule: smallest gauge with `free_air_a * 0.65 >= rating`, conductor limit 200 C. Rule shape is STANDARD (AS50881 free-air/bundle derating); the ampacity table and 0.65 derate are APPROX. Wire surface h is calibrated so that bundle-limit current at 20 C ambient settles exactly at 473.15 K.
- APPROX: all module/contactor/battery/generator thermal constants, zone temperatures, cell cold coefficient (0.025/K), entropic 5e-5 V/K per cell.

## faults
- `catalog() -> Registry` (from `CATALOG`): 33 part types, 133 modes, each with at least one observable. Observable prefixes (`OBSERVABLE_PREFIXES`): `eicas:`, `bite:`, `sig:`, `lamp:`, `cbic:`, `therm:`.
- Part type consts (`VFSG`, `GCU`, `CONTACTOR`, `SSPC`, `WIRE_SEGMENT` ...) and `part_types()`.
- `FaultBook` (per part instance id -> `FailureSet`, type-checked against the catalog):
  - `arm(part, part_type, mode, magnitude, Onset, now_us)`, `arm_immediate(...)`, `arm_intermittent(part, part_type, mode, magnitude, (period_us, duty), now_us)` (seed = `seed_for(part, mode)`, deterministic), `clear(part, mode)`, `clear_part`, `clear_all`, `trigger(name, now_us)` (latent onset).
  - `magnitude(part, mode, now_us)`, `snapshot(part, now_us) -> Magnitudes`, `active(now_us)`, `active_observables(part, now_us)`, `part_type`, `registry`.
  - Per-type helpers: `contactor`, `sspc`, `wire`, `connector`, `battery_cell`, `machine` (VFSG/ASG/RAT), `gcu`, `converter` (ATRU/ATU/TRU/SPU/CMSC), `indication` (lamp/EICAS input).
- Pure mapping functions on `Magnitudes` (modes absent = 0): `contactor_effect -> ContactorEffect {welded, coil_open, aux_stuck, stuck_open, resistance_mult, operate_time_mult}`, `sspc_effect -> SspcEffect {stuck_open, stuck_closed, force_trip, arc_false_trip, current/voltage_telemetry_scale, leakage_mult, trip_disabled}`, `wire_effect -> {open, resistance_mult, ground_fault_ohm, bus_short, arcing}`, `connector_effect`, `battery_cell_effect -> {internal_short_ohm, capacity_scale, r0_mult, extra_heat_w, runaway_precursor}`, `machine_effect -> {friction_loss_w, vibration, oil_flow_scale, exciter_ripple, phase_open, winding_short, pmg_scale, disconnect_jam}`, `gcu_effect -> {setpoint_offset_v (+-40 V), cpu_failed, powered_time_preload_us, ttpc_lost, ct_offset_a, fan_failed}`, `converter_effect`, `indication_effect`.
- Conventions: binary modes are set at magnitude >= 0.5 (`BINARY_THRESHOLD`); `resistance_multiplier(m) = 1 + 99 m`; `ground_fault_resistance_ohm(m)` log-interpolates 1 Mohm (m to 0, None at 0) to 0.05 ohm (m = 1); `counter_preload_us(m)` scales to 2^31 ticks of 10 ms (248.55 d) so m = 1 is the overflow point.
- Causal wiring (system/causal.rs, net/overlay.rs): faults act only on model state; `elec.fault.<part>.<mode>` is a debug readout, not an observable, and no EICAS message or `obs` signal is injected.
  - Net: `set_shunt` (ground-fault / lamp short / RAT heater leakage), `set_converter_mod` (ATRU/TRU/ATU ratio, impedance, open), `set_battery_diode_force/drop_ohm`, `set_rpdu_feed_series_ohm` (ground-stud return path), `shunt_site`.
  - Wire chafe / pin-short: shunt at the feeder source node; GCB pickup on sensed machine current (or estimated leak current) opens the feeder and drops the generator.
  - ELCC/ELCU -> `AC FEED RPDU n` contactors; TCB/SPDU card/RPDU module -> feed SSPCs; RPDU controller -> pdu comms/power loss; ground stud -> series ohm on RPDU feeds (DC diode switch, AC feed contactor).
  - Sensors: EICAS input faults alter `CrewInputs` sensor fields; CMSC/SPU/OAV set the unit's BIT flags; BMU/charger faults go through `BmuInput`/`ChargerInput`; receptacle faults alter pin E/F and phase voltages seen by the monitor.
  - Panel: switch stuck/open-contact/bounce rewrite `PanelOutputs`; lamp faults dark/dim `PanelLamps`; lamp short loads DC28_L.
  - Thermal: OAV locked-closed cuts oil cooling, PECS pump loss cuts module cooling flow.
  - Audit (thermal_fault_tests): arms one mode per part type and requires a change in a fingerprint of real state (buses, contactors, SSPC/loads, batteries, lamps, EICAS, CBIC, GCU/ext/BPCU/BMU/charger outputs, RPDU rows, sensed inputs, temperatures) versus a fault-free run.
- Hooks for the system loop: apply `resistance_mult` to the electrical element AND to `ElecThermal::set_resistance_multiplier`; `extra_heat_w`/`friction_loss_w` to `set_dissipation`; show `active_observables` in the inspector (rule 9 audit).

## Tests
- heat (13): wire settles at 473 K at limit current, wire at rating below limit, parallel feeders, contactor heats and R rises, high-resistance heats more, battery warms under discharge, cold soak (long soak and direct), module cooling air, generator oil/cooler, energy balance, from_parts, determinism.
- faults (16): audit passes, no duplicate modes, observable prefixes, all 28 required types plus 5 extras, required modes present, unknown/mixed type rejected, arm/clear per instance, effect mappings (contactor, binary threshold, wire, SSPC, cell, GCU drift and 248-day preload, machine), deterministic intermittent with duty, latent trigger and active observables.
