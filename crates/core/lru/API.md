# b789-lru (lib `lru`) public API

All times are `u64` microseconds, all physical values `f64` SI (V, Hz, W, F, degC). No wall clock, no randomness, `BTreeMap` only. The caller steps the box at its own event times; power state is assumed to hold between steps.

## power
- `PowerKind { Dc28, Ac115 }`, `PowerInput { kind, voltage_v, frequency_hz, available }` with `dc28(v)`, `ac115(v, hz)`, `dead(kind)`.
- `InputState { Ok, UnderVoltage, FrequencyOutOfRange, Dead }`.
- `PowerConfig` (under-voltage thresholds, AC frequency window, capacitance, full/min voltage, efficiency, rated life hours, reference temp, recharge time), `Default`.
- `PowerStatus { Good, HoldUp, Lost }`, `PowerEvent { Gap, PowerLost, FrequencyExcursion, UnderVoltage }`, `PowerStats { dips, resets, longest_gap_us, frequency_excursions, undervoltage_events }`.
- `PowerStage`:
  - `new(cfg)`, `step(now_us, &[PowerInput], temp_c, load_w) -> PowerStatus` (diode-OR of inputs).
  - `full_holdup_us(load_w, temp_c)`, `holdup_remaining_us(now)`, `capacity_fraction()`, `aging_acceleration(temp_c)` (life halves per 10 degC).
  - `set_age_equivalent_hours`, `age_equivalent_hours`, `set_health`, `health`, `charge`, `bus_voltage_v`, `status`, `stats`, `events`, `clear_events`, `input_state`, `config`.

## clock
- `ClockConfig { offset_ppm, turnover_temp_c, temp_coeff_ppm_per_c2, aging_ppm_per_year }`.
- `BoxClock`: `new`, `power_up`, `step(now, temp_c)`, `rate_error_ppm(temp)`, `last_rate_ppm`, `local_time_us`, `error_us(true_elapsed)`, `resync(reference_us)`, `is_synced`, `set_age_hours`, `age_hours`, `set_stuck`.
- `OverflowAction { Wrap, FailBox, ResetBox }`.
- `PoweredCounter`: `new(width_bits, tick_us)`, `advance(dt_us) -> wraps`, `value`, `overflows`, `width_bits`, `tick_us`, `time_to_overflow_us`, `reset`, `set_ticks`. 31 bits at 10 ms is the 248-day case.

## nvm
- `NvmValue { U64, I64, F64, Text, Bytes }`, `Severity { Info, Minor, Major, Critical }`, `FaultRecord { time_us, code, severity, text }`, `NvmError { WornOut, Corrupt }`.
- `Nvm`: `new(endurance, fault_capacity)`, `write`, `read`, `read_u64`, `increment`, `write_count`, `total_writes`, `wear_fraction`, `keys`, `log_fault`, `faults`, `fault_count(code)`, `clear_faults`, `set_corrupt`, `is_corrupt`.

## identity
- `Identity { part_number, serial_number, software_pn, config_pn }`, `Identity::new`.
- `Expected { part_numbers, software_pn, config_pn }`, `matching(&Identity)`, `check(&Identity) -> Vec<IdentityMismatch>`.
- `IdentityMismatch { PartNumber, SoftwarePn, ConfigPn }`.

## app (partitions)
- `Signal { value, valid, timestamp_us }`.
- `AppIo`: `set_input`, `input`, `input_value` (valid only), `write`, `write_invalid`, `output`, `input_names`, `output_names`, `invalidate_outputs`.
- `trait HostedApp { name, period_us, budget_us, run(now_us, &mut AppIo), exec_time_us (default budget), reset }`.
- `Schedule { major_frame_us, windows }` with `new`, `window(app, offset, duration)`, `validate`, `windows_between`; `Window`, `ScheduleError { ZeroFrame, WindowOutsideFrame, WindowsOverlap }`.
- `OverrunAction { LogOnly, RestartPartition, ResetBox }`, `ExecRecord { time_us, slot, overrun }`.

## computer (the box)
- `Mode { Off, PowerOnReset, Booting(BootStage), Operational, Degraded, Failed, Bite, DataLoad }`, `BootStage { PowerGood, Post, ConfigCheck, SoftwareCheck, Init }`, `ResetCause`.
- `BootTimings` (`por_us`, `power_good_us`, `post_us`, `config_us`, `software_us`, `init_us`, `total_us()`), `LruConfig::new(identity)` with public fields (power, boot, watchdog timeout, NVM endurance, overrun action, clock, counter width/tick/overflow action, min start temp, BITE duration, load).
- `Lru`:
  - `new(cfg)`, `add_app(Box<dyn HostedApp>)`, `set_schedule`.
  - `step(now_us, &[PowerInput], temp_c)`.
  - `set_input(app, name, value, valid, ts)`, `output(app, name)` (valid=false whenever box not running, app not live or partition restarting; value frozen).
  - `mode`, `is_running`, `is_no_go`, `mismatches`, `boot_stage`, `identity`, `config`, `operational_since_us`.
  - `report_fault(code, severity, text, now)`, `clear_fault`, `active_faults`, `command_reset`, `start_bite`, `start_data_load(identity, duration, now)`, `resync_clock`.
  - `set_software_hang` (stops watchdog kicks), `set_post_fault`.
  - Accessors: `power`, `power_mut`, `nvm`, `nvm_mut`, `clock`, `clock_mut`, `counter`, `counter_mut`, `reset_count`, `last_reset_cause`, `trace`, `clear_trace`, `overruns(app)`, `runs(app)`.
- Fault codes: `FAULT_POWER_LOSS`, `FAULT_WATCHDOG`, `FAULT_OVERRUN`, `FAULT_CONFIG_MISMATCH`, `FAULT_POST`, `FAULT_COUNTER_OVERFLOW`, `FAULT_COMMANDED_RESET`, `FAULT_DATA_LOAD`.
