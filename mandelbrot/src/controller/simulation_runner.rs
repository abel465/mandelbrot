use web_time::Instant;

pub struct SimulationRunner {
    pub paused: bool,
    pub speed: f32,
    distance: f32,
    last_frame: Instant,
}

impl SimulationRunner {
    pub fn new(now: Instant, paused: bool) -> Self {
        Self {
            speed: 1.0,
            distance: 0.0,
            last_frame: now,
            paused,
        }
    }

    pub fn add_iteration(&mut self) {
        self.distance += 1.0;
    }

    pub fn iterations(&mut self) -> u32 {
        let speed = if self.paused { 0.0 } else { self.speed };
        let t = self.last_frame.elapsed().as_secs_f32() * 30.0;
        self.last_frame = Instant::now();
        self.distance += speed * t;
        if self.distance >= 1.0 {
            let iterations = self.distance as u32;
            self.distance = self.distance.fract();
            iterations
        } else {
            0
        }
    }
}
