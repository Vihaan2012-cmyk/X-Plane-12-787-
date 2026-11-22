# solver_electrical (b789-solver-electrical)

## Network
- `NodeId(usize)`, `ElementId(usize)`, `NodeKind { Dc, Ac3 }` (Ac3 = phases A, B, C plus neutral N; index 0..3).
- `Network`: `new`, `add_node(kind)`, `add_element(Element) -> Result<ElementId, NetworkError>`, `set_element(id, Element)`, `set_switch(id, closed)`, `element`, `elements`, `node_count`, `element_count`, `node_kind`, `topology_version`.
  - Topology version bumps on add, switch change, or a changed connection; value-only changes do not.
- `NetworkError { UnknownNode, UnknownElement, KindMismatch }`.
- `LoadKind { Impedance { z }, Current { amps, power_factor, v_nominal }, Power { p, q, v_nominal } }`; for Ac3 it is per phase (phase-to-neutral rms).
- `Element`:
  - `Resistor`, `Impedance { z_phase, z_neutral }`, `Switch { closed, r_closed }`
  - `DcSource { e, r }`, `AcSource { emf, angle, r, x, frequency_hz, r_neutral_ground }`
  - `DcCurrentSource`, `AcCurrentSource`
  - `DcLoad { kind }`, `AcLoad { phases: [LoadKind; 3] }`, `Element::balanced_ac_load(kind, node)`
  - `Rectifier { ac, pos, neg: Option, ratio, r_out, efficiency, ac_v_nominal }`: Vdc = ratio x mean line-line rms minus r_out drop; with `neg` the rails are +/- half; input is balanced constant power Pout / efficiency.
  - `Autotransformer { input, output, ratio, r_leak, x_leak, efficiency, in_v_nominal, r_neutral_ground }`: per-phase EMF = ratio x input phase phasor; input draws S_out / efficiency.
  - `power_sense()` -> `PowerSense { Absorbed, Delivered }`.

## Solver
- `SolverOptions { gmin, low_voltage_fraction, max_inner, max_outer, inner_tolerance, outer_tolerance }` with `Default`.
- `Solver::new(opts)` / `Default`; `solve(&mut self, &Network) -> Result<Solution, SolveError>`.
  - Islands = connected components through resistors, impedances and closed switches.
  - Islands with no live source solve to zero.
  - AC islands solve before DC; outer loop iterates converter powers and EMFs to convergence.
  - Constant-power and constant-current loads iterate inner on a fixed matrix; below `low_voltage_fraction` x v_nominal they fall back to constant impedance.
  - Symbolic factorisation is cached per island and reused; value changes only refactor.
- `Solution`: `dc_voltage`, `phase_voltages` ([A, B, C, N] phasors vs chassis), `line_voltage(node, a, b)`, `element_currents` ([A, B, C, neutral-return sum]; DC in slot 0), `element_power` (complex, sign per `power_sense`), `converter_output_power`, `island_of`, `island`, `island_count`, `frequency_of`, `is_live`, `report`.
- `IslandInfo { nodes, kind, live, frequency_hz, frequency_conflict }`.
- `ConvergenceReport { converged, outer_iterations, max_inner_iterations, outer_residual, inner_residual, islands_refactored, islands_factored, topology_rebuilt }`.
- `SolveError::Singular { island }`.
- Re-exports `Complex`.
