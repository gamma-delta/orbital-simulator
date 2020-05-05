//! Lets you load a SolarSystem from a file.

use serde::Deserialize;

/// A Vector2D or Point2D.
#[derive(Deserialize)]
struct Vec2D(f64, f64);

/// A point in space with children in relation to it.
#[derive(Deserialize)]
#[serde(untagged)]
enum Orbiter {
    Locus(OLocus),
    Solid(OSolid),
}
/// An Orbiter only used to mark a point in space so other things can use it as relatives.
#[derive(Deserialize)]
struct OLocus {
    pos: Vec2D,
    #[serde(default)]
    children: Vec<Orbiter>,
}
/// An actual orbiter with a body & kinemat
#[derive(Deserialize)]
struct OSolid {
    body: Body,
    kinemat: Kinemat,
    #[serde(default)]
    children: Vec<Orbiter>,
}

/// A Body in space
#[derive(Deserialize)]
#[serde(untagged)]
enum Body {
    Prefab(String), // A pre-made pre-defined Body
    Custom(BodyConfig),
}

#[derive(Deserialize)]
struct BodyConfig {
    mass: f64,
    radius: f64,
    color: u32,
    outline: u32,
}

#[derive(Deserialize)]
struct Kinemat {
    pos: Vec2D,
    vel: Vec2D,
}

/// Serde needs you to define the thing to use it on...
#[derive(Deserialize)]
struct RawSolarSystem(Vec<Orbiter>);

use crate::builder::{SolarSystemBuilder, SolarSystemBuilderEntry as SSBE};
use euclid::default::{Point2D, Vector2D};
use json5;
use simulator::bodies as real;

/// Loads a file and returns the ingredients for a solar system.
pub fn load(contents: String) -> Result<Vec<real::Orbiter>, json5::Error> {
    let contents = &*contents;
    let raw: RawSolarSystem = json5::from_str(contents)?;
    let builder = &mut SolarSystemBuilder::new();

    for root in raw.0 {
        builder.add(do_one_level(root));
    }

    Ok(builder.construct())
}

/// Helper function to DFS convert from serde to real
fn do_one_level(orbiter: Orbiter) -> SSBE {
    let (children, ssbe) = match orbiter {
        Orbiter::Locus(l) => (l.children, SSBE::new_empty(Point2D::new(l.pos.0, l.pos.1))),
        Orbiter::Solid(s) => (
            s.children,
            SSBE::new_parts(
                match s.body {
                    Body::Prefab(id) => get_body_from_id(id),
                    Body::Custom(cfg) => real::Body {
                        mass: cfg.mass,
                        radius: cfg.radius,
                        color: cfg.color,
                        outline: cfg.outline,
                    },
                },
                real::Kinemat {
                    pos: Point2D::new(s.kinemat.pos.0, s.kinemat.pos.1),
                    vel: Vector2D::new(s.kinemat.vel.0, s.kinemat.vel.1),
                },
            ),
        ),
    };
    ssbe.add_bulk(children.into_iter().map(|child| do_one_level(child)))
}

/// Gets a premade Body from a string
fn get_body_from_id(id: String) -> real::Body {
    use crate::prefabs::bodies;
    use std::collections::HashMap;

    macro_rules! maker {
        (
            $($name:ident),*
        ) => {
            {
                let mut h: HashMap<String, fn() -> real::Body> = HashMap::new();
                $( h.insert(stringify!($name).to_string(), bodies::$name); )*
                h
            }
        };
    }

    lazy_static! {
        static ref BODIES: HashMap<String, fn() -> real::Body> = {
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
