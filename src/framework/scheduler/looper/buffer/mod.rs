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
pub enum BufferWorkingState {
    Unusable,
    Usable,
}

#[derive(Debug)]
pub struct PackageInfo {
    pub pid: pid_t,
    pub pkg: String,
}

#[derive(Debug)]
pub struct FrameTimeState {
    pub current_fps: f64,
    pub current_fpses: VecDeque<f64>,
    pub avg_time: Duration,
    pub frametimes: VecDeque<Duration>,
    pub additional_frametime: Duration,
}

#[derive(Debug)]
pub struct TargetFpsState {
    pub target_fps: Option<u32>,
    target_fps_config: TargetFps,
}

#[derive(Debug)]
pub struct BufferState {
    pub last_update: Instant,
    pub working_state: BufferWorkingState,
    calculate_timer: Instant,
    working_state_timer: Instant,
}

#[derive(Debug)]
pub struct Buffer {
    pub package_info: PackageInfo,
    pub frametime_state: FrameTimeState,
    pub target_fps_state: TargetFpsState,
    pub state: BufferState,
}

impl Buffer {
    pub fn new(target_fps_config: TargetFps, pid: pid_t, pkg: String) -> Self {
        Self {
            package_info: PackageInfo { pid, pkg },
            target_fps_state: TargetFpsState {
                target_fps: None,
                target_fps_config,
            },
            frametime_state: FrameTimeState {
                current_fps: 0.0,
                current_fpses: VecDeque::with_capacity(144 * 3),
                avg_time: Duration::ZERO,
                frametimes: VecDeque::with_capacity(1440),
                additional_frametime: Duration::ZERO,
            },
            state: BufferState {
                last_update: Instant::now(),
                calculate_timer: Instant::now(),
                working_state: BufferWorkingState::Unusable,
                working_state_timer: Instant::now(),
            },
        }
    }

    pub fn push_frametime(&mut self, d: Duration, extension: &Extension) {
        self.frametime_state.additional_frametime = Duration::ZERO;
        self.state.last_update = Instant::now();

        while self.frametime_state.frametimes.len()
            >= self.target_fps_state.target_fps.unwrap_or(144) as usize * 5
        {
            self.frametime_state.frametimes.pop_back();
            self.try_usable();
        }

        self.frametime_state.frametimes.push_front(d);
        self.try_calculate(extension);
    }

    fn try_calculate(&mut self, extension: &Extension) {
        if unlikely(self.state.calculate_timer.elapsed() >= Duration::from_millis(100)) {
            self.state.calculate_timer = Instant::now();
            self.calculate_current_fps();
            self.calculate_target_fps(extension);
        }
    }

    pub fn try_usable(&mut self) {
        if self.state.working_state == BufferWorkingState::Unusable
            && self.state.working_state_timer.elapsed() >= Duration::from_secs(1)
        {
            self.state.working_state = BufferWorkingState::Usable;
        }
    }

    pub fn unusable(&mut self) {
        self.state.working_state = BufferWorkingState::Unusable;
        self.state.working_state_timer = Instant::now();
    }

    pub fn additional_frametime(&mut self, extension: &Extension) {
        self.frametime_state.additional_frametime = self.state.last_update.elapsed();
        self.try_calculate(extension);
    }
}
