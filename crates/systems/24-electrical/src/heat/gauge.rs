pub const MAX_CONDUCTOR_K: f64 = 473.15;
pub const BUNDLE_DERATE: f64 = 0.65;
pub const AMBIENT_REF_K: f64 = 293.15;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gauge {
    pub awg: &'static str,
    pub area_m2: f64,
    pub free_air_a: f64,
}

pub const GAUGES: [Gauge; 17] = [
    Gauge {
        awg: "24",
        area_m2: 0.205e-6,
        free_air_a: 13.0,
    },
    Gauge {
        awg: "22",
        area_m2: 0.324e-6,
        free_air_a: 17.0,
    },
    Gauge {
        awg: "20",
        area_m2: 0.519e-6,
        free_air_a: 23.0,
    },
    Gauge {
        awg: "18",
        area_m2: 0.823e-6,
        free_air_a: 33.0,
    },
    Gauge {
        awg: "16",
        area_m2: 1.31e-6,
        free_air_a: 41.0,
    },
    Gauge {
        awg: "14",
        area_m2: 2.08e-6,
        free_air_a: 55.0,
    },
    Gauge {
        awg: "12",
        area_m2: 3.31e-6,
        free_air_a: 76.0,
    },
    Gauge {
        awg: "10",
        area_m2: 5.26e-6,
        free_air_a: 101.0,
    },
    Gauge {
        awg: "8",
        area_m2: 8.37e-6,
        free_air_a: 135.0,
    },
    Gauge {
        awg: "6",
        area_m2: 13.3e-6,
        free_air_a: 181.0,
    },
    Gauge {
        awg: "4",
        area_m2: 21.2e-6,
        free_air_a: 245.0,
    },
    Gauge {
        awg: "2",
        area_m2: 33.6e-6,
        free_air_a: 330.0,
    },
    Gauge {
        awg: "1",
        area_m2: 42.4e-6,
        free_air_a: 380.0,
    },
    Gauge {
        awg: "1/0",
        area_m2: 53.5e-6,
        free_air_a: 440.0,
    },
    Gauge {
        awg: "2/0",
        area_m2: 67.4e-6,
        free_air_a: 520.0,
    },
    Gauge {
        awg: "3/0",
        area_m2: 85.0e-6,
        free_air_a: 610.0,
    },
    Gauge {
        awg: "4/0",
        area_m2: 107.2e-6,
        free_air_a: 700.0,
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizing {
    pub gauge: Gauge,
    pub parallel: u32,
    pub limit_a: f64,
}

impl Gauge {
    pub fn bundle_limit_a(&self) -> f64 {
        self.free_air_a * BUNDLE_DERATE
    }

    pub fn conductor_diameter_m(&self) -> f64 {
        (4.0 * self.area_m2 / std::f64::consts::PI).sqrt()
    }

    pub fn outer_diameter_m(&self) -> f64 {
        let d = self.conductor_diameter_m();
        d + 2.0 * (0.25e-3 + 0.05 * d)
    }
}

pub fn size_wire(rating_a: f64) -> Sizing {
    let rating = rating_a.max(0.0);
    for g in GAUGES {
        if g.bundle_limit_a() >= rating {
            return Sizing {
                gauge: g,
                parallel: 1,
                limit_a: g.bundle_limit_a(),
            };
        }
    }
    let g = GAUGES[GAUGES.len() - 1];
    let n = (rating / g.bundle_limit_a()).ceil().max(1.0);
    Sizing {
        gauge: g,
        parallel: n as u32,
        limit_a: g.bundle_limit_a() * n,
    }
}

pub fn ambient_derate(ambient_k: f64) -> f64 {
    ((MAX_CONDUCTOR_K - ambient_k).max(0.0) / (MAX_CONDUCTOR_K - AMBIENT_REF_K)).sqrt()
}
