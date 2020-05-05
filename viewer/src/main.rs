mod state;
use state::State;

use ggez::{
    conf::{Conf, WindowMode, WindowSetup},
    event, ContextBuilder,
};

pub fn main() {
    let c = Conf::new();
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("orbit_simulator", "me")
        .conf(c)
        .window_setup(WindowSetup {
            title: "Orbit simulator!".to_owned(),
            ..Default::default()
        })
        .window_mode(WindowMode {
            resizable: true,
            ..Default::default()
        })
        .build()
        .unwrap();

    let path_to_system = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() == 2 {
            args[1].clone()
        } else {
            "systems/ours.json5".to_string()
        }
    };
    let contents = std::fs::read_to_string(path_to_system).unwrap();
    let bodies = loader::load(contents).unwrap();
    let system = simulator::SolarSystem::new(bodies);

    let state = &mut State::new(ctx, system);

    event::run(ctx, event_loop, state).unwrap();
}
