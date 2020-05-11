//! Asteroids builder

use crate::builder::*;

/// An AsteroidsBuilder is a helper struct for creating a bunch of asteroids.
/// Give it the total mass of the asteroids.
/// This way you don't end up with an asteroid belt heavier than the sun...
// pissily, i need to say Serialize even though you never should. >:(
// TODO: possibly manually implement Serialize that panics.
pub struct AsteroidsBuilder {
    total_mass: f64,
    min_orbit: f64,
    max_orbit: f64,
    standard_dev: f64,
    max_bodies: Option<usize>,
    seed: u64,
    clockwise: bool,
}

impl Entry for AsteroidsBuilder {
    fn construct(&mut self, relative: Relative) -> Vec<Orbiter> {
        use rand::{rngs::SmallRng, Rng, SeedableRng};
        use rand_distr::{Distribution, Normal};

        // The mass when the normal returns 1 (~0.4% chance)
        // Currently, set to half the mass of Ceres.
        const MASS_AT_1: f64 = 9.3835e20 / 2.0;

        let seed = self
            .total_mass
            .to_bits()
            .wrapping_add(self.min_orbit.to_bits())
            .wrapping_add(self.max_orbit.to_bits())
            .wrapping_add(self.standard_dev.to_bits())
            .wrapping_add(self.max_bodies.unwrap_or(0) as u64)
            .wrapping_add(self.seed)
            .wrapping_add(self.clockwise as u64);
        let mut rand = SmallRng::seed_from_u64(seed);
        let normal = Normal::new(0.0, self.standard_dev).unwrap();

        // Generate the prefix name for the asteroid system
        const ASTEROID_SYSTEM_CHARS: &[u8] = "ABCDEFGHJKLMNPQRSTUVWXYZ1234567890".as_bytes();
        let system_name: String = std::iter::once('A') // I really can't think of a better way to prepend something like this...
            .chain((0..rand.gen_range(4, 7)).map(|_| {
                ASTEROID_SYSTEM_CHARS[rand.gen_range(0, ASTEROID_SYSTEM_CHARS.len())] as char
            }))
            .collect();

        let mut asteroids: Vec<Orbiter> = Vec::new();
        let mut remaining_mass = self.total_mass;
        while remaining_mass > 0.0
            && match self.max_bodies {
                Some(max_bodies) => asteroids.len() < max_bodies,
                None => true,
            }
        {
            let mass = {
                let wip_mass = normal.sample(&mut rand).abs() * MASS_AT_1;
                remaining_mass -= wip_mass;
                if remaining_mass < 0.0 {
                    -remaining_mass // Don't withdraw more than the avaliable mass
                } else {
                    wip_mass
                }
            };
            // Isn't Wikipedia great? I can read all about the types of asteroids and their % in our asteroid belt!
            let asteroid_kind_id = rand.gen_range(0, 100);
            let (density, color, outline, id_char): (f64, u32, u32, char) = if asteroid_kind_id < 75
            {
                // Carbonaceous asteroids
                (1380.0, 0x4C1505, 0x8B7979, 'C')
            } else if asteroid_kind_id < 75 + 17 {
                // Silicate asteroids
                (2710.0, 0x819284, 0xA8CDBD, 'S')
            } else {
                // Metallic asteroids
                (5320.0, 0xC9D2E4, 0x618CD6, 'M')
            };

            let radius = (mass / density * 3.0 / (4.0 * 3.14159)).cbrt();
            let name = format!("{}-{:04}{}", system_name, asteroids.len(), id_char);

            // Kinematic info
            let system_mass = mass + relative.mass;
            let theta = rand.gen_range(0f64, 2.0 * 3.14159f64);
            let orbit = rand.gen_range(self.min_orbit, self.max_orbit);
            let pos_x = theta.cos() * orbit;
            let pos_y = theta.sin() * orbit;
            let vel = (simulator::GRAV_CONSTANT * system_mass * orbit.recip()).sqrt()
                * if self.clockwise { -1.0 } else { 1.0 };
            let vel_x = theta.cos() * vel;
            let vel_y = theta.sin() * vel;
            asteroids.push(Orbiter(
                Body {
                    mass,
                    radius,
                    color,
                    outline,
                    name,
                    immovable: false,
                },
                Kinemat::new(Point2D::new(pos_x, pos_y), Vector2D::new(vel_x, vel_y)),
            ))
        }
        asteroids
    }
}
