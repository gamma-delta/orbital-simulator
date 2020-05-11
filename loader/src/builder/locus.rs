//! Locus

use crate::builder::{Entry, EntryWithChildren, Relative};

use simulator::bodies::*;

use euclid::default::Point2D;

/// A Locus is an Entry that just positions its children relative to it.
pub struct Locus {
    position: Point2D<f64>,
    children: Vec<Box<dyn Entry>>,
}

impl Locus {
    fn new(pos: Point2D<f64>) -> Self {
        Self {
            position: pos,
            children: Vec::new(),
        }
    }
}

impl Entry for Locus {
    fn construct(&mut self, relative: Relative) -> Vec<Orbiter> {
        let relative = Relative {
            pos: self.position + relative.pos.to_vector(),
            vel: relative.vel,
            mass: 0.0, // Mass is NOT carried over.
        };
        let mut out: Vec<Orbiter> = Vec::new();
        for child_entry in self.children {
            out.append(&mut child_entry.construct(relative));
        }
        out
    }
}

impl EntryWithChildren for Locus {
    fn add_child<T: EntryWithChildren>(&mut self, child: Box<dyn Entry>) -> &T {
        self.children.push(child);
        Box::new(self)
    }
    fn add_bulk_children<T: EntryWithChildren>(&mut self, children: Vec<Box<dyn Entry>>) -> &T {
        self.children.extend(children);
        Box::new(self)
    }
}
