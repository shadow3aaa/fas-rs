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

        let policy = Self::policy_config(config)?;

        debug!("mode policy: {policy:?}");

        let frame_unit: Duration = buffer.frame_unit.iter().sum();
        let normalized_frame_unit = frame_unit * target_fps;

        let normalized_limit_scale = Duration::from_secs(1)
            .div_f64((f64::from(target_fps) - policy.tolerant_frame_limit).max(1.0))
            * target_fps
            * FRAME_UNIT.try_into().unwrap();
        let normalized_jank_scale = Duration::from_secs(1)
            .div_f64((f64::from(target_fps) - policy.tolerant_frame_jank).max(1.0))
            * target_fps
            * FRAME_UNIT.try_into().unwrap();
        let normalized_big_jank_scale =
            Duration::from_secs(1) * FRAME_UNIT.try_into().unwrap() * 10;

        debug!("target_fps: {target_fps}");
        debug!("normalized_frame_unit: {normalized_frame_unit:?}");

        if normalized_frame_unit > normalized_big_jank_scale {
            controller.release_max(config)?; // big jank
            buffer.counter = policy.jank_rec_count;
        } else if normalized_frame_unit < normalized_limit_scale {
            if buffer.counter != 0 {
                buffer.counter -= 1;
                return Ok(());
            }

            if let Some(last_limit) = buffer.last_limit {
                let normalized_last_limit = last_limit.elapsed() * target_fps;
                if normalized_last_limit <= Duration::from_secs(3) {
                    return Ok(());
                } // 1 limit is allowed every 3 frames
            }

            buffer.last_limit = Some(Instant::now());

            controller.limit(config)?;
        } else if normalized_frame_unit > normalized_jank_scale {
            buffer.counter = policy.jank_rec_count;

            if let Some(last_release) = buffer.last_release {
                let normalized_last_release = last_release.elapsed() * target_fps;
                if normalized_last_release <= Duration::from_secs(3) {
                    return Ok(());
                } // 1 release is allowed every 3 frames
            }

            buffer.last_release = Some(Instant::now());

            if let Some(front) = buffer.frame_unit.front_mut() {
                *front = Duration::from_secs(1) / target_fps;
            }

            controller.release(config)?;
        }

        Ok(())
    }
}
