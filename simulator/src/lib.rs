//! Handles the simulation of the solar system

pub mod bodies;
use crate::bodies::{Body, Kinemat, Orbiter};
use euclid::default::Vector2D;

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub struct SolarSystem {
    /// Maps each ID number to a Body.
    /// When something is removed from `kinemats` it's not removed from here.
    /// Hey isn't that a memory leak, I hear you ask? Shut up!
    bodies: Vec<Body>,
    /// Every single kinemat that needs to be processed.
    kinemats: HashMap<usize, Kinemat>,
    /// All the saved states.
    /// This gets saved every `savePer` frames.
    saves: VecDeque<HashMap<usize, Kinemat>>,
    save_per: usize,
    frames_elapsed: usize,

    mode: SimulationMode,
}

/// What the solar system is up to.
#[derive(Copy, Clone)]
pub enum SimulationMode {
    /// Normal simulation
    Simulating,
    /// Selecting a save to load (possibly)
    LoadingSave(usize),
}

impl SolarSystem {
    pub fn new(orbiters: Vec<Orbiter>) -> Self {
        let mut bodies = Vec::with_capacity(orbiters.len());
        let mut kinemats = HashMap::with_capacity(orbiters.len());
        for (idx, Orbiter(body, kmat)) in orbiters.iter().enumerate() {
            bodies.push(*body);
            kinemats.insert(idx, *kmat);
        }
        SolarSystem {
            save_per: SAVE_EVERY,
            bodies,
            kinemats,
            saves: VecDeque::new(),
            frames_elapsed: 0,
            mode: SimulationMode::Simulating,
        }
    }

    pub fn update(&mut self, dt: f64) {
        match self.mode {
            SimulationMode::Simulating => {
                if self.frames_elapsed % self.save_per == 0 {
                    // time to save!
                    self.save()
                }

                let mut forces: HashMap<usize, Vector2D<f64>> =
                    HashMap::with_capacity(self.kinemats.len());
                // Stores any new orbiters formed by collision, and the IDs of the two orbiters that formed it
                let mut new_orbiters: Vec<(Orbiter, (usize, usize))> = Vec::new();
                // IDs of things we need to skip for whatever reason, like it was combined with something else.
                let mut skip_ids: HashSet<usize> = HashSet::new();
                for (&id, kmat) in self.kinemats.iter() {
                    if skip_ids.contains(&id) {
                        continue;
                    }
                    let body = &self.bodies[id];
                    let mut wip_force = Vector2D::<f64>::zero();
                    for (&other_id, other_kmat) in self.kinemats.iter() {
                        if other_id == id {
                            continue;
                        }
                        // Now if Rust only let me do a nested for loop properly I wouldn't need this bullshit
                        let other_body = self.bodies[other_id];
                        let dx = other_kmat.pos.x - kmat.pos.x;
                        let dy = other_kmat.pos.y - kmat.pos.y;
                        let dist_squared = dx * dx + dy * dy;
                        if dist_squared
                            < (body.radius + other_body.radius) * (body.radius + other_body.radius)
                        {
                            // ooh, a collision!
                            skip_ids.insert(other_id);
                            new_orbiters.push((
                                Orbiter(
                                    Body {
                                        mass: body.mass + other_body.mass,
                                        // Combine the radii as if they were actually spheres instead of just adding them.
                                        radius: ((3.0 / 4.0 * 3.14159f64)
                                            * (body.mass + other_body.mass))
                                            .cbrt(),
                                        color: (body.color
                                                + other_body.color)
                                                / 2,
                                        outline:
                                            (body.outline
                                                + other_body.outline)
                                                / 2,
                                        
                                    },
                                    Kinemat::new(
                                        // Position it fractionally beween this and that based on the mass proportion
                                        kmat.pos
                                            + Vector2D::new(dx, dy) * (other_body.mass / body.mass),
                                        // Momentum (mass * vel) is conserved!
                                        kmat.vel * body.mass + other_kmat.vel * other_body.mass,
                                    ),
                                ),
                                (id, other_id),
                            ))
                        } else {
                            let force =
                                GRAV_CONSTANT * ((body.mass * other_body.mass) / dist_squared);
                            let norm = Vector2D::new(dx, dy) / dist_squared.sqrt();
                            let force = norm * force;
                            wip_force += force;
                        }
                    }
                    forces.insert(id, wip_force);
                }

                for (&id, &force) in forces.iter() {
                    let acc = force / self.bodies[id].mass;
                    let kmat = self.kinemats.get_mut(&id).unwrap();
                    kmat.update(dt, acc);
                }
                for (new_orbiter, (id1, id2)) in new_orbiters.iter() {
                    self.kinemats.remove(id1);
                    self.kinemats.remove(id2);
                    self.add_orbiter(*new_orbiter);
                }

                // dbg!(self.kinemats.get(&1).unwrap());

                self.frames_elapsed += 1;
            }
            SimulationMode::LoadingSave(_) => {
                // Do jack shit
            }
        }
    }

    /// Add an orbiter to the SolarSystem.
    pub fn add_orbiter(&mut self, oer: Orbiter) {
        self.bodies.push(oer.0);
        self.kinemats.insert(self.bodies.len() - 1, oer.1);
    }

    /// Get a BTreeMap associating each id with an Orbiter.
    /// This makes a copy of the Oribters internally.
    /// It gets converted to a BTreeMap so the State can get the next ID easily if there's holes
    pub fn get_orbiters(&self) -> BTreeMap<usize, Orbiter> {
        match self.mode {
            SimulationMode::Simulating => &self.kinemats,
            SimulationMode::LoadingSave(number) => &self.saves[number],
        }
        .iter()
        .map(|(&id, &kmat)| (id, Orbiter(self.bodies[id], kmat)))
        .collect()
    }

    /// Save the current state
    fn save(&mut self) {
        self.saves.push_back(self.kinemats.clone());
        if self.saves.len() > SAVE_COUNT {
            // too long! Void the oldest please.
            self.saves.pop_front();
        }
    }

    /// Get the current mode
    pub fn get_mode(&mut self) -> SimulationMode {
        self.mode.clone()
    }

    /// Turn on LoadingSave mode. Also saves the current state.
    /// Returns whether it was successful or not.
    pub fn enable_load(&mut self) {
        println!(
            "Backup size: {} using {}k bytes of ram",
            self.saves.len(),
            (self.saves.iter().fold(0, |mem, hmap| mem
                + std::mem::size_of::<Kinemat>() * hmap.len()
                + std::mem::size_of::<HashMap<usize, Kinemat>>())
                + std::mem::size_of::<Vec<HashMap<usize, Kinemat>>>())
                / 1024
        );
        match self.mode {
            SimulationMode::Simulating => {
                self.save();
                let newest_save = self.saves.len() - 1;
                self.mode = SimulationMode::LoadingSave(newest_save);
            }
            SimulationMode::LoadingSave(_) => {
                panic!("Tried to turn on loading when loading was already turned on!")
            }
        }
    }

    /// If in LoadingSave mode, request to change which backup is viewed.
    /// Returns whether it was successful
    pub fn change_load(&mut self, by: isize) {
        match self.mode {
            SimulationMode::LoadingSave(number) => {
                let new_number = (number as isize + by)
                    .max(0)
                    .min(self.saves.len() as isize - 1) as usize;
                if let Some(_) = self.saves.get(new_number) {
                    // thats a valid index!
                    self.mode = SimulationMode::LoadingSave(new_number);
                } // else I don't know about that index somehow...
            }
            SimulationMode::Simulating => {
                panic!("Tried to change which backup is loaded while in simulating mode!")
            }
        }
    }

    /// Exit LoadingSave mode, switch to the selected backup, and delete all things newer than the backup restored.
    /// Returns whether it was successful yadayada etc
    pub fn exit_load(&mut self) {
        match self.mode {
            SimulationMode::LoadingSave(number) => {
                let save_to_restore = self.saves.get(number);
                match save_to_restore {
                    Some(restore) => {
                        self.kinemats = restore.to_owned();
                        self.mode = SimulationMode::Simulating;
                        self.saves.truncate(number);
                    }
                    None => panic!("Tried to restore to backup #{} but couldn't!", number),
                }
            }
            SimulationMode::Simulating => {
                panic!("Tried to exit loading mode while in simulating mode!")
            }
        }
    }
}

const SAVE_EVERY: usize = 24; // Save once every simulation day.
const SAVE_COUNT: usize = 10_000; // Save this many previous points.
pub const GRAV_CONSTANT: f64 = 6.674e-11;