quantity!(Volts);
quantity!(Amps);
quantity!(Ohms);
quantity!(Siemens);
quantity!(Watts);
quantity!(Vars);
quantity!(VoltAmps);
quantity!(Hertz);
quantity!(RadiansPerSecond);
quantity!(Rpm);
quantity!(Seconds);
quantity!(Kelvin);
quantity!(Joules);
quantity!(Coulombs);
quantity!(AmpHours);
quantity!(Farads);
quantity!(Henries);
quantity!(Kilograms);
quantity!(KilogramsPerSecond);
quantity!(Pascals);
quantity!(Meters);
quantity!(SquareMeters);
quantity!(CubicMeters);
quantity!(WattsPerKelvin);
quantity!(JoulesPerKelvin);
quantity!(Ratio);

impl Ratio {
    pub const ONE: Self = Self(1.0);
}
