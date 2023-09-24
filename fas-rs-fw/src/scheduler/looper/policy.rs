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

use super::{Buffer, Looper, BUFFER_MAX, FRAME_UNIT};
use crate::{config::TargetFps, error::Result, Config, PerformanceController};

const JANK_KEEP_COUNT: u8 = 30;
const NORMAL_KEEP_COUNT: u8 = 8;

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

    fn calulate_fps(buffer: &Buffer) -> Option<u32> {
        if buffer.frametimes.len() < BUFFER_MAX {
            return None;
        }

        let avg_time: Duration =
            buffer.frametimes.iter().sum::<Duration>() / BUFFER_MAX.try_into().unwrap();

        if avg_time < Duration::from_micros(6800) {
            None
        } else if avg_time < Duration::from_micros(8130) {
            Some(144)
        } else if avg_time < Duration::from_micros(10638) {
            Some(120)
        } else if avg_time < Duration::from_micros(16129) {
            Some(90)
        } else if avg_time < Duration::from_micros(21740) {
            Some(60)
        } else if avg_time < Duration::from_micros(32258) {
            Some(45)
        } else if avg_time < Duration::from_micros(50000) {
            Some(30)
        } else {
            None
        }
    }

    fn do_policy(buffer: &mut Buffer, controller: &P, config: &Config) -> Result<()> {
        if buffer.frametimes.len() < FRAME_UNIT {
            return Ok(());
        }

        let Some(target_fps) = (match buffer.target_fps {
            TargetFps::Auto => Self::calulate_fps(buffer),
            TargetFps::Value(f) => Some(f),
        }) else {
            return Ok(());
        };

        let frame = buffer.frametimes.front().copied().unwrap();
        let frame_unit: Duration = buffer.frametimes.iter().take(FRAME_UNIT).sum();

        let normalized_frame = frame * target_fps;
        let noramlized_frame_unit = frame_unit * target_fps;

        if normalized_frame > Duration::from_millis(3500) {
            controller.release_max(config)?; // big jank
        } else if normalized_frame > Duration::from_millis(1700) {
            buffer.rec_counter = JANK_KEEP_COUNT; // jank

            let last_jank = buffer.last_jank;
            buffer.last_jank = Some(Instant::now());

            if let Some(last_jank) = last_jank {
                let normalized_last_jank = last_jank.elapsed() * target_fps;

                if normalized_last_jank >= Duration::from_secs(60) {
                    return Ok(()); // 1 jank is allowed every 60 frames
                }
            }

            if let Some(front) = buffer.frametimes.front_mut() {
                *front = Duration::from_secs(1) / target_fps;
            }

            controller.limit(config)?;
        } else if noramlized_frame_unit < Duration::from_secs(1) * FRAME_UNIT.try_into().unwrap() {
            if buffer.rec_counter != 0 {
                buffer.rec_counter -= 1;
                return Ok(());
            }

            if let Some(last_limit) = buffer.last_limit {
                let normalized_last_limit = last_limit.elapsed() * target_fps;
                if normalized_last_limit <= Duration::from_secs(30) {
                    return Ok(());
                } // 1 limit is allowed every 30 frames
            }

            buffer.last_limit = Some(Instant::now());
            buffer.rec_counter = NORMAL_KEEP_COUNT;

            controller.limit(config)?;
        } else if noramlized_frame_unit > Duration::from_secs(1) * FRAME_UNIT.try_into().unwrap() {
            if let Some(front) = buffer.frametimes.front_mut() {
                *front = Duration::from_secs(1) / target_fps;
            }

            controller.release(config)?;
        }

        Ok(())
    }
}
