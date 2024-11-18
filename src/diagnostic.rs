use std::collections::VecDeque;
use std::time::Instant;

pub struct FrameQueue {
    frames: VecDeque<Instant>,
    max_frames: usize,
}

impl FrameQueue {
    pub fn new(max_frames: usize) -> Self {
        Self {
            frames: VecDeque::new(),
            max_frames,
        }
    }

    pub fn record_frame(&mut self) {
        let timestamp = Instant::now();

        self.frames.push_back(timestamp);

        if self.frames.len() > self.max_frames {
            self.frames.pop_front();
        }
    }

    pub fn calculate_fps(&self) -> f32 {
        if self.frames.len() < 2 {
            return 0.0;
        }

        let first = *self.frames.front().unwrap();
        let last = *self.frames.back().unwrap();
        let duration = (last - first).as_secs_f32();

        self.frames.len() as f32 / duration
    }

    pub fn get_last_frame(&self) -> Option<Instant> {
        self.frames.back().cloned()
    }
}
