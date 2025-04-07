// Copyright 2024-2025, shadow3aaa
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

use std::time::Duration;

use likely_stable::unlikely;
#[cfg(debug_assertions)]
use log::debug;

use super::Buffer;
use crate::{Extension, api::trigger_target_fps_change, framework::config::TargetFps};

impl Buffer {
    pub fn calculate_current_fps(&mut self) {
        let avg_time_long = self.calculate_average_frametime(None);
        #[cfg(debug_assertions)]
        debug!("avg_time_long: {avg_time_long:?}");

        self.frametime_state.avg_time_long = avg_time_long;

        let current_fps_long = 1.0 / avg_time_long.as_secs_f64();
        #[cfg(debug_assertions)]
        debug!("current_fps_long: {current_fps_long:.2}");

        self.frametime_state.current_fps_long = current_fps_long;

        let avg_time_short = self
            .calculate_average_frametime(self.target_fps().map(|target_fps| target_fps as usize));
        #[cfg(debug_assertions)]
        debug!("avg_time_short: {avg_time_short:?}");

        self.frametime_state.avg_time_short = avg_time_short;

        let current_fps_short = 1.0 / avg_time_short.as_secs_f64();
        #[cfg(debug_assertions)]
        debug!("current_fps_short: {current_fps_short:.2}");

        self.frametime_state.current_fps_short = current_fps_short;
    }

    fn calculate_average_frametime(&self, it_takes: Option<usize>) -> Duration {
        let total_time: Duration = self
            .frametime_state
            .frametimes
            .iter()
            .take(it_takes.unwrap_or(self.frametime_state.frametimes.len()))
            .sum::<Duration>()
            .saturating_add(self.frametime_state.additional_frametime);

        total_time
            .checked_div(
                it_takes
                    .unwrap_or(self.frametime_state.frametimes.len())
                    .min(self.frametime_state.frametimes.len())
                    .try_into()
                    .unwrap(),
            )
            .unwrap_or_default()
    }

    pub fn calculate_target_fps(&mut self, extension: &Extension) {
        let new_target_fps = self.target_fps();
        if self.target_fps_state.target_fps != new_target_fps || new_target_fps.is_none() {
            self.reset_frametime_state();
            if let Some(target_fps) = new_target_fps {
                self.trigger_target_fps_change(extension, target_fps);
            }
            self.target_fps_state.target_fps = new_target_fps;
            self.unusable();
        }
    }

    fn reset_frametime_state(&mut self) {
        self.frametime_state.frametimes.clear();
    }

    fn trigger_target_fps_change(&self, extension: &Extension, target_fps: u32) {
        trigger_target_fps_change(extension, target_fps, self.package_info.pkg.clone());
    }

    fn target_fps(&self) -> Option<u32> {
        let target_fpses = match &self.target_fps_state.target_fps_config {
            TargetFps::Value(t) => vec![*t],
            TargetFps::Array(arr) => arr.clone(),
        };

        let current_fps = self.frametime_state.current_fps_long;

        if unlikely(current_fps < (target_fpses.first()?.saturating_sub(10).max(10)).into()) {
            return None;
        }

        for &target_fps in &target_fpses {
            if current_fps <= f64::from(target_fps) + 3.0 {
                #[cfg(debug_assertions)]
                debug!(
                    "Matched target_fps: current: {current_fps:.2} target_fps: {target_fps}"
                );
                return Some(target_fps);
            }
        }

        target_fpses.last().copied()
    }
}
