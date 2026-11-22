# Tests

- `scenarios/`: scripted runs for `sim-cli`, such as cold and dark, battery on, external power, and APU start.
- `facts/`: generated tests, one per testable fact in `refs/facts`, with its tolerance and citation.
- `golden/`: reference images for inspector pages and displays.

Unit tests live inside each crate. `cargo xtask check` runs everything.
