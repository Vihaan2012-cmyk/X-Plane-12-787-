use crate::*;
use core::f64::consts::{PI, TAU};

const TOL: f64 = 1e-9;

macro_rules! ops_tests {
    ($module:ident, $t:ident) => {
        mod $module {
            use super::*;

            #[test]
            fn arithmetic() {
                let a = $t(6.0);
                let b = $t(2.0);
                assert_eq!(a + b, $t(8.0));
                assert_eq!(a - b, $t(4.0));
                assert_eq!(-a, $t(-6.0));
                assert_eq!(a * 2.0, $t(12.0));
                assert_eq!(2.0 * a, $t(12.0));
                assert_eq!(a / 2.0, $t(3.0));
                assert_eq!(a / b, 3.0);
                assert_eq!(a * Ratio(0.5), $t(3.0));
                let mut c = a;
                c += b;
                assert_eq!(c, $t(8.0));
                c -= b;
                assert_eq!(c, a);
                c *= 2.0;
                assert_eq!(c, $t(12.0));
                c /= 4.0;
                assert_eq!(c, $t(3.0));
            }

            #[test]
            fn helpers() {
                let a = $t(-3.0);
                assert_eq!(a.abs(), $t(3.0));
                assert_eq!(a.min($t(1.0)), a);
                assert_eq!(a.max($t(1.0)), $t(1.0));
                assert_eq!(a.clamp($t(-1.0), $t(1.0)), $t(-1.0));
                assert_eq!($t(5.0).clamp($t(-1.0), $t(1.0)), $t(1.0));
                assert_eq!(a.signum(), -1.0);
                assert_eq!($t::new(2.5).value(), 2.5);
                assert!($t::ZERO.is_zero());
                assert!(!a.is_zero());
                assert!(a.is_finite());
                assert!(!$t(f64::INFINITY).is_finite());
                assert_eq!($t(0.0).lerp($t(10.0), 0.25), $t(2.5));
                assert!($t(1.0).approx_eq($t(1.0 + 1e-12), 1e-9));
                assert_eq!($t::default(), $t::ZERO);
            }

            #[test]
            fn ordering_and_sum() {
                assert!($t(1.0) < $t(2.0));
                assert!($t(2.0) >= $t(2.0));
                assert_eq!(
                    $t(1.0).partial_cmp(&$t(2.0)),
                    Some(core::cmp::Ordering::Less)
                );
                let v = [$t(1.0), $t(2.0), $t(3.0)];
                let s: $t = v.iter().copied().sum();
                assert_eq!(s, $t(6.0));
                let s2: $t = v.iter().sum();
                assert_eq!(s2, $t(6.0));
                let empty: $t = core::iter::empty::<$t>().sum();
                assert_eq!(empty, $t::ZERO);
                assert!(format!("{}", $t(1.5)).starts_with("1.5 "));
            }
        }
    };
}

ops_tests!(volts, Volts);
ops_tests!(amps, Amps);
ops_tests!(ohms, Ohms);
ops_tests!(siemens, Siemens);
ops_tests!(watts, Watts);
ops_tests!(vars, Vars);
ops_tests!(volt_amps, VoltAmps);
ops_tests!(hertz, Hertz);
ops_tests!(radians_per_second, RadiansPerSecond);
ops_tests!(rpm, Rpm);
ops_tests!(seconds, Seconds);
ops_tests!(kelvin, Kelvin);
ops_tests!(joules, Joules);
ops_tests!(coulombs, Coulombs);
ops_tests!(amp_hours, AmpHours);
ops_tests!(farads, Farads);
ops_tests!(henries, Henries);
ops_tests!(kilograms, Kilograms);
ops_tests!(kilograms_per_second, KilogramsPerSecond);
ops_tests!(pascals, Pascals);
ops_tests!(meters, Meters);
ops_tests!(square_meters, SquareMeters);
ops_tests!(cubic_meters, CubicMeters);
ops_tests!(watts_per_kelvin, WattsPerKelvin);
ops_tests!(joules_per_kelvin, JoulesPerKelvin);
ops_tests!(ratio, Ratio);

#[test]
fn volts_amps_watts() {
    assert_eq!(Volts(28.0) * Amps(2.0), Watts(56.0));
    assert_eq!(Amps(2.0) * Volts(28.0), Watts(56.0));
    assert_eq!(Watts(56.0) / Volts(28.0), Amps(2.0));
    assert_eq!(Watts(56.0) / Amps(2.0), Volts(28.0));
}

#[test]
fn ohms_law() {
    assert_eq!(Volts(28.0) / Amps(2.0), Ohms(14.0));
    assert_eq!(Amps(2.0) * Ohms(14.0), Volts(28.0));
    assert_eq!(Ohms(14.0) * Amps(2.0), Volts(28.0));
    assert_eq!(Volts(28.0) / Ohms(14.0), Amps(2.0));
    assert_eq!(Volts(28.0) * Siemens(0.5), Amps(14.0));
    assert_eq!(Amps(14.0) / Siemens(0.5), Volts(28.0));
    assert_eq!(Amps(14.0) / Volts(28.0), Siemens(0.5));
    assert_eq!(Ohms(4.0) * Siemens(0.25), Ratio(1.0));
    assert_eq!(Ohms(4.0).recip(), Siemens(0.25));
    assert_eq!(Siemens(0.25).recip(), Ohms(4.0));
}

#[test]
fn energy_and_charge() {
    assert_eq!(Watts(100.0) * Seconds(3.0), Joules(300.0));
    assert_eq!(Seconds(3.0) * Watts(100.0), Joules(300.0));
    assert_eq!(Joules(300.0) / Seconds(3.0), Watts(100.0));
    assert_eq!(Joules(300.0) / Watts(100.0), Seconds(3.0));
    assert_eq!(Amps(2.0) * Seconds(5.0), Coulombs(10.0));
    assert_eq!(Coulombs(10.0) / Seconds(5.0), Amps(2.0));
    assert_eq!(Coulombs(10.0) / Amps(2.0), Seconds(5.0));
    assert_eq!(Farads(2.0) * Volts(5.0), Coulombs(10.0));
    assert_eq!(Coulombs(10.0) / Volts(5.0), Farads(2.0));
    assert_eq!(Coulombs(10.0) / Farads(2.0), Volts(5.0));
}

#[test]
fn time_constants() {
    assert_eq!(Ohms(1000.0) * Farads(1e-3), Seconds(1.0));
    assert_eq!(Seconds(1.0) / Ohms(1000.0), Farads(1e-3));
    assert_eq!(Henries(2.0) / Ohms(4.0), Seconds(0.5));
    assert_eq!(Ohms(4.0) * Seconds(0.5), Henries(2.0));
    assert_eq!(Hertz(50.0) * Seconds(2.0), Ratio(100.0));
    assert_eq!(Ratio(100.0) / Hertz(50.0), Seconds(2.0));
}

#[test]
fn amp_hours_and_coulombs() {
    assert_eq!(Coulombs::from(AmpHours(2.0)), Coulombs(7200.0));
    assert_eq!(AmpHours::from(Coulombs(7200.0)), AmpHours(2.0));
    assert_eq!(AmpHours(1.0).to_coulombs(), Coulombs(3600.0));
    assert_eq!(Coulombs(1800.0).to_amp_hours(), AmpHours(0.5));
    assert!(AmpHours::from_milliamp_hours(500.0).approx_eq(AmpHours(0.5), TOL));
    assert!(AmpHours(0.5).milliamp_hours() - 500.0 < TOL);
    assert_eq!(Amps(10.0) * Seconds::from_hours(1.0), Coulombs(36000.0));
    assert_eq!(
        (Amps(10.0) * Seconds::from_hours(2.0)).to_amp_hours(),
        AmpHours(20.0)
    );
}

#[test]
fn frequency_conversions() {
    assert!(RadiansPerSecond::from(Hertz(1.0)).approx_eq(RadiansPerSecond(TAU), TOL));
    assert!(Hertz::from(RadiansPerSecond(TAU)).approx_eq(Hertz(1.0), TOL));
    assert!(
        Hertz(400.0)
            .to_radians_per_second()
            .approx_eq(RadiansPerSecond(800.0 * PI), TOL)
    );
    assert!(
        RadiansPerSecond(800.0 * PI)
            .to_hertz()
            .approx_eq(Hertz(400.0), TOL)
    );
    assert_eq!(Hertz::from(Rpm(6000.0)), Hertz(100.0));
    assert_eq!(Rpm::from(Hertz(100.0)), Rpm(6000.0));
    assert_eq!(Rpm(60.0).to_hertz(), Hertz(1.0));
    assert_eq!(Hertz(1.0).to_rpm(), Rpm(60.0));
    assert!(RadiansPerSecond::from(Rpm(60.0)).approx_eq(RadiansPerSecond(TAU), TOL));
    assert!(Rpm::from(RadiansPerSecond(TAU)).approx_eq(Rpm(60.0), TOL));
    assert!(
        Rpm(60.0)
            .to_radians_per_second()
            .approx_eq(RadiansPerSecond(TAU), TOL)
    );
    assert!(RadiansPerSecond(TAU).to_rpm().approx_eq(Rpm(60.0), TOL));
    assert!(RadiansPerSecond(PI).degrees_per_second() - 180.0 < TOL);
    assert!(RadiansPerSecond::from_degrees_per_second(180.0).approx_eq(RadiansPerSecond(PI), TOL));
    let round = Rpm::from(RadiansPerSecond::from(Hertz::from(Rpm(12345.0))));
    assert!(round.approx_eq(Rpm(12345.0), 1e-6));
}

#[test]
fn period_and_frequency() {
    assert_eq!(Hertz(400.0).period(), Seconds(0.0025));
    assert_eq!(Seconds(0.0025).frequency(), Hertz(400.0));
}

#[test]
fn time_units() {
    assert!(Seconds::from_millis(250.0).approx_eq(Seconds(0.25), TOL));
    assert!(Seconds::from_micros(250.0).approx_eq(Seconds(0.00025), TOL));
    assert_eq!(Seconds::from_minutes(2.0), Seconds(120.0));
    assert_eq!(Seconds::from_hours(2.0), Seconds(7200.0));
    assert!((Seconds(0.25).millis() - 250.0).abs() < TOL);
    assert!((Seconds(0.00025).micros() - 250.0).abs() < TOL);
    assert_eq!(Seconds(120.0).minutes(), 2.0);
    assert_eq!(Seconds(7200.0).hours(), 2.0);
}

#[test]
fn kelvin_celsius() {
    assert!(Kelvin::from_celsius(0.0).approx_eq(Kelvin(273.15), TOL));
    assert!(Kelvin::from_celsius(-56.5).approx_eq(Kelvin(216.65), TOL));
    assert!((Kelvin(373.15).to_celsius() - 100.0).abs() < TOL);
    assert!((Kelvin::from_celsius(37.0).to_celsius() - 37.0).abs() < TOL);
    assert!(Kelvin::from_fahrenheit(32.0).approx_eq(Kelvin(273.15), TOL));
    assert!(Kelvin::from_fahrenheit(212.0).approx_eq(Kelvin(373.15), TOL));
    assert!((Kelvin(373.15).to_fahrenheit() - 212.0).abs() < 1e-9);
    assert_eq!(ZERO_CELSIUS_KELVIN, 273.15);
    assert!((Kelvin::from_celsius(30.0) - Kelvin::from_celsius(10.0)).approx_eq(Kelvin(20.0), TOL));
}

#[test]
fn thermal_relations() {
    assert_eq!(WattsPerKelvin(5.0) * Kelvin(20.0), Watts(100.0));
    assert_eq!(Kelvin(20.0) * WattsPerKelvin(5.0), Watts(100.0));
    assert_eq!(Watts(100.0) / Kelvin(20.0), WattsPerKelvin(5.0));
    assert_eq!(Watts(100.0) / WattsPerKelvin(5.0), Kelvin(20.0));
    assert_eq!(JoulesPerKelvin(500.0) * Kelvin(10.0), Joules(5000.0));
    assert_eq!(Joules(5000.0) / JoulesPerKelvin(500.0), Kelvin(10.0));
    assert_eq!(Joules(5000.0) / Kelvin(10.0), JoulesPerKelvin(500.0));
    assert_eq!(WattsPerKelvin(5.0) * Seconds(10.0), JoulesPerKelvin(50.0));
    assert_eq!(JoulesPerKelvin(50.0) / WattsPerKelvin(5.0), Seconds(10.0));
}

#[test]
fn geometry_relations() {
    assert_eq!(Meters(3.0) * Meters(4.0), SquareMeters(12.0));
    assert_eq!(SquareMeters(12.0) / Meters(4.0), Meters(3.0));
    assert_eq!(SquareMeters(12.0) * Meters(2.0), CubicMeters(24.0));
    assert_eq!(Meters(2.0) * SquareMeters(12.0), CubicMeters(24.0));
    assert_eq!(CubicMeters(24.0) / SquareMeters(12.0), Meters(2.0));
    assert_eq!(CubicMeters(24.0) / Meters(2.0), SquareMeters(12.0));
}

#[test]
fn mass_flow() {
    assert_eq!(KilogramsPerSecond(2.0) * Seconds(10.0), Kilograms(20.0));
    assert_eq!(Kilograms(20.0) / Seconds(10.0), KilogramsPerSecond(2.0));
    assert_eq!(Kilograms(20.0) / KilogramsPerSecond(2.0), Seconds(10.0));
    assert!(KilogramsPerSecond::from_kg_per_hour(3600.0).approx_eq(KilogramsPerSecond(1.0), TOL));
    assert!((KilogramsPerSecond(1.0).kg_per_hour() - 3600.0).abs() < TOL);
}

#[test]
fn power_triangle() {
    let s = apparent_power(Watts(800.0), Vars(600.0));
    assert!(s.approx_eq(VoltAmps(1000.0), TOL));
    assert!(power_factor(Watts(800.0), s).approx_eq(Ratio(0.8), TOL));
    assert_eq!(power_factor(Watts(0.0), VoltAmps(0.0)), Ratio(1.0));
    assert!(watts_from_apparent(s, Ratio(0.8)).approx_eq(Watts(800.0), TOL));
    assert!(reactive_power_from_pf(s, Ratio(0.8)).approx_eq(Vars(600.0), TOL));
    assert_eq!(VoltAmps(1000.0) * Ratio(0.8), VoltAmps(800.0));
    assert_eq!(VoltAmps::from_kva(90.0), VoltAmps(90_000.0));
    assert_eq!(VoltAmps(90_000.0).kva(), 90.0);
}

#[test]
fn si_prefix_conversions() {
    assert!(Volts::from_millivolts(250.0).approx_eq(Volts(0.25), TOL));
    assert!(Volts::from_kilovolts(0.115).approx_eq(Volts(115.0), TOL));
    assert!((Volts(0.25).millivolts() - 250.0).abs() < TOL);
    assert!((Volts(230.0).kilovolts() - 0.23).abs() < TOL);
    assert!(Amps::from_milliamps(100.0).approx_eq(Amps(0.1), TOL));
    assert!((Amps(0.1).milliamps() - 100.0).abs() < TOL);
    assert!(Ohms::from_milliohms(5.0).approx_eq(Ohms(0.005), TOL));
    assert!(Ohms::from_kilohms(2.0).approx_eq(Ohms(2000.0), TOL));
    assert!((Ohms(0.005).milliohms() - 5.0).abs() < TOL);
    assert_eq!(Watts::from_kilowatts(250.0), Watts(250_000.0));
    assert_eq!(Watts::from_megawatts(1.0), Watts(1_000_000.0));
    assert_eq!(Watts(250_000.0).kilowatts(), 250.0);
    assert_eq!(Watts(2_000_000.0).megawatts(), 2.0);
    assert!((Watts(745.699_871_582_270_2).horsepower() - 1.0).abs() < TOL);
    assert!(Farads::from_microfarads(10.0).approx_eq(Farads(1e-5), 1e-15));
    assert!((Farads(1e-5).microfarads() - 10.0).abs() < TOL);
    assert!(Henries::from_millihenries(10.0).approx_eq(Henries(0.01), 1e-15));
    assert!((Henries(0.01).millihenries() - 10.0).abs() < TOL);
}

#[test]
fn rms_peak_and_line_voltage() {
    assert!(
        Volts(115.0)
            .rms_to_peak()
            .approx_eq(Volts(162.634_559_672_906_2), 1e-6)
    );
    assert!(
        Volts(115.0)
            .rms_to_peak()
            .peak_to_rms()
            .approx_eq(Volts(115.0), TOL)
    );
    assert!(
        Volts(115.0)
            .line_to_line_from_phase()
            .approx_eq(Volts(115.0 * 3.0_f64.sqrt()), TOL)
    );
    assert!(
        Volts(200.0)
            .phase_from_line_to_line()
            .line_to_line_from_phase()
            .approx_eq(Volts(200.0), TOL)
    );
}

#[test]
fn energy_units() {
    assert_eq!(Joules::from_kilojoules(2.0), Joules(2000.0));
    assert_eq!(Joules::from_watt_hours(1.0), Joules(3600.0));
    assert_eq!(Joules::from_kilowatt_hours(1.0), Joules(3_600_000.0));
    assert_eq!(Joules(2000.0).kilojoules(), 2.0);
    assert_eq!(Joules(7200.0).watt_hours(), 2.0);
    assert_eq!(Joules(7_200_000.0).kilowatt_hours(), 2.0);
    assert_eq!(
        Watts(1000.0) * Seconds::from_hours(1.0),
        Joules::from_kilowatt_hours(1.0)
    );
}

#[test]
fn mass_and_pressure_units() {
    assert!(Kilograms::from_pounds(1.0).approx_eq(Kilograms(0.453_592_37), TOL));
    assert!((Kilograms(0.453_592_37).pounds() - 1.0).abs() < TOL);
    assert!(Pascals::from_kilopascals(101.325).approx_eq(Pascals(101_325.0), TOL));
    assert!(Pascals::from_hectopascals(1013.25).approx_eq(Pascals(101_325.0), TOL));
    assert!((Pascals(101_325.0).kilopascals() - 101.325).abs() < TOL);
    assert!((Pascals(101_325.0).hectopascals() - 1013.25).abs() < TOL);
    assert!((Pascals::from_psi(14.696).psi() - 14.696).abs() < TOL);
    assert!(Pascals::from_psi(14.696).approx_eq(Pascals(101_325.0), 10.0));
    assert!((Pascals::from_inhg(29.92).inhg() - 29.92).abs() < TOL);
    assert!(Pascals::from_inhg(29.92).approx_eq(Pascals(101_325.0), 10.0));
}

#[test]
fn length_and_volume_units() {
    assert!(Meters::from_feet(1000.0).approx_eq(Meters(304.8), TOL));
    assert!((Meters(304.8).feet() - 1000.0).abs() < TOL);
    assert_eq!(Meters::from_kilometers(2.0), Meters(2000.0));
    assert_eq!(Meters(2000.0).kilometers(), 2.0);
    assert_eq!(Meters::from_nautical_miles(1.0), Meters(1852.0));
    assert_eq!(Meters(3704.0).nautical_miles(), 2.0);
    assert!(Meters::from_millimeters(5.0).approx_eq(Meters(0.005), TOL));
    assert!((Meters(0.005).millimeters() - 5.0).abs() < TOL);
    assert!(CubicMeters::from_liters(1000.0).approx_eq(CubicMeters(1.0), TOL));
    assert!((CubicMeters(0.5).liters() - 500.0).abs() < TOL);
}

#[test]
fn ratio_helpers() {
    assert!(Ratio::from_percent(25.0).approx_eq(Ratio(0.25), TOL));
    assert!((Ratio(0.25).percent() - 25.0).abs() < TOL);
    assert_eq!(Ratio(0.25).complement(), Ratio(0.75));
    assert_eq!(Ratio(1.5).clamp01(), Ratio(1.0));
    assert_eq!(Ratio(-0.5).clamp01(), Ratio(0.0));
    assert_eq!(Ratio::ONE, Ratio(1.0));
    assert_eq!(Ratio(0.5) * Ratio(0.5), Ratio(0.25));
}

#[test]
fn approx_eq_free_function() {
    assert!(approx_eq(1.0, 1.0 + 1e-10, 1e-9));
    assert!(!approx_eq(1.0, 1.1, 1e-9));
}

#[test]
fn battery_discharge_scenario() {
    let capacity = AmpHours(65.0);
    let load = Watts(560.0) / Volts(28.0);
    let drawn: Coulombs = load * Seconds::from_hours(1.0);
    let remaining = Coulombs::from(capacity) - drawn;
    assert!(load.approx_eq(Amps(20.0), TOL));
    assert!(remaining.to_amp_hours().approx_eq(AmpHours(45.0), TOL));
}

#[test]
fn three_phase_line_power_formula() {
    let phase = Volts(115.0);
    let current = Amps(10.0);
    let i = ThreePhaseAmps::balanced(current, 0.0);
    let v = ThreePhaseVolts::balanced(phase, 0.0);
    let s = v.power(i);
    assert!(s.apparent().approx_eq(VoltAmps(3.0 * 115.0 * 10.0), TOL));
    let line = phase.line_to_line_from_phase();
    assert!((3.0_f64.sqrt() * line.0 * current.0 - s.apparent().0).abs() < 1e-6);
}
