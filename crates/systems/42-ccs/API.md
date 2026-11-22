# b789-sys-ccs (lib `sys_ccs`) public API

CCS and CDN per the 42 plan (steps 1–7). All times are `u64` microseconds. Deterministic; `seed` sets airframe tolerances.

## Data (`aircraft/42-ccs`)
- `hosted-apps.csv`: application → GPMs (L1–R8).
- `rdcs.csv`: RDC → primary and secondary bus, source and breaker.
- `mmel.csv`: dispatch items (not loaded yet).

## Ccs
- `Ccs::new(data_dir, seed)`, `Ccs::from_data(CcsData, seed)`.
- `step(dt_us, &CcsInputs)`:
  - `bus_v` keyed by `CIB`, `FIB`, `DC28_L`, `DC28_R`, `HBB_MAIN`;
  - `on_ground`, `fuel_run[2]`, `fuel_data_valid`;
  - `cooling_supply_ok`, `cooling_air_c`, `ambient_c`.
- `loads_w() -> Vec<(name, W)>`: the 6 electrical load names (`CABINET CCR L PRI/SEC/BAT`, `CB CE4221821 CABINET CCR R PRI`, `CB CE4221822 CABINET CCR R SEC`, `CABINET CCR R BAT`). The draw shifts to `BAT` during HBB hold-up.
- `press_reset(side)`, `set_gpm_failed("L4", bool)`, `set_rdc_failed("RDC 19", bool)`.
- `gpm_running(name)`, `app_available(app)`, `status_messages()`, `inhibit()`.
- `value(name) -> Option<CcsValue>`:
  - `ccs.ccr.{l,r}.{state, mode, powered, on_battery, hosted_ready, displays_ready, temp_c, fan_on, valve_closed, power_w, boots, nvm_saves, hard_offs, startup_complete, resets}`;
  - `ccs.gpm.{l1..r8}.running`;
  - `ccs.app.<slug>.available`;
  - `ccs.rdc.<n>.{running, powered, on_secondary}`, `ccs.rdc.running_count`;
  - `ccs.status.<message_slug>`.

## Modules
- `pcm`: `Pcm::step(now, primary_v, battery_v)` switches to the HBB below 18 V, then saves and shuts down after 11 s. `press_reset` gives a hard off with no save.
- `cabinet`: `Cabinet` with 2 PCMs and 8 GPMs.
  - Start mode: uninhibited (full PBIT, seeded 120–180 s) or inhibited (displays 60 s, hosted functions seeded 55–75 s).
  - Thermal: LPS and AAF drive the fan and valve; GPM overtemp stop; ground overtemp shutdown. `APPROX` values.

## Host coupling
`sim-cli` target `aircraft` (feature `aircraft`) steps electrical and CCS in 100 ms slices. It feeds bus voltages and fuel switches in, writes CCR loads back with `Electrical::set_external_load`, and routes the `CCR_RESET_L/R`, `GPM_<n>` and `RDC <n>` actions.

## Network (`cdn`, `network`, `icd`)
- `network::build(rdcs, hosted, seed) -> Cdn`: devices, links and the generated VL table.
- `Cdn`: `add_device`, `connect`, `add_vl`, `build_routes`, `publish`, `unpublish`, `run_until`, `rx(dev, vl)`, counters (`police_drops`, `queue_drops`, link `crc_errors`, `unknown_vl_drops`).
- `icd::Dataset`: `new(name, version, params)`, `encode_at(values, status, ts_ms)`, `decode`, `status`, `version_of`, `timestamp_of`, `stamp`; `crc32`.

## Software (`gpm`, `platform`)
- `GpmSoftware`: partitions (6 IS plus hosted apps), `running(part)`, `lane_failed`, `data_load`, health events.
- `Ccs`:
  - `instance_live(app, gpm)`, `app_available(app)`, `cas_live_host()`.
  - `set_elec_icd(names)`, `encode_elec(active)`, `cas_elec_messages()`.
  - `crew_status_messages()`, `component_faults()`, `maint_page(n)`.
  - `set_powered_days(d)`, `monitor_lost()`, `read(dest, vl) -> (data, age_us, valid)`.

## Failures (`faults`)
- `CATALOG` of 9 part types and 35 modes; `instances(ccs, type)`, `part_type_of`, `arm(ccs, part, mode, on)`, `failure_points(ccs)`.

## More signals
- `ccs.cdn.*`, `ccs.switch.<name>.up`;
- `ccs.data.<vl>.<dest>.{valid, age_ms}`, `ccs.rdc.<n>.ifz.<k>.valid`;
- `ccs.gpm.<g>.partition.<p>.running`, `ccs.gpm.<g>.{single_lane, data_load}`;
- `ccs.cas.{available, elec_data_valid, count}`, `ccs.cas.msg.<slug>`;
- `ccs.crew_status.<slug>`, `ccs.maint.page<n>.<item>`;
- `ccs.monitor_lost`, `ccs.powered_days`, `ccs.gnd_test`, `ccs.param.<name>`.
