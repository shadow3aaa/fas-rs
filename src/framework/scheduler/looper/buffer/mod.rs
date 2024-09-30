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

use libc::pid_t;
use likely_stable::unlikely;

use crate::{framework::config::TargetFps, Extension};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BufferState {
    Unusable,
    Usable,
}

#[derive(Debug)]
pub struct Buffer {
    pub pid: pid_t,
    pub pkg: String,
    pub target_fps: Option<u32>,
    pub current_fps: f64,
    pub current_fpses: VecDeque<f64>,
    pub avg_time: Duration,
    pub frametimes: VecDeque<Duration>,
    pub last_update: Instant,
    target_fps_config: TargetFps,
    timer: Instant,
    pub state: BufferState,
    state_timer: Instant,
    pub additional_frametime: Duration,
}

impl Buffer {
    pub fn new(target_fps_config: TargetFps, pid: pid_t, pkg: String) -> Self {
        Self {
            pid,
            pkg,
            target_fps: None,
            target_fps_config,
            current_fps: 0.0,
            current_fpses: VecDeque::with_capacity(144 * 3),
            avg_time: Duration::ZERO,
            frametimes: VecDeque::with_capacity(1440),
            last_update: Instant::now(),
            timer: Instant::now(),
            state: BufferState::Unusable,
            state_timer: Instant::now(),
            additional_frametime: Duration::ZERO,
        }
    }

    pub fn push_frametime(&mut self, d: Duration, extension: &Extension) {
        self.additional_frametime = Duration::ZERO;
        self.last_update = Instant::now();

        while self.frametimes.len() >= self.target_fps.unwrap_or(144) as usize * 5 {
            self.frametimes.pop_back();
            self.try_usable();
        }

        self.frametimes.push_front(d);

        if unlikely(self.timer.elapsed() >= Duration::from_secs(1)) {
            self.timer = Instant::now();
            self.calculate_current_fps();
            self.calculate_target_fps(extension);
        }
    }

    pub fn try_usable(&mut self) {
        if self.state == BufferState::Unusable
            && self.state_timer.elapsed() >= Duration::from_secs(1)
        {
            self.state = BufferState::Usable;
        }
    }

    pub fn unusable(&mut self) {
        self.state = BufferState::Unusable;
        self.state_timer = Instant::now();
    }

    pub fn additional_frametime(&mut self) {
        self.additional_frametime = self.last_update.elapsed();
    }
}
