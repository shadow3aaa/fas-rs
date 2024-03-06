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
use std::time::Duration;

#[cfg(debug_assertions)]
use log::debug;

use super::Buffer;
use crate::framework::config::TargetFps;

impl Buffer {
    pub fn calculate_current_fps(&mut self) {
        let avg_time: Duration = self
            .frametimes
            .iter()
            .sum::<Duration>()
            .saturating_add(self.frame_prepare)
            / self.frametimes.len().try_into().unwrap();
        #[cfg(debug_assertions)]
        debug!("avg_time: {avg_time:?}");

        self.avg_time = avg_time;

        let current_fps = 1.0 / avg_time.as_secs_f64();

        #[cfg(debug_assertions)]
        debug!("current_fps: {:.2}", current_fps);

        self.current_fps = current_fps;
    }

    pub fn calculate_target_fps(&mut self) {
        let target_fpses = match &self.target_fps_config {
            TargetFps::Value(t) => vec![*t],
            TargetFps::Array(arr) => arr.clone(),
        };

        if self.current_fps < (target_fpses[0].saturating_sub(10).max(10)).into() {
            self.target_fps = None;
            return;
        }

        for target_fps in target_fpses.iter().copied() {
            if self.current_fps <= f64::from(target_fps) + 3.0 {
                #[cfg(debug_assertions)]
                debug!(
                    "Matched target_fps: current: {:.2} target_fps: {target_fps}",
                    self.current_fps
                );

                self.target_fps = Some(target_fps);
                return;
            }
        }

        self.target_fps = target_fpses.last().copied();
    }
}
