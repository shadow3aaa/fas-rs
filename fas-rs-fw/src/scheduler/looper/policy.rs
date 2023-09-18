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

use log::debug;

use super::Looper;
use crate::{error::Result, PerformanceController};

const JANK_REC: usize = 5;
const BIG_JANK_REC: usize = 10;

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

        let level = self
            .buffers
            .values_mut()
            .filter_map(|(scale_time, jank_time)| {
                let result = if *jank_time > *scale_time {
                    self.jank_counter = BIG_JANK_REC;
                    Some(10)
                } else if *jank_time > *scale_time / 2 {
                    self.jank_counter = JANK_REC;
                    Some(6)
                } else if *jank_time > *scale_time / 4 {
                    self.jank_counter = JANK_REC;
                    Some(4)
                } else if *jank_time > *scale_time / 8 {
                    self.jank_counter = JANK_REC;
                    Some(2)
                } else if *jank_time > *scale_time / 16 {
                    self.jank_counter = JANK_REC;
                    Some(1)
                } else {
                    None
                };

                if result.is_some() {
                    *jank_time = 0;
                }

                result
            })
            .max()
            .unwrap_or_default();

        debug!("jank-level: {level}");

        if level == 0 && self.jank_counter > 0 {
            self.jank_counter -= 1;
            return Ok(());
        }

        self.controller.perf(level, &self.config);

        Ok(())
    }
}
