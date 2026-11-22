# Shared: architecture

## Shape
```
sim-core  (worker threads, one discrete-event clock at µs resolution)
 ├─ world:     atmosphere, terrain, GNSS satellites, radio paths
 ├─ networks:  electrical · fluid · thermal · shafts · engine gas path
 ├─ parts:     ~50k parts (temperature, wear, tolerance, failures, zone)
 ├─ computers: every box powers up, boots, runs its software partitions
 ├─ data:      CDN/AFDX · ARINC 429 · CAN · discrete wires, real bits
 └─ EFB:       sim tablet screen (inspector) + Boeing EFB, drawn by our renderer
      ⇅ truth in / outputs out (one interface)
 sim-cli   now:   stand-in truth, EFB screen in a desktop window, tests, replay
 xp-host   last:  X-Plane 12 fills the same interface; EFB screen becomes a cockpit texture
```

## Build order
- **Every system is built and tested offline in `sim-cli` first.** X-Plane is hooked up last.
- **Truth while offline** (`APPROX`, replaced by X-Plane at hookup):
  - scripted profiles (cold and dark, taxi, takeoff, climb, cruise, descent, landing), for open-loop tests;
  - a stand-in 6-DOF with 787-9 mass, inertia and geometry, for closed loops (fly-by-wire, autoflight, gear on the ground).
- **The truth interface is the one X-Plane fills later,** so the hookup is a swap, not a rewrite.

## Workspace (one Rust workspace, three layers of crates)
```
crates/
├─ core/       units · clock · store · bus · icd · lru · failures · persist · world · render
├─ solvers/    common (sparse linear algebra, integrators) · electrical · fluid · thermal · gas-path · mechanical
├─ systems/    27 crates, one per system (21-air … 24-electrical … 71-engines)
├─ tablet/     sim tablet: inspector, failures, ground services, settings
├─ sim-cli/    offline runner
└─ xp-host/    last
aircraft/      the aircraft as data (parts, wiring, plumbing; grade + citation per item)
tests/         scenarios · generated fact tests · golden images
tools/         xtask (checks) · pdf · facts · mockups
```
- **Package names** start with `b789-` (e.g. `b789-sys-electrical`). Folders keep the ATA number for ordering.
- **Dependencies point one way:** systems → solvers → core, tablet → core.
- **CCS-hosted software lives in its own system's crate** (e.g. load management and CBIC in `24-electrical`). It implements the `lru` hosted-app interface, and `42-ccs` runs it in a GPM partition by configuration, so no system depends on another.
- **Each system draws its own inspector pages** with `render`. The tablet only hosts them.
  - Solvers know nothing about the aircraft. They are pure physics and maths, tested on their own.
  - Systems describe parts and behaviour and hand their networks to the solvers.
  - Systems never depend on each other. Only `sim-cli` and `xp-host` see everything. A check enforces this on every build.
- **Core crates:**
  - `units`: typed SI quantities.
  - `clock`: the event scheduler.
  - `store`: part storage.
  - `bus`: data links with real packing and timing.
  - `icd`: generated from [shared-data.md](shared-data.md).
  - `lru`: the computer model, see [shared-computer.md](shared-computer.md).
  - `failures`, `persist`, `world`.
  - `render`: one 2D drawing API. CPU backend for the EFB, tests and golden images; GPU backend for the cockpit displays if the CPU can't hold their frame rate.
- **Boeing material stays in `refs/`.** `aircraft/` holds only numbers, names and citations (e.g. `eps24.txt:27610`).

## Rules enforced in code
1. **Systems never call each other.** They meet only at ports: wire, pipe, shaft, duct, data bus. A system crate cannot depend on another system crate.
2. **The aircraft is data, behaviour is code.**
   - Parts, wiring and plumbing are definition files generated from `refs/facts`.
   - Each part records its grade (`VERIFIED`, `DERIVED`, `STANDARD`, `APPROX`) and its source.
   - Code covers only the few hundred part types.
3. **Deterministic:**
   - fixed event order;
   - no unordered maps in sim state;
   - fixed floating-point sum order;
   - the same result on any thread count.

   This gives bit-exact replay.
4. **Idle parts sleep** until an event wakes them.
5. **Every build runs the checks:** fact tests, every failure observable, every output consumed, no part without a source.

## Electrical solver (full)
- **One network for the whole aircraft.** Every source, contactor, breaker, wire segment, connector contact, ground stud and load is a circuit element.
- **Method:** sparse modified nodal analysis.
  - AC: unbalanced three-phase phasors at each source's live frequency.
  - DC: 28 V and ±270 V networks.
  - Converters (ATRU, ATU, TRU, CMSC, SPU) are averaged models with losses and limits, coupling AC and DC.
- **Dynamics:**
  - VFSG, ASG and RAT generator with their exciters, and voltage regulation by GCU software;
  - battery cells with chemistry and temperature;
  - capacitor charging, motor and transformer inrush, contact bounce.
- **Stepping:**
  - 1 ms steps while anything changes; asleep when steady.
  - Refactorise only when the topology changes (a contactor or breaker moves).
  - Solve each isolated island on its own.
- **Protection reads the solved currents and voltages:** I²t, instant trip, arc fault, GCU differential, over/under voltage and frequency.
- **Not modelled:** waveforms (EMT). Harmonics appear only as a number, and only where a monitor reads them.

## EFB and inspector
- **The inspector is an app on the simulated sim tablet.** No browser. See [shared-inspector.md](shared-inspector.md) and [46-efb.md](46-efb.md).
- **Pages are drawn by `render`** into the tablet's screen buffer on a worker thread.
  - Offline: `sim-cli` shows that buffer in a desktop window.
  - In X-Plane: the same buffer becomes the tablet's cockpit texture.
- The same drawing code later draws the cockpit displays; only the backend may differ.

## Performance targets (`APPROX`)
- The sim core uses at most about a quarter of an 8-core CPU.
- X-Plane's main thread spends at most 2 ms per frame on us.
- If the sim falls behind it warns; it never stretches time ([shared-timing.md](shared-timing.md)).

## Recipe per system
Verified facts → plan → definition data → behaviour → tests from the facts → inspector pages → checks pass. Then the next system starts.
