//! Handles bodies and such

use euclid::default::{Point2D, Vector2D};

/// The representation of a body, like a star, planet, comet...
/// Doesn't store its position or velocity.
#[derive(Copy, Clone, Debug)]
pub struct Body {
    pub mass: f64,
    pub radius: f64,
    /// Color is stored as 0xRRGGBB
    pub color: u32,
    /// Color is stored as 0xRRGGBB
    pub outline: u32,
}

/// A Kinemat holds all the kinematic information about something.
#[derive(Copy, Clone, Debug)]
pub struct Kinemat {
    pub pos: Point2D<f64>,
    pub vel: Vector2D<f64>,
}

impl Kinemat {
    pub fn new(pos: Point2D<f64>, vel: Vector2D<f64>) -> Self {
        Self { pos, vel }
    }

    pub fn zero() -> Self {
        Self {
            pos: Point2D::zero(),
            vel: Vector2D::zero(),
        }
    }

    pub fn update(&mut self, dt: f64, acc: Vector2D<f64>) {
        self.vel += acc * dt;
        self.pos += self.vel * dt;
    }
}

/// An Orbiter is a combination of a Body and a Kinemat.
/// In other words, a thing and where it is (and how fast it's going.)
#[derive(Copy, Clone)]
pub struct Orbiter(pub Body, pub Kinemat);
