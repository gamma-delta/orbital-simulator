//! The root of this crate doesn't do anything.
//! It just re-exports its contents.

pub mod builder;
pub use builder::{SolarSystemBuilder, SolarSystemBuilderEntry}; // SolarSystemBuilder directly
pub mod deserialize;
pub mod prefabs; // prefabs::bodies::whatever
pub use deserialize::*;

#[macro_use]
extern crate lazy_static;
