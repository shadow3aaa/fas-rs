/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
pub mod calculate;
mod frame_acc;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::framework::config::TargetFps;
pub use frame_acc::Acc;

const BUFFER_LEN_SECS: usize = 10;

#[derive(Debug)]
pub struct Buffer {
    pub target_fps: Option<u32>,
    pub current_fps: f64,
    pub avg_time: Duration,
    pub frametimes: VecDeque<Duration>,
    pub frame_prepare: Duration,
    pub deviation: f64,
    pub last_update: Instant,
    pub acc_frame: Acc,
    pub acc_timer: Instant,
    pub limit_timer: Instant,
    deviations: VecDeque<f64>,
    target_fps_config: TargetFps,
    timer: Instant,
}

impl Buffer {
    pub fn new(t: TargetFps) -> Self {
        Self {
            target_fps: None,
            current_fps: 0.0,
            avg_time: Duration::ZERO,
            frametimes: VecDeque::with_capacity(144 * BUFFER_LEN_SECS),
            frame_prepare: Duration::ZERO,
            deviation: 0.0,
            last_update: Instant::now(),
            acc_frame: Acc::new(),
            acc_timer: Instant::now(),
            limit_timer: Instant::now(),
            deviations: VecDeque::with_capacity(144 * BUFFER_LEN_SECS),
            timer: Instant::now(),
            target_fps_config: t,
        }
    }

    pub fn push_frametime(&mut self, d: Duration) {
        self.last_update = Instant::now();
        self.frame_prepare = Duration::ZERO;

        while self.frametimes.len() >= self.target_fps.unwrap_or(60) as usize * BUFFER_LEN_SECS {
            self.frametimes.pop_back();
        }

        self.frametimes.push_front(d);
        self.calculate_current_fps();
        self.calculate_deviation();

        if self.timer.elapsed() >= Duration::from_secs(BUFFER_LEN_SECS as u64) {
            self.timer = Instant::now();
            self.calculate_target_fps();
        }
    }

    pub fn frame_prepare(&mut self) {
        self.frame_prepare = self.last_update.elapsed();
        self.calculate_current_fps();
        self.calculate_target_fps();
    }
}
