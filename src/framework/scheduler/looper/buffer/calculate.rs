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

use std::time::Duration;

use likely_stable::unlikely;
#[cfg(debug_assertions)]
use log::debug;

use super::Buffer;
use crate::{
    api::{v2::ApiV2, v3::ApiV3},
    framework::config::TargetFps,
    Extension,
};

impl Buffer {
    pub fn calculate_current_fps(&mut self) {
        let avg_time: Duration = self
            .frametime_state
            .frametimes
            .iter()
            .sum::<Duration>()
            .saturating_add(self.frametime_state.additional_frametime)
            .checked_div(self.frametime_state.frametimes.len().try_into().unwrap())
            .unwrap_or_default();
        #[cfg(debug_assertions)]
        debug!("avg_time: {avg_time:?}");

        self.frametime_state.avg_time = avg_time;

        let current_fps = 1.0 / avg_time.as_secs_f64();

        #[cfg(debug_assertions)]
        debug!("current_fps: {:.2}", current_fps);

        self.frametime_state.current_fps = current_fps;

        while self.frametime_state.current_fpses.len() >= 5 {
            self.frametime_state.current_fpses.pop_back();
        }

        self.frametime_state.current_fpses.push_front(current_fps);
    }

    pub fn calculate_target_fps(&mut self, extension: &Extension) {
        let new_target_fps = self.target_fps();
        if self.target_fps_state.target_fps != new_target_fps {
            if let Some(target_fps) = new_target_fps {
                extension.trigger_extentions(ApiV2::TargetFpsChange(
                    target_fps,
                    self.package_info.pkg.clone(),
                ));
                extension.trigger_extentions(ApiV3::TargetFpsChange(
                    target_fps,
                    self.package_info.pkg.clone(),
                ));
            }

            self.target_fps_state.target_fps = new_target_fps;
            self.unusable();
        }
    }

    fn target_fps(&self) -> Option<u32> {
        let target_fpses = match &self.target_fps_state.target_fps_config {
            TargetFps::Value(t) => vec![*t],
            TargetFps::Array(arr) => arr.clone(),
        };

        let mut current_fps: Option<f64> = None;
        for next_fps in self.frametime_state.current_fpses.iter().copied().take(144) {
            if let Some(fps) = current_fps {
                current_fps = Some(fps.max(next_fps));
            } else {
                current_fps = Some(next_fps);
            }
        }

        let current_fps = current_fps?;

        if unlikely(current_fps < (target_fpses[0].saturating_sub(10).max(10)).into()) {
            return None;
        }

        for target_fps in target_fpses.iter().copied() {
            if current_fps <= f64::from(target_fps) + 3.0 {
                #[cfg(debug_assertions)]
                debug!(
                    "Matched target_fps: current: {:.2} target_fps: {target_fps}",
                    current_fps
                );

                return Some(target_fps);
            }
        }

        target_fpses.last().copied()
    }
}
