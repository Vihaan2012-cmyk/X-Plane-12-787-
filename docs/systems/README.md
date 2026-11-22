# 787-9 (Trent 1000) systems plans

One plan per system. Code lives in `crates/` (skeleton only so far); `cargo xtask check` runs every check.

## Decisions
- **Aircraft:** 787-9 with **Trent 1000-J3** (LOT FCOM Performance Dispatch; limits from EASA TCDS E.036), matched to LOT Polish Airlines' configuration so it agrees with the LOT FCOM Rev 12. GEnx was dropped because the best documents are Trent.
- **Simulator:** X-Plane 12 on Windows. The plugin layer is thin, so Mac, Linux and X-Plane 11 can be added later.
- **Language:** Rust. Nothing is reused from the A380 port or FlyByWire.
- **Flight model:** X-Plane's. We drive the surfaces, thrust, weight and CG, and the extra ice and damage forces.
- **Core:** one solver per physical network, one crate per system, shared entity storage, and joint solves where coupling is tight. See [shared-architecture.md](shared-architecture.md).
- **Build order:** every system is finished offline in `sim-cli` first, one system at a time. X-Plane is hooked up last.
- **Inspector:** only on the sim tablet (EFB), drawn by our own renderer.
- **HUD:** no HUD symbology. The HUD units exist only as hardware (power loads, breakers, BITE, status messages) so electrical and alerting stay complete.
- **Depth floor:** at least the A380 port's `deep/` layer, everywhere.

## Rules
1. **Causal.** Displays read sensors, sensors read physical state, and state comes from its sources. Nothing is scripted.
2. **Failures** apply to parts, as 0–1 values. Never to outcomes.
3. **Every part** carries temperature, wear, airframe tolerance, failures and zone.
4. **Model the source.** Trace every input back to its physical origin and model that.
5. **Stop rule.** If no instrument, sensor or pilot could ever detect the difference, leave it out.
6. **Truth vs databases.** The world is what X-Plane draws. The aircraft's databases (terrain, magnetic variation, navigation) are separate, dated and imperfect copies.
7. **Sources.** Every constant cites a document and page. Unconfirmed values are marked **(verify)** here and `APPROX` in code.
8. **Validation, Level D style.** Every behaviour is tested against real data within a tolerance, automatically, on every build.
9. **Audits.** Every failure must change something observable, and every output must have a consumer.
10. **Full data movement.** Systems exchange data only over the aircraft's real paths: CDN frames, ARINC 429, CAN, and discrete or analog wires. This is enforced in code. See [42-ccs-cdn.md](42-ccs-cdn.md).
11. **Exact timing.** One discrete-event clock with microsecond resolution. Internal timing never depends on X-Plane's frame rate. Time is never stretched. See [shared-timing.md](shared-timing.md).

## Plan sections
Hardware, physics, software behaviour, links, failures, temperature/wear/airframe tolerances, tests, documents.

## Plans
| # | System | Plan | Status |
|---|---|---|---|
| S0 | Shared: architecture | [shared-architecture.md](shared-architecture.md) | Draft. Workspace created (skeleton, layering check) |
| S1 | Shared: radio propagation | [shared-radio.md](shared-radio.md) | Draft |
| S2 | Shared: computer (LRU) model | [shared-computer.md](shared-computer.md) | Draft |
| S3 | Shared: data dictionary (ICD) | [shared-data.md](shared-data.md) | Draft |
| S4 | Inventory: every computer | [computers.md](computers.md) | 286 boxes: 190 OK, 25 confirmed by a quote, 69 partial, 0 flagged |
| S6 | Data flows (seeds the ICD) | [data-flows.md](data-flows.md) | About 220 flows, fixed and rechecked |
| S5 | Shared: timing | [shared-timing.md](shared-timing.md) | Draft |
| S7 | Shared: system inspector pages | [shared-inspector.md](shared-inspector.md) | Draft + mockups |
| 1 | Inertial (2 IRU + 2 AHRU) and air data (ADRS) | [34-adirs.md](34-adirs.md) | Draft, corrected against the LOT FCOM |
| 2 | Electrical, including every circuit breaker | [24-electrical.md](24-electrical.md) | First pass |
| 3 | Central computers and data network (CCS/CDN) | [42-ccs-cdn.md](42-ccs-cdn.md) | Draft |
| 4 | Engines (Trent 1000-J3) | [71-engines.md](71-engines.md) | Draft, from 711 verified facts |
| 5 | APU | | |
| 6 | Fuel, nitrogen generation | | |
| 7 | Hydraulics | | |
| 8 | Flight controls | | |
| 9 | Gear, brakes, steering | | |
| 10 | Air conditioning, pressurisation, cooling | | |
| 11 | Ice and rain | | |
| 12 | Fire protection | | |
| 13a | GNSS | [34-gnss.md](34-gnss.md) | Draft |
| 13b | ILS, VOR/DME, markers, radio altimeter | | |
| 14 | Transponder/ADS-B, TCAS, terrain warning, weather radar | | |
| 15 | Communications: VHF, HF, SATCOM | | |
| 16 | Displays and crew alerting | | |
| 17 | Autoflight and FMS | | |
| 18a | EFB (Boeing installed) and the sim tablet | [46-efb.md](46-efb.md) | Draft |
| 18b | Central maintenance (CMCF) | | |
| 19a | Exterior lights | [33-exterior-lights.md](33-exterior-lights.md) | Draft |
| 19b | Interior lights, oxygen, water and waste, doors | | |
| 20 | Environment, airframe, persistence | | |
