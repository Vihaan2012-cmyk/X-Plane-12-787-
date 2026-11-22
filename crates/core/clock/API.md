# b789-clock public API (lib name `clock`)

## Time (`time.rs`)
- `SimTime(u64 us)`, `SimDuration(u64 us)`: Copy, Ord, Hash, Default, `Display` (`T+12.345678s` / `12.345678s`).
- `SimDuration`: `ZERO`, `MAX`, `from_micros/millis/secs/minutes/hours/days`, `from_secs_f64 -> Option`, `as_micros/millis/secs/subsec_micros/as_secs_f64`, `is_zero`, `checked_add/sub/mul/div/rem`, `saturating_add/sub/mul`, `div_duration -> Option<u64>`.
- `SimTime`: `ZERO`, `MAX`, `from_micros/millis/secs`, `from_secs_f64 -> Option`, `as_micros`, `as_secs_f64`, `since_start`, `checked_add/sub`, `checked_duration_since`, `saturating_add/sub/duration_since`, `signed_micros_since -> i128`.
- Operators `+`, `-`, `+=` are saturating.

## Scheduler (`scheduler.rs`)
- `Scheduler<E>`: `new`, `starting_at`, `now`, `len`, `is_empty`, `fired_count`.
- `schedule(at, priority: u32, event) -> EventId` (past times clamp to `now`), `schedule_after(delay, priority, event)`.
- `cancel(id) -> Option<E>`, `is_pending`, `scheduled_time`.
- `peek() -> Option<(SimTime, u32, EventId, &E)>`, `peek_time`.
- `pop_next() -> Option<Fired<E>>` (moves `now` to the event time), `pop_due(t)` (only events with time <= t; use for cascading events), `advance_to(t) -> Vec<Fired<E>>`, `advance_by(d)`, `finish_to(t)`.
- Order: time, then priority (lower first), then insertion sequence. `EventId::sequence()`.
- `Fired<E> { id, time, priority, event }`.

## Rate groups and major frames (`rate.rs`)
- `RateTable`: `register(period, phase, priority) -> Option<RateId>` (None for zero period), `period/phase/priority/next_index`, `tick_time(id, n)`, `first_tick_at_or_after(id, t)`, `count_in(id, from, to)` (half-open), `collect_until(t) -> Vec<Tick>` sorted by (time, priority, rate id) and advancing internal cursors, `seek(id, t)`.
- `Tick { time, priority, rate, index }`. Tick n is at `phase + n * period`.
- `MajorFrame::new(length, epoch, windows) -> Result<_, MajorFrameError>` (`ZeroLength`, `ZeroDuration`, `ExceedsFrame`, `Overlap`); windows are sorted by offset.
- `MajorFrame`: `length`, `epoch`, `windows`, `idle_time`, `frame_index(t)`, `frame_start(n)`, `window_at(t) -> Option<(frame, window index, &Window)>`, `events_in(from, to) -> Vec<WindowEvent>` over [from, to), ordered by time, Close before Open, window, frame.
- `Window { partition: u32, offset, duration }`, `WindowEvent { time, kind: WindowEventKind::{Close, Open}, partition, window, frame }`.

## Frame driver (`frame.rs`)
- `FrameDriver::new(budget)`, `starting_at(now, budget)`: sim time advances by the host dt with a 2^-32 us fixed-point remainder kept exactly. Time is never stretched or dropped.
- `step(dt_seconds, cost) -> Result<FrameReport, FrameError>` (`InvalidDt` for NaN, negative, infinite; `TimeOverflow`), `step_exact(SimDuration, cost)`, `drive(&mut Scheduler<E>, dt, cost) -> (FrameReport, Vec<Fired<E>>)`.
- `set_speed(u32)` multiplies each step (sim speed), `set_budget`, `budget`, `now`, `frames`, `fraction_units`, `overrun_count`, `worst_excess`, `last_overrun`.
- `FrameReport { frame, start, end, overrun: Option<Overrun> }` with `advanced()`; `Overrun { frame, cost, budget, excess }` when `cost > budget` (cost and budget are caller units, no wall clock).

## Powered-time counters (`counter.rs`)
- `CounterConfig { bits (1..=64), tick, mode: OverflowMode::{Wrap, Saturate} }`, presets `gcu_248_day()` (31 bits of 10 ms), `ccs_51_day()` (32 bits, tick derived from 51 days), `for_overflow_days(bits, days, mode)`.
- `PoweredCounter::new(config) -> Option<_>`, `power_on(at)`, `power_off(at)`, `reset(at)`, `restore(powered_total)`, `is_powered`, `powered_time(at)`, `raw_ticks(at)`, `value(at)`, `max_value`, `overflow_count(at)`, `has_overflowed(at)`, `overflow_period`, `time_to_next_overflow(at)`.

## Oscillator (`oscillator.rs`)
- `OscillatorConfig { initial_offset_ppb, turnover_celsius, temp_coeff_ppb_per_c2, aging_ppb_per_year, max_aging_ppb, initial_age }`, presets `ideal()`, `tcxo(initial_offset_ppb)`.
- `Oscillator::new(config, local_start)`: `advance(dt, temp_celsius)` (integer ppb accumulation with exact sub-us remainder, rate taken at interval start), `advance_unpowered(dt)` (ages only), `local_time`, `error_vs(reference)`, `frequency_error_ppb(temp)`, `temperature_ppb`, `aging_ppb`, `age`, `resync(reference)` (clears remainder), `sync_count`, `last_sync_step_us`.
- Frequency error = initial offset - coeff * (T - turnover)^2 + linear aging capped at `max_aging_ppb`.
