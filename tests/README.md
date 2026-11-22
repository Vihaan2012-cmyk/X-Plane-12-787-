# Tests

- `scenarios/`: scripted runs for `sim-cli`, such as cold and dark, battery on, external power, and APU start.
- `facts/`: generated tests, one per testable fact in `refs/facts`, with its tolerance and citation.
- `golden/`: reference images for inspector pages and displays.

Unit tests live inside each crate. `cargo xtask check` runs everything.
- `facts-pending/`: facts cited from electrical sources that belong to other systems (fire, hydraulics, lights, APU, starting). Moved into `facts/` when that system is built.
