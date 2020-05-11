//! Lets you construct solar systems with nested orbiting more easily.

pub mod asteroids_builder;
pub mod locus;
pub mod moons_builder;
pub mod orbiter;

use euclid::default::{Point2D, Vector2D};

use simulator::bodies::*;

/// Use this struct to construct a solar system easily
pub struct SolarSystemBuilder {
    /// The stuff in the solar system
    entries: Vec<Box<dyn Entry>>,
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
    pub fn add(&mut self, entry: Box<dyn Entry>) -> &mut Self {
        if self.used_up {
            panic!("Tried to add an entry to a SolarSystemBuilder after it was constructed!")
        }
        self.entries.push(entry);
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
            out.append(&mut base_entry.construct(Relative::default()));
        }

        out
    }
}

/// An entry in a SolarSystemBuilder.
/// Must contain all the information needed to put Orbiters into a SolarSystem.
pub trait Entry {
    /// Return all the children.
    /// If you call it twice on something that relies on moves,
    /// it's OK to panic.
    fn construct(&mut self, relative: Relative) -> Vec<Orbiter>;
}

/// An Entry that can have children added to it.
pub trait EntryWithChildren: Entry {
    /// Add a new Entry as a child of this one.
    /// Must return itself so the method can be chained.
    fn add_child(&mut self, child: Box<dyn Entry>) -> Box<dyn EntryWithChildren>;
    /// Add multiple children as a child of this one.  
    /// By default it just calls `add_child` for each thing in the iterator,
    /// but some implementors of `EntryWithChildren` might do
    /// it differently.  
    /// Returns itself so it can be chained again.
    fn add_bulk_children(&mut self, children: Vec<Box<dyn Entry>>) -> Box<dyn EntryWithChildren> {
        for child in children {
            self.add_child(child);
        }
        self
    }
}

/// A Relative is used to position something relatively to its parents.
pub struct Relative {
    pos: Point2D<f64>,
    vel: Vector2D<f64>,
    mass: f64,
}

impl Default for Relative {
    fn default() -> Self {
        Self {
            pos: Point2D::zero(),
            vel: Vector2D::zero(),
            mass: 0.0,
        }
    }
}
