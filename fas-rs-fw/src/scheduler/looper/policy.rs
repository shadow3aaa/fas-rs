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
use std::time::{Duration, Instant};

use log::debug;

use super::{Buffer, Looper, FRAME_UNIT};
use crate::{config::TargetFps, error::Result, Config, PerformanceController};

impl<P: PerformanceController> Looper<P> {
    pub fn buffers_policy(&mut self) -> Result<()> {
        if self.buffers.is_empty() && self.started {
            self.controller.init_default(&self.config)?;
            self.started = false;
            return Ok(());
        } else if !self.buffers.is_empty() && !self.started {
            self.controller.init_game(&self.config)?;
            self.started = true;
        }

        for buffer in self.buffers.values_mut() {
            Self::do_policy(buffer, &self.controller, &self.config)?;
        }

        Ok(())
    }

    fn do_policy(buffer: &mut Buffer, controller: &P, config: &Config) -> Result<()> {
        if buffer.frametimes.len() < FRAME_UNIT {
            return Ok(());
        }

        let Some(target_fps) = (match buffer.target_fps {
            TargetFps::Auto => Self::calculate_fps(buffer),
            TargetFps::Value(f) => Some(f),
        }) else {
            return Ok(());
        };

        let policy = Self::policy_config()?;

        debug!("mode policy: {policy:?}");

        let frame = buffer.frame_unit.front().copied().unwrap();
        let frame_unit: Duration = buffer.frame_unit.iter().sum();

        let normalized_frame = frame * target_fps;
        let normalized_frame_unit = frame_unit * target_fps;

        debug!("target_fps: {target_fps}");
        debug!("normalized_frame: {normalized_frame:?}");
        debug!("normalized_frame_unit: {normalized_frame_unit:?}");

        if normalized_frame > Duration::from_millis(3500) + policy.tolerant_big_jank {
            controller.release_max(config)?; // big jank
        } else if normalized_frame > Duration::from_millis(1700) + policy.tolerant_jank {
            buffer.rec_counter = policy.jank_keep_count; // jank

            let last_jank = buffer.last_jank;
            buffer.last_jank = Some(Instant::now());

            if let Some(last_jank) = last_jank {
                let normalized_last_jank = last_jank.elapsed() * target_fps;

                if normalized_last_jank >= Duration::from_secs(60) {
                    return Ok(()); // 1 jank is allowed every 60 frames
                }
            }

            if let Some(front) = buffer.frame_unit.front_mut() {
                *front = Duration::from_secs(1) / target_fps;
            }

            controller.release(config)?;
        } else if normalized_frame_unit
            <= Duration::from_secs(1) * FRAME_UNIT.try_into().unwrap() * policy.tolerant_unit
        {
            if buffer.rec_counter != 0 {
                buffer.rec_counter -= 1;
                return Ok(());
            }

            if let Some(last_limit) = buffer.last_limit {
                let normalized_last_limit = last_limit.elapsed() * target_fps;
                if normalized_last_limit <= Duration::from_secs(3) {
                    return Ok(());
                } // 1 limit is allowed every 3 frames
            }

            buffer.last_limit = Some(Instant::now());
            buffer.rec_counter = policy.normal_keep_count;

            controller.limit(config)?;
        } else if normalized_frame_unit
            > Duration::from_secs(1) * FRAME_UNIT.try_into().unwrap() * policy.tolerant_unit
        {
            if let Some(front) = buffer.frame_unit.front_mut() {
                *front = Duration::from_secs(1) / target_fps;
            }

            controller.release(config)?;
        }

        Ok(())
    }
}
