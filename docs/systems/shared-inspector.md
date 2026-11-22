# Shared: system inspector pages (sim tablet)

One page per system, all in the same plain style: greyscale monospace, with red used only for faults. Mockups are in `docs/mockups/`.

- **Runs only on the sim tablet (EFB).** No browser.
- **Drawn by our own renderer** (`render`, see [shared-architecture.md](shared-architecture.md)).
  - Offline: `sim-cli` shows the tablet screen in a desktop window.
  - In X-Plane: the same pixels become the tablet's cockpit texture.

## Every page has the same layout
- **Top line:** system name, sim time, flight phase, current view.
- **Left:** a list of every system page.
- **Centre:** a live diagram of the system.
- **Right:** the selected item. Its model state, its inputs and outputs (with source or consumer), wear, temperature, and the failure magnitudes currently set.
- **Bottom:** a table of every connection, line, station or link in that system.
- **Data sources:** pages show the truth (model state) and the sensed value side by side wherever a sensor exists. All values come from the live simulation, none are faked.

## Detail rules
- **Every individual part is drawn and selectable:** every valve, pump, check valve, jet pump, sensor, contactor and breaker. Nothing is grouped as "×8".
- **Engineering-drawing style:** true-scale 787-9 geometry (planform, spars, ribs), parts at their documented rib/station, AMM-style linework, callout bubbles, detail insets, title block and scale bar. Flow shown as moving dots.
- **Granularity:** sub-parts (motor, bearings, impeller, screen, seals, micro-switches), pipe segments, couplings, bonding, wires, connectors. Parts below document detail are rule-generated and marked `APPROX`. Targets: fuel ~2,400, aircraft ~40–60k.
- **Zoom levels** (electrical, and the same idea elsewhere):
  1. aircraft single-line;
  2. bus or panel;
  3. RPDU (a grid of every SSPC channel);
  4. one breaker: state, current, I²t versus the trip curve, arc-fault detector, instant trip, wire (gauge, length, bundle, zone, temperature), load, history, failures, current trace.
- **Valve detail:** type, location, power (RPDU channel), command source, position, micro-switches, motor current, last travel time, cycles, indication logic (switch light, EICAS, synoptic), history, failures. This includes the stuck micro-switch from SB 280015.
- **Colours:** dark background with white and grey. White for values and live parts, grey for labels and dead parts. Red only for faults.

## Pages and their diagrams
| Page | Diagram | Bottom table |
|---|---|---|
| Engines | Meridional section. Flow lines shaded by total temperature, width set by mass flow. Stations, spools, variable geometry, bleeds, gearbox offtake | Stations: Pt, Tt, W, Mach, sensor reading |
| Electrical | Single-line diagram. Sources, GCBs, bus ties, buses, ATRUs, ATUs, TRUs, batteries, CBIC/SSPC summary | Every feeder: device, state, V, A, wire gauge, length and zone, wire temperature, trip margin |
| Fuel | Plan view. Tanks, pumps, manifolds, crossfeed, spar valves, APU feed, jettison, NGS | Valves, pumps and lines: commanded vs actual, power, flow, pressure, temperature |
| Hydraulics | L/C/R schematic. Reservoirs, pumps, manifolds, accumulators, users, demand | Users: system, demand, pressure at the user, temperature, state |
| Air data, inertial, GNSS | Aircraft side view with probes, ports, vanes, ADMs, RDCs and the CDN. IRU/AHRU. GNSS antennas with signal paths to each satellite, plus a sky plot | Links: source, path, port/VL, rate, age, consumers, status |
| CDN network | (frame inspector) | Virtual link statistics, switch ports |
| All other systems | Same pattern: a schematic of the system's real parts | Its connections |

## Mockups
`fuel-drawing.png` (reference style), `engines.png`, `electrical.png`, `electrical-breaker.png`, `hydraulics.png`, `airdata.png`, `network-inspector.png`. Older pages get redrawn in the fuel-drawing style.
