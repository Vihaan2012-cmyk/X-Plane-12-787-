# API: protection + pdu (b789-sys-electrical)

Units: SI f64 (V, A, ohm, J, degC), time u64 microseconds. Feature `pdu` requires `protection`.

## protection::sspc
- Consts: `RPDU_RATINGS_A` [2.5,5,7.5,10], `SPDU_RATINGS_A` [15,22.5,35,45,50], I2T_CONSTANT_S, INSTANT_TRIP_MULTIPLE, COOLING_TAU_S, SOFT_START_US, arc thresholds, telemetry resolutions (APPROX, see below).
- Enums: `SspcState {Open, Closed, Tripped, LockedOpen}`, `SspcKind {DcSoftStart, AcZeroCross}`, `CommandSource {None, Local, System, LoadShed}`, `TripCause {I2t, Instantaneous, SeriesArc, ParallelArc}`, `SspcHealth {Ok, OverCurrentTripped, ArcFaultDetected, LoadFeedUndervoltage, LockedInop}`.
- `SspcConfig {rating_a, kind, phases}` with `::dc(rating)`, `::ac(rating, phases 1..=3)`.
- `SspcInput {dt_us, v_in, currents_a:[f64;3], noise_a, close_cmd, source, reset, lock, v_zero_cross, i_zero_cross}` (Default).
- `SspcOutput {state, conducting, v_out, current_a:[f64;3], current_limit_a, soft_starting, inop_tag, trip_cause, telemetry}`; `SspcTelemetry {v_in, v_out, current_a, health}` quantized.
- `Sspc::new(config)`, `.step(&SspcInput) -> SspcOutput`, `.state()`, `.rating_a()`, `.heat_fraction()`, `.last_source()`, `.inop_tag()`; `quantize(x, res)`.
- Behaviour: I2t trip per phase with thermal memory and cooling; instantaneous trip (blanked during soft start); series and parallel arc detectors from `noise_a`; DC soft start (current limit ramp over 5 ms); AC closes on `v_zero_cross`, opens on `i_zero_cross` (faults trip immediately); lock = LockedOpen + INOP tag, ignores close; trip needs `reset` with close removed.

## protection::thermal_breaker
- `ThermalBreaker::new(rating_a)`, `.step(&BreakerInput {dt_us, current_a, ambient_c, pull, push}) -> BreakerOutput {state, closed, theta, current_a}`, `.state()`, `.theta()`, `.rating_a()`; `BreakerState {Closed, Tripped, Pulled}`.
- Bimetal first-order model, MS-style band (no trip at 100 %, trips ~46 s at 135 %, ~13 s at 200 %, ~1.2 s at 600 %), ambient shift, trip-free (push refused until theta < 0.9).

## protection::contactor
- `Contactor::new(ContactorConfig)`, `.step(&ContactorInput {dt_us, coil_v, current_a}, line_v) -> ContactorOutput {contacts_closed, armature_closed, aux_no, aux_nc, resistance_ohm, welded, bouncing}`, `.cycles()`, `.arc_energy_j()`, `.welded()`, `.force_weld()`, `.resistance_ohm()`.
- `BOUNCE_EDGES_US` bounce sequence; resistance = r0 * (1 + wear per cycle + wear per arc joule); weld by arc energy or `force_weld`.

## protection::elcu
- `SupplyClass {Ac235, Dc270, Ac115}` with `on_threshold()` / `off_threshold()`; `ElcuPowerSense::new(class)`, `.update(v) -> powered`, `.powered()`.
- `Elcc::new(rating_a)` (thermal breaker + contactor), `.step(&ElccInput) -> ElccOutput`.
- `ElcuP::new(&[ratings])` (max 6 ELCC), `ElcuC::new(class)`, `.add_module(ElcuP) -> bool` (max 6), `.step(supply_v, dt_us, close_cmds, currents_a, line_v, ambient_c) -> Vec<Vec<ElccOutput>>`.

## pdu::channel
- `ChannelAddr {slot, channel}`, `DefaultState {Off, On, LastCommanded}`, `ChannelCommand {addr, close, source, reset, lock}`, `ChannelLoad {addr, currents_a, noise_a}`, `ChannelStatus {addr, rating_a, state, v_out, current_a, telemetry, default_active, inop_tag}`, `Channel` (SSPC + commanded/default/lock).

## pdu::rpdu
- `RpduKind {Gateway, Standard}`, `PowerModuleKind {Ac12, Dc19, Spare}`, `PowerModule::new(kind)`, `AC12_RATINGS_A`, `DC19_RATINGS_A`.
- `Rpdu::new(kind, [PowerModuleKind; 6])` (slots 2..=7; DC1 feeds slots 2-4, DC2 slots 5-7), `.step(&RpduInput {dt_us, ac_v, dc1_v, dc2_v, backup_28v, comms_ok, commands, loads}) -> RpduOutput {controller_powered, default_active, statuses}`, `.set_default()`, `.program_phases()`, `.channel_mut()`, `.backup_output_v(addr)`, `.controller_powered()`.
- Fail-safe: default state applied after 3 s without comms, or immediately when controller power (28 V main or backup, 50 ms hold-up) is gone.

## pdu::spdu
- `SpduCardKind {Ac1, Ac2, Dc2, Dc4, Dc19}`, `.ratings_a()`, `SpduCard::new`, `Spdu::new()` (16 slots), `.install(slot, kind) -> bool`, `.channel_mut()`, `.step(&SpduInput {dt_us, ac_v, dc_v, commands, loads}) -> Vec<ChannelStatus>`.

## APPROX constants (not from facts)
I2T_CONSTANT_S 10, INSTANT_TRIP_MULTIPLE 15, COOLING_TAU_S 20, soft start 5 ms / limit curve, MIN_DC/AC_INPUT_V, arc thresholds and times, telemetry resolution 0.1 V / 0.01 A, breaker taus 30/60 s and THETA_AT_RATING 0.7, contactor pickup 18 V / dropout 5 V / operate 15 ms / release 8 ms / bounce edges / wear and weld constants, CONTROLLER_HOLDUP_US 50 ms, ELCC coil drive 28 V. Verified from facts: ELCU thresholds, channel mixes, SPDU cards, 3 s comms default, 6 P per C.

No APPROX_* constants exist in protection or pdu; elec3 research found nothing to replace there (all listed values above stay APPROX by design, none graded VERIFIED or DERIVED).
