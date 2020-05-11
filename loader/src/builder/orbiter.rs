//! Orbiter

use crate::builder::{Entry, EntryWithChildren, Relative};

use simulator::bodies;

/// An Orbiter puts an Orbiter into the solar system at its position.
/// It also positions child entries relative to it.
pub struct Orbiter {
    orbiter: bodies::Orbiter,
    children: Vec<Box<dyn Entry>>,
}

impl Entry for Orbiter {
    fn construct(&mut self, relative: Relative) -> Vec<bodies::Orbiter> {
        let relative = Relative {
            pos: self.orbiter.1.pos + relative.pos.to_vector(),
            vel: self.orbiter.1.vel + relative.vel,
            mass: self.orbiter.0.mass, // Mass is NOT carried over.
        };
        let mut out = vec![self.orbiter];
        for child_entry in self.children {
            out.append(&mut child_entry.construct(relative));
        }
        out
    }
}

impl EntryWithChildren for Orbiter {
    fn add_child<T: EntryWithChildren>(&mut self, child: Box<dyn Entry>) -> &T {
        self.children.push(child);
        self
    }
    fn add_bulk_children<T: EntryWithChildren>(&mut self, children: Vec<Box<dyn Entry>>) -> &T {
        self.children.extend(children);
        self
    }
}
