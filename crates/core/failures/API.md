# b789-failures (lib `failures`) API

Modules: `rng`, `failure`, `registry`, `wear`. Everything re-exported at crate root. Time is u64 microseconds.

## failure
- `FailureMode { name, description, observables }`: `new()`, `described()`, `observable_via()`, `is_observable()`
- `Onset { Immediate, Ramp { duration_us }, Intermittent { seed, period_us, duty }, Latent { trigger } }`
- `Failure { mode, magnitude, onset, armed_at_us, triggered_at_us }`, `magnitude_at(now_us)`
- `FailureError::UnknownMode`
- `FailureSet` (BTreeMap by mode name)
  - `arm(mode, magnitude, onset, now_us)` (magnitude clamped 0-1), `arm_immediate()`, `arm_checked(registry, part_type, ...)`
  - `clear(mode)`, `clear_all()`, `trigger(trigger_name, now_us) -> count`
  - queries: `get()`, `is_armed()`, `len()`, `is_empty()`, `armed_modes()`, `magnitude(mode, now_us)`, `is_active()`, `active_modes(now_us)`, `any_active()`, `worst(now_us)`, `combined(now_us)`, `unknown_modes(registry, part_type)`

## registry
- `Registry`: `register(part_type, FailureMode)`, `add(part_type, mode, &[observables])`, `supports()`, `mode()`, `modes_for()`, `part_types()`, `total_modes()`, `all()`, `unobservable()`, `audit()`, `audit_known_observables(&[known])`
- `RegistryError::DuplicateMode`

## wear
- `arrhenius_factor(temp_k, ref_temp_k, activation_ev)`, `BOLTZMANN_EV_PER_K`, `US_PER_HOUR`
- `HazardModel { base_per_hour, wear_gain, wear_exponent, ref_temp_k, activation_ev }`
  - `new()`, `with_wear()`, `with_thermal()`, `hazard_per_hour(wear, temp_k)`, `failure_probability(wear, temp_k, hours)`
  - `time_to_failure_hours(seed, wear0, wear_rate_per_hour, temp_k, horizon_hours, step_hours) -> Option<f64>` (integrates the hazard against a seeded exponential target)
  - `time_to_failure_us(...) -> Option<u64>`
- `exponential_target(seed)`, `constant_hazard_failure_hours(seed, hazard)`, `failure_seed(serial, mode)`

## rng
- `SplitMix64`, `hash_at(seed, index) -> f64` (stateless, used for intermittent patterns), `hash_str()`, `seed_for()`
