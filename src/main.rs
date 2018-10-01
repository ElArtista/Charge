extern crate gl;
extern crate glutin;
extern crate time;

mod game;
mod graphics;
mod mainloop;

use game::Game;
use mainloop::MainLoop;

fn main() {
    println!("Hello, world!");
    let mut game = Game::new();
    let mut mainloop = MainLoop::new(
        Box::new(|mut game, dt| Game::update(&mut game, dt)),
        Box::new(|game, interpolation| Game::render(&game, interpolation)),
        Some(Box::new(|game, ms, fps| Game::perf(&game, ms, fps))),
        &mut game,
    );
    mainloop.run();
}
