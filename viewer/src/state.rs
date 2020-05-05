//! Handles the state for the simulator.

use simulator::{SimulationMode, SolarSystem};

use euclid::default::Point2D;
use ggez::event::{EventHandler, KeyCode};
use ggez::nalgebra::Point2;
use ggez::{
    graphics::{self, DrawMode, DrawParam, MeshBuilder},
    input::keyboard,
    timer, Context, GameResult,
};

use graphics::Color;
use std::collections::HashSet;

/// The state of the solar system.
pub struct State {
    solar_system: SolarSystem,
    /// How many seconds should be simulated per frame
    sim_seconds_per_frame: f64,
    /// All the keypresses last frame
    prev_keys: HashSet<KeyCode>,

    // Display stuff
    /// This many meters in distance = 1 pixel
    distance_scale: f64,
    /// The radius of bodies are additionally scaled by this much
    planet_scale: f64,
    /// Whether to fake the scale of planets by squishing them, for less existential dread
    fake_planet_scale: bool,
    /// What I'm focusing on
    focus: Focus,
}

/// What my focus is on
enum Focus {
    /// Contains the ID of the body I'm focusing on
    Body(usize),
    /// I'm focusing on a point in space
    Position(Point2D<f64>),
}

impl State {
    pub fn new(_ctx: &mut Context, solar_system: SolarSystem) -> Self {
        let s = State {
            solar_system,
            sim_seconds_per_frame: SIM_SECONDS_PER_FRAME,
            prev_keys: HashSet::new(),
            distance_scale: DEFAULT_SCALE,
            planet_scale: DEFAULT_PLANET_SCALE,
            fake_planet_scale: true,
            focus: Focus::Position(Point2D::zero()),
        };
        s
    }

    /// Fix the screen space to always have (0, 0) in the corner and (w, h) in the other.
    fn fix_coordinates(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult<()> {
        let rect = graphics::Rect::new(0.0, 0.0, width, height);
        graphics::set_screen_coordinates(ctx, rect)
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // Calculate how much of the simulation should be dt and how much should be steps per frame
            // At small time scales, do a lot of small steps.
            // At big time scales, do a few giant steps.
            let steps_per_second = 100.0 * (self.sim_seconds_per_frame + 100.0).recip();
            let steps_per_frame = self.sim_seconds_per_frame * steps_per_second;
            let seconds_per_step = self.sim_seconds_per_frame / steps_per_frame;

            // Weird experiment...
            let seconds_per_step = if keyboard::is_key_pressed(ctx, KeyCode::Tab) {
                -seconds_per_step
            } else {
                seconds_per_step
            };

            for _ in 0..steps_per_frame.ceil() as usize {
                self.solar_system.update(seconds_per_step);
            }
            let orbiters = self.solar_system.get_orbiters();

            // Press tilde to reset scales
            if keyboard::is_key_pressed(ctx, KeyCode::Grave) {
                self.distance_scale = DEFAULT_SCALE;
                self.planet_scale = DEFAULT_PLANET_SCALE;
                self.fake_planet_scale = true;
                self.sim_seconds_per_frame = SIM_SECONDS_PER_FRAME;
            } else {
                // Zoom & pan f i'm not trying to reset.
                if keyboard::is_key_pressed(ctx, KeyCode::Q) {
                    self.distance_scale /= ZOOM_SPEED;
                }
                if keyboard::is_key_pressed(ctx, KeyCode::Z) {
                    self.distance_scale *= ZOOM_SPEED;
                }
                if keyboard::is_key_pressed(ctx, KeyCode::E) {
                    self.planet_scale /= ZOOM_SPEED;
                }
                if keyboard::is_key_pressed(ctx, KeyCode::C) {
                    self.planet_scale *= ZOOM_SPEED;
                }

                // Flip faking the planet size with the X key
                if keyboard::is_key_pressed(ctx, KeyCode::X)
                    && !self.prev_keys.contains(&KeyCode::X)
                {
                    self.fake_planet_scale = !self.fake_planet_scale;
                }

                // BACKUPS & SPEED
                // Speed and slow the simulation with []
                if keyboard::is_key_pressed(ctx, KeyCode::LBracket)
                    && self.sim_seconds_per_frame > 0.01
                // if it goes to zero, it's never coming back. so be careful
                {
                    self.sim_seconds_per_frame /= SPEED_SPEED;
                }
                if keyboard::is_key_pressed(ctx, KeyCode::RBracket) {
                    self.sim_seconds_per_frame *= SPEED_SPEED;
                }
                match self.solar_system.get_mode() {
                    SimulationMode::Simulating => {
                        // Use Return to toggle modes
                        if keyboard::is_key_pressed(ctx, KeyCode::Return)
                            && !self.prev_keys.contains(&KeyCode::Return)
                        {
                            self.solar_system.enable_load();
                        }
                    }
                    SimulationMode::LoadingSave(_) => {
                        // Change thing to load with ; and '
                        if keyboard::is_key_pressed(ctx, KeyCode::Semicolon) {
                            // Negative = older
                            self.solar_system.change_load(-(steps_per_frame as isize));
                        }
                        if keyboard::is_key_pressed(ctx, KeyCode::Apostrophe) {
                            // Positive = newer
                            self.solar_system.change_load(steps_per_frame as isize);
                        }

                        // Use Return to toggle modes
                        if keyboard::is_key_pressed(ctx, KeyCode::Return)
                            && !self.prev_keys.contains(&KeyCode::Return)
                        {
                            self.solar_system.exit_load();
                        }
                    }
                };

                let pan_speed = PAN_SPEED * self.distance_scale;
                match &mut self.focus {
                    Focus::Position(pos) => {
                        if keyboard::is_key_pressed(ctx, KeyCode::Space)
                            && !self.prev_keys.contains(&KeyCode::Space)
                        {
                            // Reset to the origin
                            *pos = Point2D::zero();
                        } else {
                            // Check for panning if I'm not pressing a key
                            if keyboard::is_key_pressed(ctx, KeyCode::W) {
                                pos.y -= pan_speed;
                            }
                            if keyboard::is_key_pressed(ctx, KeyCode::S) {
                                pos.y += pan_speed;
                            }
                            if keyboard::is_key_pressed(ctx, KeyCode::A) {
                                pos.x -= pan_speed;
                            }
                            if keyboard::is_key_pressed(ctx, KeyCode::D) {
                                pos.x += pan_speed;
                            }
                        }

                        // Press left or right arrow to go to Body mode
                        if keyboard::is_key_pressed(ctx, KeyCode::Left)
                            || keyboard::is_key_pressed(ctx, KeyCode::Right)
                        {
                            let id_maybe = orbiters.keys().next();
                            if let Some(first_valid_id) = id_maybe {
                                self.focus = Focus::Body(*first_valid_id)
                            } // Else, there's no bodies somehow. Uh-oh...
                        }
                    }
                    Focus::Body(id) => {
                        // Press Space or any WASD to exit focusing the planet
                        if keyboard::is_key_pressed(ctx, KeyCode::Space)
                            || keyboard::is_key_pressed(ctx, KeyCode::W)
                            || keyboard::is_key_pressed(ctx, KeyCode::S)
                            || keyboard::is_key_pressed(ctx, KeyCode::A)
                            || keyboard::is_key_pressed(ctx, KeyCode::D)
                        {
                            self.focus = Focus::Position(orbiters.get(id).unwrap().1.pos);
                        } else {
                            // We're not trying to exit
                            let mut oh_no_there_are_no_orbiters = false;
                            if keyboard::is_key_pressed(ctx, KeyCode::Right)
                                && !self.prev_keys.contains(&KeyCode::Right)
                            {
                                let maybe_tup = orbiters.range(*id + 1..).next();
                                if let Some(tup) = maybe_tup {
                                    *id = *tup.0 // Move it there!
                                } else {
                                    // Cycle back to the beginning
                                    let id_maybe = orbiters.keys().next();
                                    if let Some(first_valid_id) = id_maybe {
                                        *id = *first_valid_id;
                                    } else {
                                        //there's no bodies somehow. Uh-oh...
                                        oh_no_there_are_no_orbiters = true;
                                    }
                                }
                            }
                            if keyboard::is_key_pressed(ctx, KeyCode::Left)
                                && !self.prev_keys.contains(&KeyCode::Left)
                            {
                                let maybe_tup = orbiters.range(..*id).next_back();
                                if let Some(tup) = maybe_tup {
                                    *id = *tup.0 // Move it there!
                                } else {
                                    // Cycle back to the end
                                    let id_maybe = orbiters.keys().last();
                                    if let Some(first_valid_id) = id_maybe {
                                        *id = *first_valid_id;
                                    } else {
                                        //there's no bodies somehow. Uh-oh...
                                        oh_no_there_are_no_orbiters = true;
                                    }
                                }
                            }

                            if oh_no_there_are_no_orbiters {
                                self.focus = Focus::Position(Point2D::zero());
                            }
                        }
                    }
                }
            }

            // Update previous keys
            self.prev_keys = keyboard::pressed_keys(ctx).to_owned();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb_u32(0x200b2b));

        let orbiters = self.solar_system.get_orbiters();
        let focus_coord = match self.focus {
            Focus::Body(id) => {
                if let Some(focused_orbiter) = orbiters.get(&id) {
                    focused_orbiter.1.pos
                } else {
                    // Oh no we're trying to focus on something that doesn't exist ;(
                    self.focus = Focus::Position(Point2D::zero());
                    Point2D::zero()
                }
            }
            Focus::Position(pos) => pos,
        };

        let (scr_w, scr_h) = graphics::drawable_size(ctx);

        for (&_id, orbiter) in orbiters.iter() {
            let relative_pos = orbiter.1.pos - focus_coord;
            // Make (0, 0) in pixel coords the center of the screen
            let draw_pos = Point2::new(
                scr_w / 2f32 + (relative_pos.x / self.distance_scale) as f32,
                scr_h / 2f32 + (relative_pos.y / self.distance_scale) as f32,
            );
            let draw_radius = scale_planet(
                orbiter.0.radius,
                self.distance_scale * self.planet_scale,
                self.fake_planet_scale,
            );

            // Only spend processing time drawing it if it's in frame.
            if draw_pos.x + draw_radius > 0.0
                && draw_pos.x - draw_radius <= scr_w
                && draw_pos.y + draw_radius > 0.0
                && draw_pos.y - draw_radius <= scr_h
            {
                let draw = MeshBuilder::new()
                    .circle(
                        DrawMode::fill(),
                        draw_pos,
                        draw_radius,
                        0.01, // i don't know how low is is supposed to be aaaa
                        Color::from_rgb_u32(orbiter.0.color),
                    )
                    .circle(
                        DrawMode::stroke(draw_radius / 10.0),
                        draw_pos,
                        draw_radius,
                        0.01,
                        Color::from_rgb_u32(orbiter.0.outline),
                    )
                    .build(ctx)?;

                graphics::draw(ctx, &draw, DrawParam::default())?;
            }
        }

        graphics::present(ctx)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.fix_coordinates(ctx, width, height).unwrap(); // GGEZ official examples say to unwrap this... idk
    }
}

fn scale_planet(radius: f64, scale: f64, fake: bool) -> f32 {
    if fake {
        (10f64 * (radius / scale).powf(0.3)) as f32
    } else {
        (radius / scale) as f32 //
    }
    .max(0.5f32) // Everything has to be at least half a pixel wide, unfortunately. Otherwise it becomes impossible to see.
}

const PAN_SPEED: f64 = 10f64; // Pan this many pixels per frame
const ZOOM_SPEED: f64 = 1.1f64; // multiply / divide by this many meters per frame
const SPEED_SPEED: f64 = 1.05f64; // speed speed... the number of seconds simulated per frame changes by this amount per frame

const DEFAULT_SCALE: f64 = 1e10;
const DEFAULT_PLANET_SCALE: f64 = 1f64;

const SIM_SECONDS_PER_FRAME: f64 = 60f64 * 60f64 * 24f64; // Each frame is 24 * 60 * 60 seconds, or one day
