use crate::quantities::*;

product!(Volts, Amps => Watts);
product!(Amps, Ohms => Volts);
product!(Volts, Siemens => Amps);
product!(Ohms, Siemens => Ratio);
product!(Watts, Seconds => Joules);
product!(Amps, Seconds => Coulombs);
product!(Farads, Volts => Coulombs);
product!(Ohms, Farads => Seconds);
product!(Ohms, Seconds => Henries);
product!(Hertz, Seconds => Ratio);
product!(KilogramsPerSecond, Seconds => Kilograms);
product!(WattsPerKelvin, Kelvin => Watts);
product!(JoulesPerKelvin, Kelvin => Joules);
product!(WattsPerKelvin, Seconds => JoulesPerKelvin);
square!(Meters => SquareMeters);
product!(SquareMeters, Meters => CubicMeters);
