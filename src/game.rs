use std;

pub struct Game {
    x: i32,
}

impl Game {
    pub fn new() -> Game {
        Game { x: 0 }
    }

    pub fn init(&mut self) {
        println!("Game.init!");
        self.x = 1;
    }

    pub fn update(&mut self, dt: f32) -> bool {
        println!("Game update! Dt: {}", dt);
        self.x += 1;
        self.x >= 5
    }

    pub fn render(&self, interpolation: f32) {
        println!("Game render: {}! Interpolation: {}", self.x, interpolation);
        let dur = std::time::Duration::from_micros(10);
        std::thread::sleep(dur);
    }

    pub fn perf(&self, ms: f32, fps: f32) {
        println!("Game perf: {} ms / {} fps", ms, fps);
    }

    pub fn shutdown(&mut self) {
        println!("Game.shutdown!");
        self.x = 10;
    }
}
