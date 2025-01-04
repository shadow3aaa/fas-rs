// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

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

impl FrameTimeState {
    fn new() -> Self {
        Self {
            current_fps: 0.0,
            current_fpses: VecDeque::with_capacity(144 * 3),
            avg_time: Duration::ZERO,
            frametimes: VecDeque::with_capacity(1440),
            additional_frametime: Duration::ZERO,
        }
    }
}

#[derive(Debug)]
pub struct TargetFpsState {
    pub target_fps: Option<u32>,
    target_fps_config: TargetFps,
}

impl TargetFpsState {
    const fn new(target_fps_config: TargetFps) -> Self {
        Self {
            target_fps: None,
            target_fps_config,
        }
    }
}

#[derive(Debug)]
pub struct BufferState {
    pub last_update: Instant,
    pub working_state: BufferWorkingState,
    calculate_timer: Instant,
    working_state_timer: Instant,
}

impl BufferState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_update: now,
            working_state: BufferWorkingState::Unusable,
            calculate_timer: now,
            working_state_timer: now,
        }
    }
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
            frametime_state: FrameTimeState::new(),
            target_fps_state: TargetFpsState::new(target_fps_config),
            state: BufferState::new(),
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
