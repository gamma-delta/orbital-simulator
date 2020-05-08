//! Prefabricated orbiters and solar systems.

pub mod bodies {
    use simulator::bodies::Body;

    // REAL BODIES

    /// Returns our Sun. Will not move.
    pub fn sol() -> Body {
        Body {
            mass: 1.9884e30,
            radius: 695_700_000f64,
            name: "Sol".to_string(),
            color: 0xFFDF22,
            outline: 0xE87513,
            immovable: true,
        }
    }

    /// Returns Mercury.
    /// Apparently Mercury's orbit is going to be a little off. But I'm no Einstein.
    pub fn mercury() -> Body {
        Body {
            mass: 3.3011e23,
            radius: 1_439_700f64,
            name: "Mercury".to_string(),
            color: 0xa79ea1,   // light gray
            outline: 0x737375, // dark gray
            immovable: false,
        }
    }

    /// Returns Venus.
    pub fn venus() -> Body {
        Body {
            mass: 4.8675e24,
            radius: 6_051_800f64,
            name: "Venus".to_string(),
            color: 0xfcd172,   // gray yellow
            outline: 0xaf5a23, // brown
            immovable: false,
        }
    }

    /// Returns the Earth.
    pub fn earth() -> Body {
        Body {
            mass: 5.97237e24,
            radius: 6_371_000f64,
            name: "Earth".to_string(),
            color: 0x3669FF,   // blue
            outline: 0x56FF2D, // green
            immovable: false,
        }
    }

    /// Returns our Moon.
    /// Does not come with Wire.
    pub fn luna() -> Body {
        Body {
            mass: 7.342e22,
            radius: 1_737_400f64,
            name: "Luna".to_string(),
            color: 0x3c3a38,   // dark gray,
            outline: 0xadaca9, // light gray,
            immovable: false,
        }
    }

    /// Returns Mars
    pub fn mars() -> Body {
        Body {
            mass: 6.4171e23,
            radius: 3_398_500f64,
            name: "Mars".to_string(),
            color: 0xff5c26,   // red-orange
            outline: 0xc9af9e, // gray
            immovable: false,
        }
    }

    pub fn phobos() -> Body {
        moon(1.08e16, 11_100f64)
    }

    pub fn deimos() -> Body {
        moon(1.5e15, 6_300f64)
    }

    /// Returns Jupiter.
    pub fn jupiter() -> Body {
        Body {
            mass: 1.8982e27,
            radius: 69_911_000f64,
            name: "Jupiter".to_string(),
            color: 0x977569,   // bruisey brown
            outline: 0x8b5b45, // brown red
            immovable: false,
        }
    }

    /// Returns Saturn.
    pub fn saturn() -> Body {
        Body {
            mass: 5.6834e26,
            radius: 58_232_000f64,
            name: "Saturn".to_string(),
            color: 0xf5b92f,   // yellow,
            outline: 0x8c8109, // disturbingly close to urine
            immovable: false,
        }
    }

    /// Returns Uranus.
    pub fn uranus() -> Body {
        Body {
            mass: 86810e25,
            radius: 25_632_000f64,
            name: "Uranus".to_string(),
            color: 0x48faff,   // ice blue
            outline: 0x62e4f9, // darker blue
            immovable: false,
        }
    }

    /// Returns Neptune
    pub fn neptune() -> Body {
        Body {
            mass: 1.024_13e26,
            radius: 24_622_000f64,
            name: "Neptune".to_string(),
            color: 0x6e8add,   // light blue
            outline: 0xc3ddff, // lighter blue
            immovable: false,
        }
    }

    pub fn halleys_comet() -> Body {
        Body {
            mass: 2.2e14,
            radius: 11_000f64,
            name: "Halley's Comet".to_string(),
            color: 0xddddff,   // slightly blue white
            outline: 0x80b09b, //space purple
            immovable: false,
        }
    }

    // FANTASY BODIES

    /// Returns Roshar, from The Stormlight Archive.
    /// Thankfully the Coppermind has values for Roshar, somehow...
    pub fn roshar() -> Body {
        Body {
            mass: 3.387e24,
            radius: 5_633_000f64,
            name: "Roshar".to_string(),
            color: 0x015089,   // azure
            outline: 0xc1d8e6, // light blue
            immovable: false,
        }
    }

    // BODY BUILDERS

    /// Returns a generic moon
    pub fn moon(mass: f64, radius: f64) -> Body {
        Body {
            mass,
            radius,
            name: "Anonymous Moon".to_string(),
            color: 0xe8b374,   // orangey brown
            outline: 0x71401d, // brown
            immovable: false,
        }
    }
}

pub mod solar_systems {
    use crate::builder::{SolarSystemBuilder, SolarSystemBuilderEntry as SSBE};
    use crate::prefabs::bodies;
    use euclid::default::{Point2D, Vector2D};
    use simulator::bodies::*;
    use simulator::GRAV_CONSTANT;

    /// If you zoom in really really far you can see us!
    pub fn ours() -> Vec<Orbiter> {
        SolarSystemBuilder::new()
            .add(
                SSBE::new_parts(
                    bodies::sol(),
                    Kinemat::new(Point2D::zero(), Vector2D::zero()),
                )
                .add(SSBE::new_parts(
                    bodies::mercury(),
                    Kinemat::new(
                        Point2D::new(57_909_050_000f64, 0f64),
                        Vector2D::new(0f64, -47_362f64),
                    ),
                ))
                .add(SSBE::new_parts(
                    bodies::venus(),
                    Kinemat::new(
                        Point2D::new(-108_208_000_000f64, 0f64),
                        Vector2D::new(0f64, 35_020f64), // Venus and Uranus are the only planets that rotate clockwise.
                    ),
                ))
                .add(
                    SSBE::new_parts(
                        bodies::earth(),
                        Kinemat::new(
                            Point2D::new(149_598_023_000f64, 0f64),
                            Vector2D::new(0f64, -29780f64),
                        ),
                    )
                    // the moon is attached to earth
                    .add(SSBE::new_parts(
                        bodies::luna(),
                        Kinemat::new(
                            Point2D::new(0f64, 384_399_000f64),
                            Vector2D::new(1_022f64, 0f64),
                        ),
                    )),
                )
                .add(
                    SSBE::new_parts(
                        bodies::mars(),
                        Kinemat::new(
                            Point2D::new(227_939_000_000f64, 0f64),
                            Vector2D::new(0f64, -24_007f64),
                        ),
                    )
                    // Phobos
                    // I don't know why Phobos is flying away.
                    .add(SSBE::new_parts(
                        bodies::phobos(),
                        Kinemat::new(
                            Point2D::new(0f64, -9_377_000f64),
                            Vector2D::new(-2_140f64, 0f64),
                        ),
                    ))
                    // Deimos
                    .add(SSBE::new_parts(
                        bodies::deimos(),
                        Kinemat::new(
                            Point2D::new(0f64, 23_460_000f64),
                            Vector2D::new(1_350f64, 0f64),
                        ),
                    )),
                )
                .add(SSBE::new_parts(
                    bodies::jupiter(),
                    Kinemat::new(
                        Point2D::new(7.786e11, 0f64),
                        Vector2D::new(0f64, -13_070f64),
                    ),
                ))
                .add(SSBE::new_parts(
                    bodies::saturn(),
                    Kinemat::new(Point2D::new(-1.43353e12, 0f64), Vector2D::new(0.0, 9_680.0)),
                ))
                // This is terrifying me. why am I doing this at night
                .add(SSBE::new_parts(
                    bodies::neptune(),
                    Kinemat::new(Point2D::new(0f64, 4.5e12), Vector2D::new(5_430f64, 0f64)),
                ))
                // Halley's Comet
                .add(SSBE::new_parts(
                    bodies::halleys_comet(),
                    Kinemat::new(
                        // start at perhelion (closest point)
                        // will fly clockwise, long arm to the right
                        Point2D::new(8.766108e10, 0f64),
                        Vector2D::new(
                            // https://en.wikipedia.org/wiki/Orbital_speed
                            0f64,
                            (GRAV_CONSTANT * 1.9884e30 * (2.0 / 8.766108e10 - 2.668e12f64.recip()))
                                .sqrt(),
                        ),
                    ),
                )),
            )
            .construct()
    }

    /// Let's run some collision tests!
    pub fn collision_fun() -> Vec<Orbiter> {
        SolarSystemBuilder::new()
            .add(
                SSBE::new_parts(bodies::sol(), Kinemat::zero()).add(
                    SSBE::new_parts(
                        bodies::roshar(),
                        Kinemat::new(
                            // I put it at Earth's position cause why not...
                            Point2D::new(149_598_023_000f64, 0f64),
                            Vector2D::new(0f64, -2780f64),
                        ),
                    )
                    .add_bulk((1..=10).map(|num| {
                        SSBE::new_parts(
                            bodies::luna(),
                            Kinemat::new(
                                Point2D::new(30_000_000f64 * num as f64, 0f64),
                                Vector2D::new(0f64, 30_000f64),
                            ),
                        )
                    })),
                ),
            )
            .construct()
    }
}
