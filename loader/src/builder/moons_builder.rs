//! Moons builder

use crate::builder::{Entry, Relative};

use simulator::bodies::*;

use euclid::default::{Point2D, Vector2D};

/// MoonsBuilder is a helper to build a ton of moons
pub struct MoonsBuilder {
    count: usize,
    min_mass: f64,
    max_mass: f64,
    min_orbit: f64,
    max_orbit: f64,
    seed: u64,
    clockwise: bool,
}

impl Entry for MoonsBuilder {
    fn construct(&mut self, relative: Relative) -> Vec<Orbiter> {
        use rand::{rngs::SmallRng, Rng, SeedableRng};

        // The mass when the normal returns 1 (~0.4% chance)
        // Currently, set to half the mass of Ceres.
        const MASS_AT_1: f64 = 9.3835e20 / 2.0;

        // Hash
        let seed = (self.count as u64)
            .wrapping_add(self.min_mass.to_bits())
            .wrapping_add(self.max_mass.to_bits())
            .wrapping_add(self.min_orbit.to_bits())
            .wrapping_add(self.max_orbit.to_bits())
            .wrapping_add(self.seed)
            .wrapping_add(self.clockwise as u64);
        const DENSITY: f64 = 3344.0; // The density of our moon in kg/m^3

        let mut rand = SmallRng::seed_from_u64(seed);

        // Generate the prefix name for the moon system
        const MOON_SYSTEM_CHARS: &[u8] = "ABCDEFGHJKLMNPQRSTUVWXYZ1234567890".as_bytes();
        let system_name: String = std::iter::once('M') // I really can't think of a better way to prepend something like this...
            .chain(
                (0..rand.gen_range(3, 6))
                    .map(|_| MOON_SYSTEM_CHARS[rand.gen_range(0, MOON_SYSTEM_CHARS.len())] as char),
            )
            .collect();

        (0..self.count)
            .map(|num| {
                let mass = rand.gen_range(self.min_mass, self.max_mass);
                let radius = (mass / DENSITY * 3.0 / (4.0 * 3.14159)).cbrt();
                // Do some math for a circular orbit
                let total_mass = mass + relative.mass;
                let theta = rand.gen_range(0f64, 2.0 * 3.14159f64);
                let orbit = rand.gen_range(self.min_orbit, self.max_orbit);
                let pos_x = theta.cos() * orbit;
                let pos_y = theta.sin() * orbit;
                let vel = (simulator::GRAV_CONSTANT * total_mass * orbit.recip()).sqrt()
                    * if self.clockwise { -1.0 } else { 1.0 };
                let vel_x = theta.cos() * vel;
                let vel_y = theta.sin() * vel;

                Orbiter(
                    Body {
                        mass,
                        radius,
                        color: 0x5566bb, // dark gray-blue,
                        outline: 0xeeddee,
                        name: format!("{}-{}", system_name, num),
                        immovable: false,
                    },
                    Kinemat::new(Point2D::new(pos_x, pos_y), Vector2D::new(vel_x, vel_y)),
                )
            })
            .collect()
    }
}
