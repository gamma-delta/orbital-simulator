//! Handles the state for the simulator.

use simulator::{SimulationMode, SolarSystem};

use euclid::default::{Point2D, Vector2D};
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
    /// If I'm focusing on a body
    focused_body: Option<usize>,
    /// The offset of that focus
    focus_offset: Point2D<f64>,
    /// If a pop-up appears on a planet, what's its id?
    popuped_orbiter_id: Option<usize>,
    /// Whether to even draw a popup
    draw_popup: bool,
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
            focused_body: None,
            focus_offset: Point2D::zero(),
            popuped_orbiter_id: None,
            draw_popup: true,
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
            let steps_per_second = 10.0 * (self.sim_seconds_per_frame + 1_000.0).recip();
            let steps_per_frame = self.sim_seconds_per_frame * steps_per_second;
            let seconds_per_step = self.sim_seconds_per_frame / steps_per_frame;
            // Weird experiment...
            let seconds_per_step = if keyboard::is_key_pressed(ctx, KeyCode::Tab) {
                -seconds_per_step
            } else {
                seconds_per_step
            };

            if let SimulationMode::Simulating = self.solar_system.get_mode() {
                let frames = steps_per_frame.ceil() as u32;
                for _ in 0..frames {
                    self.solar_system.update(seconds_per_step);
                }
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

                // Toggle showing the popup with /
                if keyboard::is_key_pressed(ctx, KeyCode::Slash)
                    && !self.prev_keys.contains(&KeyCode::Slash)
                {
                    self.draw_popup = !self.draw_popup;
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
                            self.solar_system.change_load(-1);
                        }
                        if keyboard::is_key_pressed(ctx, KeyCode::Apostrophe) {
                            // Positive = newer
                            self.solar_system.change_load(1);
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
                if keyboard::is_key_pressed(ctx, KeyCode::Space)
                    && !self.prev_keys.contains(&KeyCode::Space)
                {
                    if let Some(_) = self.focused_body {
                        self.focused_body = None
                    } else {
                        self.focus_offset = Point2D::zero();
                    }
                } else {
                    // Check for panning
                    if keyboard::is_key_pressed(ctx, KeyCode::W) {
                        self.focus_offset.y -= pan_speed;
                    }
                    if keyboard::is_key_pressed(ctx, KeyCode::S) {
                        self.focus_offset.y += pan_speed;
                    }
                    if keyboard::is_key_pressed(ctx, KeyCode::A) {
                        self.focus_offset.x -= pan_speed;
                    }
                    if keyboard::is_key_pressed(ctx, KeyCode::D) {
                        self.focus_offset.x += pan_speed;
                    }
                }

                // Left/right arrows
                if let Some(id) = self.focused_body {
                    // Press Space to exit focusing the planet
                    if keyboard::is_key_pressed(ctx, KeyCode::Space) {
                        self.focus_offset = orbiters.get(&id).unwrap().1.pos;
                        self.focused_body = None;
                    } else {
                        // We're not trying to exit
                        if keyboard::is_key_pressed(ctx, KeyCode::Right)
                            && !self.prev_keys.contains(&KeyCode::Right)
                        {
                            self.focus_offset = Point2D::zero();
                            let maybe_tup = orbiters.range(id + 1..).next();
                            if let Some(tup) = maybe_tup {
                                self.focused_body = Some(*tup.0) // Move it there!
                            } else {
                                // Cycle back to the beginning
                                let id_maybe = orbiters.keys().next();
                                if let Some(first_valid_id) = id_maybe {
                                    self.focused_body = Some(*first_valid_id);
                                } else {
                                    //there's no bodies somehow. Uh-oh...
                                    self.focused_body = None
                                }
                            }
                        }
                        if keyboard::is_key_pressed(ctx, KeyCode::Left)
                            && !self.prev_keys.contains(&KeyCode::Left)
                        {
                            self.focus_offset = Point2D::zero();
                            let maybe_tup = orbiters.range(..id).next_back();
                            if let Some(tup) = maybe_tup {
                                self.focused_body = Some(*tup.0) // Move it there!
                            } else {
                                // Cycle back to the end
                                let id_maybe = orbiters.keys().last();
                                if let Some(first_valid_id) = id_maybe {
                                    self.focused_body = Some(*first_valid_id);
                                } else {
                                    //there's no bodies somehow. Uh-oh...
                                    self.focused_body = None;
                                }
                            }
                        }
                    }
                } else {
                    if keyboard::is_key_pressed(ctx, KeyCode::Left)
                        || keyboard::is_key_pressed(ctx, KeyCode::Right)
                    {
                        let id_maybe = if let Some(id) = self.popuped_orbiter_id {
                            Some(id)
                        } else if let Some(id) = orbiters.keys().next() {
                            Some(*id)
                        } else {
                            None
                        };
                        if let Some(first_valid_id) = id_maybe {
                            self.focused_body = Some(first_valid_id);
                        } else {
                            // Else, there's no bodies somehow. Uh-oh...
                            self.popuped_orbiter_id = None;
                        }
                    }
                }

                // Update previous keys
                self.prev_keys = keyboard::pressed_keys(ctx).to_owned();
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb_u32(0x200b2b));

        let orbiters = self.solar_system.get_orbiters();
        let focus_coord = self.focus_offset
            + match self.focused_body {
                Some(id) => match orbiters.get(&id) {
                    Some(o) => o.1.pos.to_vector(),
                    None => Vector2D::zero(),
                },
                None => Vector2D::zero(),
            };

        let (scr_w, scr_h) = graphics::drawable_size(ctx);

        let mut body_meshes = MeshBuilder::new();
        let mut text_box_meshes = MeshBuilder::new();

        // id, (x, y), radius
        let mut drawn_ids: Vec<(usize, (f32, f32), f32)> = Vec::new();
        for (&id, orbiter) in orbiters.iter() {
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
            // make sure we don't try to make a mesh with 0 vertices
            {
                drawn_ids.push((id, (draw_pos.x, draw_pos.y), draw_radius));
                // i don't know how low is is supposed to be aaaa
                let tolerance = (draw_radius * 2.0).recip();
                body_meshes
                    .circle(
                        DrawMode::fill(),
                        draw_pos,
                        draw_radius,
                        tolerance,
                        Color::from_rgb_u32(orbiter.0.color),
                    )
                    .circle(
                        DrawMode::stroke(draw_radius / 10.0),
                        draw_pos,
                        draw_radius,
                        tolerance,
                        Color::from_rgb_u32(orbiter.0.outline),
                    );
            }
        }

        if drawn_ids.len() >= 1 {
            let draw = body_meshes.build(ctx)?;
            graphics::draw(ctx, &draw, DrawParam::default())?;

            let popuped_orbiter_id = if let Some(id) = self.focused_body {
                Some(id)
            } else {
                // Check to see if any of the things we've drawn are good for this
                drawn_ids.iter().find_map(|(id, (x, y), radius)| {
                    if ((radius * radius) / (scr_w * scr_h))
                        * (
                            // These two lines are 1 when its drawn in the center
                            // of the screen and get closer to 0 the farther away its drawn
                            (1.0 - (x - scr_w / 2.0).abs() / (scr_w / 2.0))
                                * (1.0 - (y - scr_h / 2.0).abs() / (scr_h / 2.0))
                        )
                        / (drawn_ids.len() as f32 / orbiters.len() as f32) // The fewer things onscreen the easier it is to draw
                        >= PROPORTION_REQUIRED_FOR_LABEL
                    {
                        Some(*id)
                    } else {
                        None
                    }
                })
            };
            self.popuped_orbiter_id = popuped_orbiter_id;
            if self.draw_popup {
                if let Some(popuped_orbiter_id) = popuped_orbiter_id {
                    if let Some(popuped_orbiter) = orbiters.get(&popuped_orbiter_id) {
                        use graphics::{Text, TextFragment};
                        let message = format!("\nBody info:\n- Mass: {:.2e} kg\n- Radius: {:.2e} m\nKinematic info:\n- Position: ({:.2e}, {:.2e}) m\n- Velocity: ({:.2e}, {:.2e}) m/s",
                            popuped_orbiter.0.mass, popuped_orbiter.0.radius,
                            popuped_orbiter.1.pos.x, popuped_orbiter.1.pos.y, popuped_orbiter.1.vel.x, popuped_orbiter.1.vel.y);
                        let body_text = Text::new(TextFragment::new(message));
                        let (text_w, text_h) = body_text.dimensions(ctx);
                        let (text_w, text_h) = (text_w as f32, text_h as f32);

                        // Yes i already did this calculation, I know
                        let relative_pos = popuped_orbiter.1.pos - focus_coord;
                        let draw_pos = Point2::new(
                            scr_w / 2f32 + (relative_pos.x / self.distance_scale) as f32,
                            scr_h / 2f32 + (relative_pos.y / self.distance_scale) as f32,
                        );
                        let draw_radius = scale_planet(
                            popuped_orbiter.0.radius,
                            self.distance_scale * self.planet_scale,
                            self.fake_planet_scale,
                        );

                        // Setup the title text
                        let title_text =
                            Text::new(TextFragment::new(popuped_orbiter.0.name.clone()));
                        let title_width = title_text.width(ctx) as f32;

                        let text_w = text_w.max(title_width);

                        // Calculate where the corners of the text box should go.
                        // First pretend we calculate based on the upper left corner
                        let (corner_x, corner_y) = (
                            if draw_pos.x + text_w * 1.5 < scr_w {
                                draw_pos.x + draw_radius * 1.1 + text_w / 10.0
                            } else {
                                draw_pos.x - draw_radius * 1.1 - text_w - text_w / 10.0
                            },
                            if draw_pos.y + text_h * 1.5 < scr_h {
                                draw_pos.y
                            } else {
                                draw_pos.y - text_h
                            },
                        );

                        let textbox_rect = graphics::Rect::new(
                            corner_x - text_w / 10.0,
                            corner_y - text_h / 10.0,
                            text_w + text_w / 10.0 * 2.0,
                            text_h + text_h / 10.0 * 2.0,
                        );

                        // Add the textbox to the mesh of all textboxes
                        text_box_meshes.rectangle(DrawMode::fill(), textbox_rect, graphics::BLACK);

                        // Queue up the texts
                        graphics::queue_text(
                            ctx,
                            &body_text,
                            Point2::new(corner_x, corner_y),
                            Some(graphics::WHITE),
                        );
                        let title_text_x = corner_x + (text_w - title_width) / 2.0; // centered
                        graphics::queue_text(
                            ctx,
                            &title_text,
                            Point2::new(title_text_x, corner_y),
                            Some(graphics::WHITE),
                        );
                        let draw = text_box_meshes.build(ctx)?;
                        graphics::draw(ctx, &draw, DrawParam::default())?;
                        graphics::draw_queued_text(
                            ctx,
                            DrawParam::default(),
                            None,
                            graphics::default_filter(ctx),
                        )?;
                    }
                }
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
/// How much of the screen a body has to take up to show its label.
/// Multiplied by how close to the center of the screen the body is
/// Lower == easier to draw the popup
const PROPORTION_REQUIRED_FOR_LABEL: f32 = 0.000005;

const DEFAULT_SCALE: f64 = 1e10;
const DEFAULT_PLANET_SCALE: f64 = 1f64;

const SIM_SECONDS_PER_FRAME: f64 = 60f64 * 60f64 * 24f64; // Each frame is 24 * 60 * 60 seconds, or one day
