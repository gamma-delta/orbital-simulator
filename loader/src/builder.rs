//! Lets you construct solar systems with nested orbiting more easily.

use simulator::bodies::*;
use euclid::default::{Point2D, Vector2D};

/// Use this struct to construct a solar system easily
pub struct SolarSystemBuilder {
    /// The stuff in the solar system
    entries: Vec<SolarSystemBuilderEntry>,
    /// This is set to true after .construct() is called.
    /// Trying to do operations after .construct() is called will panic.
    used_up: bool,
}

impl SolarSystemBuilder {
    /// Make a new empty Builder.
    pub fn new() -> SolarSystemBuilder {
        SolarSystemBuilder {
            entries: Vec::new(),
            used_up: false,
        }
    }

    /// Add an entry to the Builder.
    pub fn add(&mut self, ssbe: SolarSystemBuilderEntry) -> &mut SolarSystemBuilder {
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
        for base_entry in self.entries.iter() {
            // Always base it on (0, 0)
            out.append(&mut SolarSystemBuilder::construct_one_level(
                base_entry,
                Point2D::zero(),
                Vector2D::zero(),
            ));
        }

        out
    }

    /// Inner function for construct()'s recursion.
    fn construct_one_level(
        entry: &SolarSystemBuilderEntry,
        parent_pos: Point2D<f64>,
        parent_vel: Vector2D<f64>,
    ) -> Vec<Orbiter> {
        let mut out: Vec<Orbiter> = Vec::new();

        let (this_pos, this_vel) = match entry.stores {
            Orbit(oer) => (parent_pos + oer.1.pos.to_vector(), parent_vel + oer.1.vel),
            Locus(point) => (parent_pos + point.to_vector(), parent_vel),
        };

        // Put the orbiter in the orbiters if it's an orbiter
        // Wow that's a mouthful
        if let Orbit(oer) = entry.stores {
            out.push(Orbiter(oer.0, Kinemat::new(this_pos, this_vel)));
        }

        // Do the same for each child
        for child in entry.children.iter() {
            out.append(&mut SolarSystemBuilder::construct_one_level(
                child, this_pos, this_vel,
            ));
        }

        out
    }
}

/// An entry in a SolarSystemBuilder.
#[derive(Clone)]
pub struct SolarSystemBuilderEntry {
    stores: SSBEType,
    children: Vec<SolarSystemBuilderEntry>,
}

/// An SSBE can either be an Orbiter or a Locus.
/// Orbiters add the position and velocity.
/// Loci only consider the position, and no Orbiter is added to the SolarSystem because of it.
#[derive(Copy, Clone)]
enum SSBEType {
    Orbit(Orbiter),
    Locus(Point2D<f64>),
}

use SSBEType::*;

impl SolarSystemBuilderEntry {
    /// Create a new SolarSystemBuilderEntry
    pub fn new(orbiter: Orbiter) -> SolarSystemBuilderEntry {
        SolarSystemBuilderEntry {
            stores: Orbit(orbiter),
            children: Vec::new(),
        }
    }

    /// Create a new SolarSystemBuilderEntry from a Body and a Kinemat
    pub fn new_parts(body: Body, kmat: Kinemat) -> SolarSystemBuilderEntry {
        SolarSystemBuilderEntry {
            stores: Orbit(Orbiter(body, kmat)),
            children: Vec::new(),
        }
    }

    /// Create a new SolarSystemBuilderEntry with no Orbiter attached.
    /// This can be useful if you want to center things around a locus
    /// without attaching a body.
    pub fn new_empty(pos: Point2D<f64>) -> SolarSystemBuilderEntry {
        SolarSystemBuilderEntry {
            stores: Locus(pos),
            children: Vec::new(),
        }
    }

    /// Add another SolarSystemBuilderEntry as a child of this one.
    /// Returns itself so you can keep chaining it.
    pub fn add(mut self, child: SolarSystemBuilderEntry) -> SolarSystemBuilderEntry {
        self.children.push(child);
        self
    }

    /// Add a whole vector of SolarSystemBuilderEntries as a child of this one.
    /// Consumes the children.
    pub fn add_bulk<T: Iterator<Item = SolarSystemBuilderEntry>>(
        mut self,
        children: T,
    ) -> SolarSystemBuilderEntry {
        self.children.extend(children);
        self
    }
}
