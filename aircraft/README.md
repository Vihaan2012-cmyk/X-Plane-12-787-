# Aircraft data

The aircraft as data: parts, wiring, plumbing, ratings and locations. The code in `crates/` reads it; nothing here is code.

- **Every item records `grade` and `source`.**
  - Grades: `VERIFIED`, `DERIVED`, `STANDARD`, `APPROX`.
  - Sources: a citation such as `eps24.txt:27610`, or the name of the rule that generated it.
- **No Boeing text.** Only numbers, names and citations. Quotes stay in `refs/`.
- **Generated from `refs/facts` by `tools/`.** Hand-made additions go in separate `*.manual.*` files so regenerating never wipes them.
- **The file format is decided in the electrical plan.**

## Folders
- `shared/`: zones, locations (from the locator), wire types, materials.
- `24-electrical/`: sources, buses, contactors, breakers, wires, loads.
