// https://github.com/unknownue/vulkan-tutorial-rust/blob/master/src/utility/fps_limiter.rs

use std::time;

const SAMPLE_COUNT: usize = 5;
const _SAMPLE_COUNT_FLOAT: f32 = SAMPLE_COUNT as f32;

pub struct FPSLimiter {
    counter: time::Instant,
    _frame_time_prefer: u32, // microseconds
    samples: [u32; SAMPLE_COUNT],
    curr_frame: usize,
    delta_frame: u32,
}

impl FPSLimiter {
    pub fn new() -> FPSLimiter {
        const DEFAULT_PREFER_FPS: f32 = 60.0;

        FPSLimiter {
            counter: time::Instant::now(),
            _frame_time_prefer: (1000000.0_f32 / DEFAULT_PREFER_FPS) as u32,
            samples: [0; SAMPLE_COUNT],
            curr_frame: 0,
            delta_frame: 0,
        }
    }

    pub fn _set_prefer_fps(&mut self, prefer_fps: f32) {
        self._frame_time_prefer = (1000000.0_f32 / prefer_fps) as u32;
    }

    pub fn tick_frame(&mut self) {
        let time_elapsed = self.counter.elapsed();
        self.counter = time::Instant::now();

        self.delta_frame = time_elapsed.subsec_micros();
        self.samples[self.curr_frame] = self.delta_frame;
        self.curr_frame = (self.curr_frame + 1) % SAMPLE_COUNT;
    }

    pub fn _fps(&self) -> f32 {
        let mut sum = 0_u32;
        self.samples.iter().for_each(|val| {
            sum += val;
        });

        1000000.0_f32 / (sum as f32 / _SAMPLE_COUNT_FLOAT)
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_frame as f32 / 1000000.0_f32 // time in seconds
    }
}

