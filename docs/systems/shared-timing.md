# Shared: timing

How time works everywhere in the simulation.
Status: draft. **(verify)** means a source document is needed.

## Core: a discrete-event simulation
- **One sim clock at microsecond resolution.** Everything is an event with an exact timestamp:
  - frames sent, hops, arrivals;
  - partition windows;
  - sensor samples;
  - ARINC 429 words and CAN frames;
  - contactor operations;
  - physics substeps.
- **Fixed ordering.** Events are ordered by time, then priority, then id. The same inputs always give the same result.
- **Independent of X-Plane's frame rate.** Each X-Plane frame advances the sim clock by that frame's dt and runs every event inside it at its exact time. Internal timing is identical at 20 fps or 120 fps.

## Coupling to X-Plane
- **Sim time follows X-Plane's sim time.** Pause stops it, and sim-speed acceleration (up to 16×) runs proportionally more events.
- **Truth inputs** (motion, atmosphere, controls) are sampled at each frame and interpolated to the exact time of each event.
- **Outputs** (surfaces, thrust, lights, displays) are written at the frame's end. That adds one frame of delay, which is small next to the real systems' own latency.
- **Threading.** The event simulation runs on worker threads, so X-Plane's frame is never stalled. State is exchanged with X-Plane through a double buffer at frame boundaries.

## Rates, per box and per function
- **Rate groups and partition major frames** use real rates where documented (control laws, displays, sensors) **(verify)**, otherwise engineered and marked `APPROX`.
- **Jitter** comes from scheduling and the network, not added as random noise.
- **Physics substeps by domain:** fast for fly-by-wire and hydraulic or electrical transients, slow for heat. Each substep is an event on the same timeline.

## Network and bus timing
- CDN: BAG shaping, store-and-forward time per hop, queueing ([42-ccs-cdn.md](42-ccs-cdn.md)).
- ARINC 429: real bit times, word gaps, word rates.
- CAN: arbitration and bit timing.
- RDC ADC sample instants, and filter delay.

## Clocks
- **Every box has its own oscillator,** with drift that depends on temperature and age ([shared-computer.md](shared-computer.md)).
- **Synchronisation:** time comes from GPS UTC to the CCS, then to the boxes. The sync method and intervals **(verify)**. Between syncs, boxes drift apart.
- **Flight deck clocks (LOT FCOM 10.10.65, 10.20.16):**
  - One clock on each AUX display.
  - Each one derives UTC time and date from GPS independently, so the seconds can differ slightly between the two. That falls out of our model.
  - Elapsed time: reset at power-up, starts at weight off wheels, stops at weight on wheels + 30 s, and resets on the ground when V-speeds are entered.
  - Each pilot has an independent chronograph.
- **Timestamps, leap seconds, date rollover.**
- **Powered-time counters** (248-day, 51-day) count sim time while the box is powered and are kept between sessions.

## Time between sessions
- **The gap between sessions counts.** The sim date and time gap, or the real-world gap (setting), is applied as elapsed time:
  - cooldown and soak, solved analytically for that time;
  - battery self-discharge;
  - almanac aging.

## Determinism and replay
- **Record:** every X-Plane input and crew input, with timestamps.
- **Replay** is bit-exact, which is what makes debugging and regression tests possible.

## Performance budget
- **X-Plane main thread:** about 1–2 ms per frame, just the exchange.
- **Event load:**
  - Expected at 1×: hundreds of thousands of events per second.
  - At 16×: it scales up, so it's measured and profiled.
- **Never stretch time.** If the simulation falls behind, show a warning and log it. Silently slowing the clock is not allowed.

## Validation
- **Event timelines checked against video:** power-up, APU and engine start, display boot. Timestamps come from frames pulled with ffmpeg, and each event must land inside its tolerance.
- **Latency budgets:** every message stays inside its stated or engineered bound.

## Inspector
- A timeline of events per box.
- Latency histograms per VL.
- Partition window maps.
- Clock offsets between boxes.

## Tests
1. The same session at 20 fps and at 120 fps gives identical event logs.
2. Pausing freezes everything. A 4× sim speed gives the same results at 4× the rate.
3. A replay is bit-exact to the recording.
4. Boot and start timelines fall inside the video tolerances.
5. Box clocks drift apart without GPS and re-converge after sync.
6. 51 days of powered sim time across many sessions triggers the CCS bug.
