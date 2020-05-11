//! Lets you construct solar systems with nested orbiting more easily.

use euclid::default::{Point2D, Vector2D};
use simulator::bodies::*;

/// Use this struct to construct a solar system easily
// 'a says: all the references this uses will last for as long as the SSB does
pub struct SolarSystemBuilder {
    /// The stuff in the solar system
    entries: Vec<SolarSystemBuilderEntry>,
    /// This is set to true after .construct() is called.
    /// Trying to do operations after .construct() is called will panic.
    used_up: bool,
}

impl SolarSystemBuilder {
    /// Make a new empty Builder.
    pub fn new() -> Self {
        SolarSystemBuilder {
            entries: Vec::new(),
            used_up: false,
        }
    }

    /// Add an entry to the Builder.
    pub fn add(&mut self, ssbe: SolarSystemBuilderEntry) -> &mut Self {
        if self.used_up {
            panic!("Tried to add an entry to a SolarSystemBuilder after it was constructed!")
        }
        self.entries.push(ssbe);
        self
    }

    /// Calculates the positions and velocities of all entries, and returns them as a Vec
    /// suitable for passing to SolarSystem::new().
    /// Do not try to call .add() or .construct() after running this on an instance;
    /// it will panic.
    pub fn construct(&mut self) -> Vec<Orbiter> {
        if self.used_up {
            panic!("Tried to re-construct a SolarSystemBuilder after it was constructed!")
        }
        self.used_up = true;

        let mut out: Vec<Orbiter> = Vec::new();
        // Recursively do everything
        // Drain will remove the stuff from the entries
        for base_entry in self.entries.drain(0..) {
            // Always base it on (0, 0)
            out.append(&mut SolarSystemBuilder::construct_one_level(
                base_entry,
                0.0,
                Point2D::zero(),
                Vector2D::zero(),
            ));
        }

        out
    }

    /// Inner function for construct()'s recursion.
    fn construct_one_level(
        entry: SolarSystemBuilderEntry,
        parent_mass: f64,
        parent_pos: Point2D<f64>,
        parent_vel: Vector2D<f64>,
    ) -> Vec<Orbiter> {
        use SolarSystemBuilderEntry as SSBE;
        let mut out: Vec<Orbiter> = Vec::new();

        match entry {
            SSBE::Orbit(oer, mut children) => {
                let pos = parent_pos + oer.1.pos.to_vector();
                let vel = parent_vel + oer.1.vel;
                let mass = oer.0.mass;
                let this_orbiter = Orbiter(oer.0, Kinemat::new(pos, vel));
                out.push(this_orbiter);
                for child in children.drain(0..) {
                    out.append(&mut SolarSystemBuilder::construct_one_level(
                        child, mass, pos, vel,
                    ));
                }
            }

            SSBE::Locus(point, mut children) => {
                for child in children.drain(0..) {
                    out.append(&mut SolarSystemBuilder::construct_one_level(
                        child,
                        0.0,
                        point,
                        Vector2D::zero(),
                    ));
                }
            }

            SSBE::MoonsBuilder {
                count,
                min_mass,
                max_mass,
                min_orbit,
                max_orbit,
                seed,
                clockwise,
            } => {
                use rand::{rngs::SmallRng, Rng, SeedableRng};
                const DENSITY: f64 = 3344.0; // The density of our moon in kg/m^3

                let mut rand = SmallRng::seed_from_u64(seed);

                // Generate the prefix name for the moon system
                const MOON_SYSTEM_CHARS: &[u8] = "ABCDEFGHJKLMNPQRSTUVWXYZ1234567890".as_bytes();
                let system_name: String =
                    std::iter::once('M') // I really can't think of a better way to prepend something like this...
                        .chain((0..rand.gen_range(3, 6)).map(|_| {
                            MOON_SYSTEM_CHARS[rand.gen_range(0, MOON_SYSTEM_CHARS.len())] as char
                        }))
                        .collect();

                out.extend((0..count).map(|num| {
                    let mass = rand.gen_range(min_mass, max_mass);
                    let radius = (mass / DENSITY * 3.0 / (4.0 * 3.14159)).cbrt();
                    // Do some math for a circular orbit
                    let total_mass = mass + parent_mass;
                    let theta = rand.gen_range(0f64, 2.0 * 3.14159f64);
                    let orbit = rand.gen_range(min_orbit, max_orbit);
                    let pos_x = theta.cos() * orbit;
                    let pos_y = theta.sin() * orbit;
                    let vel = (simulator::GRAV_CONSTANT * total_mass * orbit.recip()).sqrt()
                        * if clockwise { -1.0 } else { 1.0 };
                    // Swap sin and cos cause it ought to be perpendicular
                    let vel_x = theta.sin() * vel;
                    let vel_y = theta.cos() * vel;

                    Orbiter(
                        Body {
                            mass,
                            radius,
                            color: 0x5566bb, // dark gray-blue,
                            outline: 0xeeddee,
                            name: format!("{}-{}", system_name, num),
                            immovable: false,
                        },
                        Kinemat::new(
                            Point2D::new(pos_x, pos_y) + parent_pos.to_vector(),
                            Vector2D::new(vel_x, vel_y) + parent_vel,
                        ),
                    )
                }));
            }
            SSBE::AsteroidsBuilder {
                total_mass,
                min_orbit,
                max_orbit,
                standard_dev,
                max_bodies,
                seed,
                clockwise,
            } => {
                use rand::{rngs::SmallRng, Rng, SeedableRng};
                use rand_distr::{Distribution, Normal};

                // The mass when the normal returns 1 (~0.4% chance)
                // Currently, set to half the mass of Ceres.
                const MASS_AT_1: f64 = 9.3835e20 / 2.0;

                let mut rand = SmallRng::seed_from_u64(seed);
                let normal = Normal::new(0.0, standard_dev).unwrap();

                // Generate the prefix name for the asteroid system
                const ASTEROID_SYSTEM_CHARS: &[u8] =
                    "ABCDEFGHJKLMNPQRSTUVWXYZ1234567890".as_bytes();
                let system_name: String =
                    std::iter::once('A') // I really can't think of a better way to prepend something like this...
                        .chain((0..rand.gen_range(4, 7)).map(|_| {
                            ASTEROID_SYSTEM_CHARS[rand.gen_range(0, ASTEROID_SYSTEM_CHARS.len())]
                                as char
                        }))
                        .collect();

                let mut asteroids: Vec<Orbiter> = Vec::new();
                let mut remaining_mass = total_mass;
                while remaining_mass > 0.0
                    && match max_bodies {
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
                    let (density, color, outline, id_char): (f64, u32, u32, char) =
                        if asteroid_kind_id < 75 {
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
                    let system_mass = mass + parent_mass;
                    let theta = rand.gen_range(0f64, 2.0 * 3.14159f64);
                    let orbit = rand.gen_range(min_orbit, max_orbit);
                    let pos_x = theta.cos() * orbit;
                    let pos_y = theta.sin() * orbit;
                    let vel = (simulator::GRAV_CONSTANT * system_mass * orbit.recip()).sqrt()
                        * if clockwise { -1.0 } else { 1.0 };
                    // Swap sin and cos cause it should be perpendicular
                    let vel_x = theta.sin() * vel;
                    let vel_y = theta.cos() * vel;
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
                out = asteroids;
            }
        };

        out
    }
}

/// One entry in a SolarSystemBuilder
pub enum SolarSystemBuilderEntry {
    /// Orbiters have an orbiter associated with them, and have children
    Orbit(Orbiter, Vec<SolarSystemBuilderEntry>),
    /// Loci only consider the position, and no Orbiter is added to the SolarSystem because of it.
    /// Still has children.
    Locus(Point2D<f64>, Vec<SolarSystemBuilderEntry>),
    /// MoonsBuilder is a helper to build a ton of moons. Give it the number of bodies you want.
    MoonsBuilder {
        count: usize,
        min_mass: f64,
        max_mass: f64,
        min_orbit: f64,
        max_orbit: f64,
        seed: u64,
        clockwise: bool,
    },
    /// AsteroidsBuilder is like a MoonBuilder but builds Asteroids instead.
    /// Give it the total mass of the asteroids.
    /// This way you don't end up with an asteroid belt heavier than the sun...
    AsteroidsBuilder {
        total_mass: f64,
        min_orbit: f64,
        max_orbit: f64,
        standard_dev: f64,
        max_bodies: Option<usize>,
        seed: u64,
        clockwise: bool,
    },
}

impl SolarSystemBuilderEntry {
    /// Create a new SolarSystemBuilderEntry::Orbiter
    pub fn new(orbiter: Orbiter) -> Self {
        SolarSystemBuilderEntry::Orbit(orbiter, Vec::new())
    }

    /// Create a new SolarSystemBuilderEntry::Orbiter from a Body and a Kinemat
    pub fn new_parts(body: Body, kmat: Kinemat) -> SolarSystemBuilderEntry {
        SolarSystemBuilderEntry::Orbit(Orbiter(body, kmat), Vec::new())
    }

    /// Create a new SolarSystemBuilderEntry::Locus
    /// This can be useful if you want to center things around a locus
    /// without attaching a body.
    pub fn new_locus(pos: Point2D<f64>) -> SolarSystemBuilderEntry {
        SolarSystemBuilderEntry::Locus(pos, Vec::new())
    }

    /// Add another SolarSystemBuilderEntry as a child of this one.
    /// Returns itself so you can keep chaining it.
    pub fn add(mut self, child: Self) -> Self {
        match &mut self {
            // The `ref` keyword is actually black magic
            SolarSystemBuilderEntry::Orbit(_, ref mut kids) => {kids.push(child); self}
            SolarSystemBuilderEntry::Locus(_, ref mut kids) => {kids.push(child); self}
            _ => panic!("Tried to add children to a SolarSystemBuilderEntry that wasn't an Orbit or a Locus!") 
        }
    }

    /// Add a whole vector of SolarSystemBuilderEntries as a child of this one.
    /// Consumes the children.
    pub fn add_bulk<T: Iterator<Item = Self>>(mut self, new_children: T) -> Self {
        match &mut self {
            SolarSystemBuilderEntry::Orbit(_, ref mut kids) => {kids.extend(new_children); self}
            SolarSystemBuilderEntry::Locus(_, ref mut kids) => {kids.extend(new_children); self}
            _ => panic!("Tried to add children to a SolarSystemBuilderEntry that wasn't an Orbit or a Locus!") 
        }
    }
}
