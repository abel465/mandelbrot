use std::collections::VecDeque;
use web_time::{Duration, Instant};

pub struct FpsCounter {
    frames: VecDeque<Instant>,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frames: VecDeque::default(),
        }
    }

    pub fn tick(&mut self) -> usize {
        let one_second_from_now = Instant::now() + Duration::from_secs(1);
        self.frames.push_back(one_second_from_now);
        let now = one_second_from_now - Duration::from_secs(1);

        while self.frames.front().is_some_and(|t| t < &now) {
            self.frames.pop_front();
        }

        self.frames.len()
    }
}
