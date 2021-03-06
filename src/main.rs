#[cfg(target_os = "android")]
extern crate android_glue;
extern crate gl;
extern crate glutin;
pub extern crate image;
extern crate nalgebra_glm;
extern crate rusttype;
extern crate time;
extern crate tobj;

mod assets;
mod game;
mod graphics;
mod mainloop;
mod math;

use game::Game;
use mainloop::MainLoop;

fn main() {
    println!("Hello, world!");
    let mut game = Game::new();
    let mut mainloop = MainLoop::new(
        Box::new(|mut game, dt| Game::update(&mut game, dt)),
        Box::new(|game, interpolation| Game::render(&game, interpolation)),
        Some(Box::new(|mut game, t, u, r| Game::perf(&mut game, t, u, r))),
        &mut game,
    );
    mainloop.run();
}
