# b789-units (lib `units`)

All values SI, `f64` newtypes with a public `.0`.

## Scalar quantities
Volts, Amps, Ohms, Siemens, Watts, Vars, VoltAmps, Hertz, RadiansPerSecond, Rpm, Seconds, Kelvin, Joules,
Coulombs, AmpHours, Farads, Henries, Kilograms, KilogramsPerSecond, Pascals, Meters, SquareMeters,
CubicMeters, WattsPerKelvin, JoulesPerKelvin, Ratio.

Every quantity has:
- `new`, `value`, `ZERO`, `abs`, `min`, `max`, `clamp`, `signum`, `is_finite`, `is_zero`, `lerp`, `approx_eq`
- `+ - neg`, `+= -=`, `* f64`, `f64 *`, `/ f64`, `*= /=` (f64), `T / T -> f64`, `T * Ratio -> T`
- `Sum` (owned and by reference), `PartialOrd`, `Default`, `Display`

## Cross-type products (each also gives the two inverse divisions)
- Volts*Amps=Watts, Amps*Ohms=Volts, Volts*Siemens=Amps, Ohms*Siemens=Ratio
- Watts*Seconds=Joules, Amps*Seconds=Coulombs, Farads*Volts=Coulombs
- Ohms*Farads=Seconds, Ohms*Seconds=Henries, Hertz*Seconds=Ratio
- KilogramsPerSecond*Seconds=Kilograms
- WattsPerKelvin*Kelvin=Watts, JoulesPerKelvin*Kelvin=Joules, WattsPerKelvin*Seconds=JoulesPerKelvin
- Meters*Meters=SquareMeters, SquareMeters*Meters=CubicMeters

## Conversions (`From` and named methods)
- Hertz, RadiansPerSecond, Rpm: all six directions; `to_hertz`, `to_rpm`, `to_radians_per_second`, `period`, `frequency`
- AmpHours <-> Coulombs (`to_coulombs`, `to_amp_hours`)
- Kelvin: `from_celsius`, `to_celsius`, `from_fahrenheit`, `to_fahrenheit`, `ZERO_CELSIUS_KELVIN`
- Ohms `recip` -> Siemens, Siemens `recip` -> Ohms
- Prefix and unit helpers: Volts (milli, kilo, rms_to_peak, peak_to_rms, line_to_line_from_phase,
  phase_from_line_to_line), Amps, Ohms, Watts (kilo, mega, horsepower), VoltAmps (kva), Seconds (millis, micros,
  minutes, hours), Joules (kJ, Wh, kWh), AmpHours (mAh), Farads (uF), Henries (mH), Kilograms (pounds),
  KilogramsPerSecond (kg/h), Pascals (kPa, hPa, psi, inHg), Meters (feet, km, nm, mm), CubicMeters (liters),
  Ratio (percent, complement, clamp01, ONE)
- Power triangle: `apparent_power`, `power_factor`, `watts_from_apparent`, `reactive_power_from_pf`

## Complex
`Complex { re, im }`: `new`, `from_polar`, `from_real`, `from_imag`, `ZERO`, `ONE`, `I`, `magnitude`,
`norm_sqr`, `angle`, `to_polar`, `conj`, `recip`, `exp`, `sqrt`, `powi`, `rotate`, `is_finite`, `approx_eq`.
Ops with Complex and f64 (`+ - * /`, f64 on either side, assign forms), `Neg`, `Sum`, `From<f64>`,
`From<(f64, f64)>`, `Display`.

## Phasors
`ComplexVolts`, `ComplexAmps`, `Impedance` (R + jX), `Admittance`: tuple wrappers over `Complex` with
`new`, `from_polar`, `from_real`, `from_imag`, `real`, `imag`, `magnitude`, `angle`, `to_polar`, `conj`,
`rotate`, `approx_eq`, `+ - neg`, `* / f64`, `* / Complex`, `Sum`.
- `ComplexAmps * Impedance = ComplexVolts`, `ComplexVolts / Impedance = ComplexAmps`,
  `ComplexVolts / ComplexAmps = Impedance`, `ComplexVolts * Admittance = ComplexAmps`
- `Impedance`: `resistor`, `inductor(L, f)`, `capacitor(C, f)`, `resistance`, `reactance`, `admittance`,
  `series`, `parallel`
- `Admittance`: `conductance`, `susceptance`, `impedance`
- `ComplexVolts::power(ComplexAmps) -> ComplexPower` (S = V * conj(I)), also `ComplexAmps::power(V)`
- `ComplexPower`: `from_pq`, `from_apparent_and_factor`, `active`, `reactive`, `apparent`, `power_factor`,
  `phase_angle`, `+ - neg`, `* / f64`, `Sum`

## Three phase
- `alpha()` (1 at 120 degrees)
- `fortescue(a, b, c) -> SequenceComponents { zero, positive, negative }`, `inverse_fortescue`
- `SequenceComponents::{unbalance_ratio, zero_ratio, approx_eq}`
- `ThreePhaseVolts`, `ThreePhaseAmps` ({a, b, c}): `new`, `balanced(magnitude, angle)` (a, a-120, a+120),
  `balanced_negative_sequence`, `magnitudes`, `sequence() -> SequenceVolts/SequenceAmps`, `sum`, `rotate`,
  `approx_eq`
- `ThreePhaseVolts::{line_to_line, power(ThreePhaseAmps)}`, `ThreePhaseAmps::power`
- `SequenceVolts`, `SequenceAmps`: `new`, `from_positive`, `phases()`, `unbalance_ratio`, `approx_eq`

## Misc
`approx_eq(a, b, tolerance)` on f64.
