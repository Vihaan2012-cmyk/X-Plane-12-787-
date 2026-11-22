# b789-solver-thermal (lib solver_thermal)

No dependencies, no unsafe, no comments, deterministic (dense Cholesky, fixed ordering).

## Network (network.rs)
- `NodeId`, `LinkId` (opaque, `.index()`), `STEFAN_BOLTZMANN`
- `Network::new`
- Nodes: `add_node(capacity_j_k, temp_k)`, `add_boundary(temp_k)`, `node_count`, `is_boundary`, `temperature`, `set_temperature`, `capacity`, `set_capacity`, `source`, `set_source`, `add_source`
- Links: `add_conductance(a, b, g)`, `set_conductance`, `add_convection(a, b, area, length)`, `set_airflow(link, velocity, pressure)`, `add_radiation(a, b, emissivity, area)`, `link_count`, `link_conductance`
- Stepping: `step(dt) -> StepReport` (implicit backward Euler, unconditionally stable), `advance(duration, max_dt) -> StepReport` (summed)
- Energy: `stored_energy_j`, `total_source_j`, `total_in_j`, `total_out_j`
- `StepReport { dt_s, source_j, boundary_j, energy_in_j, energy_out_j, stored_delta_j, imbalance_j }` (imbalance = stored - source - boundary, roundoff only)
- `convection_coefficient(velocity, pressure, length, film_temp, delta_t)` forced (laminar/turbulent flat plate) blended with natural, scaled by pressure

## Electrical helpers (helpers.rs)
- `Conductor { Copper, Aluminium }`: `resistivity_20c`, `alpha_per_k` (0.00393 Cu), `density`, `specific_heat`, `resistivity_at`
- `conductor_resistance(conductor, length, area, temp)`
- `WireSpec`, `WireSegment::{add, node, temperature, resistance, heat_w, apply}`, `wire_ampacity(spec, ambient, max_temp)`
- `BundleSpec`, `WireBundle::{add, node, wires, temperature, apply_uniform, apply}` (shared bundle node)
- `ContactSpec`, `ContactorContact::{add, node, temperature, resistance, apply(current, coil_w)}`
- `BatterySpec`, `BatteryCell::{add, node, temperature, resistance_at, generation_w, apply}` (I2R(T) plus entropic term)
- `LruSpec`, `LruBox::{add, core, shell, core_temperature, set_dissipation, set_cooling_flow(kg/s), cooling_conductance}` (eps-NTU cooling air)
- `soak_temperature(initial, ambient, tau, t)`, `soak_time_constant(capacity, g)`

Helpers that depend on temperature (`apply`) set the node source from the start-of-step temperature; call before each `step`.
