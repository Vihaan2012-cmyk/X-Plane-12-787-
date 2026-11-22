# b789-persist (lib `persist`)

Airframe state kept between sessions. No serde, no deps, no wall clock: the host passes all times in.

## File format (schema v2, `CURRENT_VERSION`)
- Line 1: `B789PERSIST v<N>`.
- Per section: `[name] crc=<fnv1a64 hex16 of body>` then sorted `key = value` lines.
- Last line: `[end] sections=<n> crc=<fnv1a64 hex16 of everything before it>`.
- Values: `f:<bits hex16>(<decimal hint>)`, `u:`, `i:`, `b:0|1`, `s:"..."`, `x:<hex>`, `a:[v,v]`.
- Names: `[A-Za-z0-9_./:-]+`.
- Section layout: `airframe`, `part/<name>`, `battery/<name>`, `breakers`, anything else kept in `Airframe::extra` (per-system sections).

## Types
- `Value` { F64, U64, I64, Bool, Str, Bytes, Array }: `encode()`, `decode(&str)`, `kind()`; `PartialEq` compares f64 by bits.
- `Section`: `set/get/remove/contains`, typed getters `f64 u64 i64 bool str bytes array f64_array f64_or`.
- `Document { version, sections }`: `new(v)`, `current()`, `section(_mut)`, `to_text()`, `from_text()`.
- `PersistError`: Io, NotFound, BadHeader, UnsupportedVersion, Truncated, Trailing, Syntax, SectionChecksum, FileChecksum, SectionCount, DuplicateSection, DuplicateKey, InvalidName, Missing, WrongType, Migration.
- `Airframe { identity, settings, stamp, parts, batteries, breakers, extra }`: `new(tail, serial)`, `to_document()`, `from_document()`.
- `Identity { tail, serial, model, options }`, `Settings { gap_mode, max_gap_s }`, `GapMode { Sim, Real }`, `SessionClock { sim_s, real_s }`.
- `PartRecord { wear, run_s, cycles, powered_s, unpowered_s, soak_c, soak_tau_s, nvm, counters, faults }`, `Counter { value, powered_only }`, `FaultEntry { t_s, code, flags }`.
- `BatteryRecord { soc, cell_soc, cell_health, cycles, soak_c, soak_tau_s, self_discharge_per_day }`.
- `BreakerRecord { state, locked, tag }`, `BreakerState { Open, Closed }`.

## Functions
- `fnv1a64(&[u8]) -> u64`, `valid_name(&str) -> bool`.
- `migrate_to_current(&mut Document) -> Result<u32>`, `migrate_with(doc, table, target)`, `MIGRATIONS` (v1 to v2: hours to seconds, soc_pct to fraction, temp_c to soak_c, new defaults).
- `decode_airframe(&[u8]) -> Result<(Airframe, migration_steps)>`.
- `Airframe::begin_session(now, ambient_c) -> GapReport`: elapsed from sim or real clock (setting), clamped to `max_gap_s`, negative or NaN gives zero.
  - Parts: `unpowered_s` grows, soak temperature relaxes analytically to ambient, counters with `powered_only == false` advance, powered-only counters and `powered_s` and `run_s` do not.
  - Batteries: temperature soak plus Arrhenius-style self-discharge (rate doubles per 10 C above 20 C), applied to pack and every cell.
- `Airframe::end_session(now)`, `Airframe::elapsed_since_stamp(now)`.
- `soak_toward(temp, ambient, tau, dt)`, `apply_part_gap`, `apply_battery_gap`.

## Storage
- `trait Storage { read, write, rename, copy, remove }`; `DiskStorage` (fsync), `MemStorage` (fault injection: `fail_at`, `torn_write`).
- `SaveStore<S>`: `new(storage, name)`, `save(&Airframe)`, `save_text(bytes)`, `load() -> Result<Option<Loaded>>`.
  - Save: write `.tmp`, read back and verify, copy the current file to `.prev` only if it still decodes, rename `.tmp` over main. Failure at any step leaves main intact.
  - Load: main, else `.prev` (`LoadSource::Previous`, `primary_error` kept), `Ok(None)` if no save exists, error if all are corrupt.
