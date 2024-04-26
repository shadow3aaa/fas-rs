// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod calculate;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::framework::config::TargetFps;

#[derive(Debug)]
pub struct Buffer {
    pub pkg: String,
    pub target_fps: Option<u32>,
    pub current_fps: f64,
    pub current_fpses: VecDeque<f64>,
    pub avg_time: Duration,
    pub frametimes: VecDeque<Duration>,
    pub frame_prepare: Duration,
    pub last_update: Instant,
    target_fps_config: TargetFps,
    timer: Instant,
}

impl Buffer {
    pub fn new(target_fps_config: TargetFps, pkg: String) -> Self {
        Self {
            pkg,
            target_fps: None,
            target_fps_config,
            current_fps: 0.0,
            current_fpses: VecDeque::with_capacity(144 * 3),
            avg_time: Duration::ZERO,
            frametimes: VecDeque::with_capacity(144),
            frame_prepare: Duration::ZERO,
            last_update: Instant::now(),
            timer: Instant::now(),
        }
    }

    pub fn push_frametime(&mut self, d: Duration) {
        self.last_update = Instant::now();
        self.frame_prepare = Duration::ZERO;

        while self.frametimes.len() >= self.target_fps.unwrap_or(144) as usize {
            self.frametimes.pop_back();
        }

        self.frametimes.push_front(d);
        self.calculate_current_fps();

        if self.timer.elapsed() >= Duration::from_secs(5) {
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
