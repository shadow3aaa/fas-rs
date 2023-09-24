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

use log::debug;

use super::{Buffer, Looper};
use crate::{error::Result, PerformanceController};

struct Policy {
    pub basic_scale: Duration,
    pub basic_rec: usize,
    pub big_scale: Duration,
    pub big_rec: usize,
}

impl Policy {
    pub fn adapt(target_fps: u32) -> Self {
        if target_fps <= 60 {
            Self {
                basic_scale: Duration::from_secs(1) / target_fps / target_fps * 3 / 2,
                basic_rec: 8,
                big_scale: Duration::from_secs(1) / target_fps * 5,
                big_rec: 30,
            }
        } else {
            Self {
                basic_scale: Duration::from_secs(1) / target_fps / target_fps * 3,
                basic_rec: 0,
                big_scale: Duration::from_secs(1) / target_fps * 20,
                big_rec: 15,
            }
        }
    }
}

impl<P: PerformanceController> Looper<P> {
    pub fn buffer_policy(&mut self) -> Result<()> {
        if self.buffers.is_empty() && self.started {
            self.controller.init_default(&self.config)?;
            self.started = false;
            return Ok(());
        } else if !self.buffers.is_empty() && !self.started {
            self.controller.init_game(&self.config)?;
            self.started = true;
        }

        let levels: Vec<_> = self.buffers.values().map(Self::calculate_level).collect();

        let Some(level) = levels.iter()
            .filter_map(|l| *l)
            .max() else {
            return Ok(());
        };

        if level == 0 && levels.iter().any(Option::is_none) {
            return Ok(());
        }

        debug!("jank-level: {level}");

        self.controller.perf(level, &self.config);
        Ok(())
    }

    fn calculate_level(buffer: &Buffer) -> Option<u32> {
        let policy = Policy::adapt(buffer.target_fps);
        let target_frametime = Duration::from_secs(1) / buffer.target_fps;

        let frametimes = &buffer.frametimes;
        let frametime = frametimes.iter().next()?;

        let diff = frametime.saturating_sub(target_frametime);

        debug!("diff: {diff:?}");

        let level = if diff < policy.big_scale {
            diff.as_nanos() / policy.basic_scale.as_nanos()
        } else {
            10
        };

        let level: u32 = level.try_into().unwrap_or(u32::MAX);

        if level == 0
            && (frametimes
                .iter()
                .take(policy.basic_rec)
                .any(|d| d.saturating_sub(target_frametime) > policy.basic_scale)
                || frametimes
                    .iter()
                    .take(policy.big_rec)
                    .any(|d| d.saturating_sub(target_frametime) > policy.big_scale))
        {
            None
        } else {
            Some(level)
        }
    }
}
