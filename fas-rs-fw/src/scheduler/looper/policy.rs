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

const LITTLE_JANK_REC: usize = 8;
const BIG_JANK_REC: usize = 30;

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
        let scale = buffer.scale;
        let target_frametime = Duration::from_secs(1) / buffer.target_fps;

        let little_jank = scale * 3 / 2;
        let big_jank = target_frametime * 5;

        let buffer = &buffer.frametimes;

        let frametime = buffer.iter().next()?;
        let diff = frametime.saturating_sub(target_frametime);

        debug!("diff: {diff:?}");

        let level = if diff < little_jank {
            0
        } else if diff < big_jank {
            1
        } else {
            8
        };

        if level == 0
            && (buffer
                .iter()
                .take(LITTLE_JANK_REC)
                .any(|d| d.saturating_sub(target_frametime) > little_jank)
                || buffer
                    .iter()
                    .take(BIG_JANK_REC)
                    .any(|d| d.saturating_sub(target_frametime) > big_jank))
        {
            None
        } else {
            Some(level)
        }
    }
}
