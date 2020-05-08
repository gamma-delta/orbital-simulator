//! Lets you load a SolarSystem from a file.

use serde::Deserialize;

/// A Vector2D or Point2D.
#[derive(Deserialize)]
struct Vec2D(f64, f64);

/// A point in space with children in relation to it.
#[derive(Deserialize)]
#[serde(untagged)]
enum Entry {
    Locus {
        pos: Vec2D,
        #[serde(default)]
        children: Vec<Entry>,
    },
    Orbiter {
        body: Body,
        kinemat: Kinemat,
        #[serde(default)]
        children: Vec<Entry>,
    },
    MoonsBuilder {
        count: usize,
        min_mass: f64,
        max_mass: f64,
        min_orbit: f64,
        max_orbit: f64,
        #[serde(default)]
        seed: u64,
        #[serde(default)]
        clockwise: bool,
    },
    AsteroidsBuilder {
        total_mass: f64,
        min_orbit: f64,
        max_orbit: f64,
        #[serde(default = "get_one_for_serde")]
        standard_dev: f64,
        #[serde(default)]
        max_bodies: Option<usize>,
        #[serde(default)]
        seed: u64,
        #[serde(default)]
        clockwise: bool,
    },
}

/// Returns `1f64` because Serde needs a function
fn get_one_for_serde() -> f64 {
    1f64
}

/// A Body in space
#[derive(Deserialize)]
#[serde(untagged)]
enum Body {
    Prefab(String), // A pre-made pre-defined Body
    Custom {
        mass: f64,
        radius: f64,
        name: String,
        color: u32,
        outline: u32,
        #[serde(default)]
        immovable: bool,
    },
}

#[derive(Deserialize)]
struct Kinemat {
    pos: Vec2D,
    vel: Vec2D,
}

/// Serde needs you to define the thing to use it on...
#[derive(Deserialize)]
struct RawSolarSystem(Vec<Entry>);

use crate::builder::{SolarSystemBuilder, SolarSystemBuilderEntry as SSBE};
use euclid::default::{Point2D, Vector2D};
use json5;
use simulator::bodies;

/// Loads a file and returns the ingredients for a solar system.
pub fn load(contents: String) -> Result<Vec<bodies::Orbiter>, json5::Error> {
    let contents = &*contents;
    let raw: RawSolarSystem = json5::from_str(contents)?;
    let builder = &mut SolarSystemBuilder::new();

    for root in raw.0 {
        builder.add(do_one_level(root));
    }

    Ok(builder.construct())
}

/// Helper function to DFS convert from serde to real
fn do_one_level(entry: Entry) -> SSBE {
    match entry {
        Entry::Locus { pos, mut children } => SSBE::new_locus(Point2D::new(pos.0, pos.1))
            .add_bulk(children.drain(0..).map(|kid| do_one_level(kid))),
        Entry::Orbiter {
            body,
            kinemat,
            mut children,
        } => SSBE::new_parts(
            match body {
                Body::Prefab(id) => get_body_from_id(id),
                Body::Custom {
                    mass,
                    radius,
                    name,
                    color,
                    outline,
                    immovable,
                } => bodies::Body {
                    mass,
                    radius,
                    name,
                    color,
                    outline,
                    immovable,
                },
            },
            bodies::Kinemat {
                pos: Point2D::new(kinemat.pos.0, kinemat.pos.1),
                vel: Vector2D::new(kinemat.vel.0, kinemat.vel.1),
            },
        )
        .add_bulk(children.drain(0..).map(|kid| do_one_level(kid))),
        Entry::MoonsBuilder {
            count,
            min_mass,
            max_mass,
            min_orbit,
            max_orbit,
            clockwise,
            seed,
        } => SSBE::MoonsBuilder {
            count,
            min_mass,
            max_mass,
            min_orbit,
            max_orbit,
            clockwise,
            seed,
        },
        Entry::AsteroidsBuilder {
            total_mass,
            min_orbit,
            max_orbit,
            standard_dev,
            max_bodies,
            seed,
            clockwise,
        } => SSBE::AsteroidsBuilder {
            total_mass,
            min_orbit,
            max_orbit,
            standard_dev,
            max_bodies,
            seed,
            clockwise,
        },
    }
}

/// Gets a premade Body from a string
fn get_body_from_id(id: String) -> bodies::Body {
    use crate::prefabs;
    use std::collections::HashMap;

    macro_rules! maker {
        (
            $($name:ident),*
        ) => {
            {
                let mut h: HashMap<String, fn() -> bodies::Body> = HashMap::new();
                $( h.insert(stringify!($name).to_string(), prefabs::bodies::$name); )*
                h
            }
        };
    }

    lazy_static! {
        static ref BODIES: HashMap<String, fn() -> bodies::Body> = {
            let h = maker![
                sol,
                mercury,
                venus,
                earth,
                luna,
                mars,
                phobos,
                deimos,
                jupiter,
                saturn,
                uranus,
                neptune,
                halleys_comet
            ];

            h
        };
    }

    BODIES
        .get(&id)
        .expect(&format!("No prefab body named {}", id))()
}
