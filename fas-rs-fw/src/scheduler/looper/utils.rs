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
use std::{
    collections::hash_map::Entry,
    time::{Duration, Instant},
};

use log::{debug, info};

use super::{super::FasData, Buffer, Looper, BUFFER_MAX};
use crate::{config::TargetFps, error::Result, PerformanceController};

const NORMALIZED_BASIC_JANK_SCALE: Duration = Duration::from_millis(1700);
const NORMALIZED_RECACULATE_JANK_SCALE: Duration = Duration::from_secs(BUFFER_MAX as u64);

impl<P: PerformanceController> Looper<P> {
    /* 检查是否为顶层应用，并且删除不是顶层应用的buffer **/
    pub fn retain_topapp(&mut self) -> Result<()> {
        self.buffers
            .retain(|(_, p), _| self.topapp_checker.is_topapp(*p));

        if self.buffers.is_empty() && self.started {
            self.controller.init_default(&self.config)?;
            self.started = false;
            return Ok(());
        } else if !self.buffers.is_empty() && !self.started {
            self.controller.init_game(&self.config)?;
            self.started = true;
        }

        Ok(())
    }

    pub fn buffer_update(&mut self, d: &FasData) {
        if !self.topapp_checker.is_topapp(d.pid) || d.frametime.is_zero() {
            return;
        } else if d.target_fps == TargetFps::Value(0) {
            panic!("Target fps must be bigger than zero");
        }

        let process = (d.pkg.clone(), d.pid);
        let target_fps = d.target_fps;

        match self.buffers.entry(process) {
            Entry::Occupied(mut o) => o.get_mut().push_frametime(d.frametime),
            Entry::Vacant(v) => {
                info!("Loaded fas on {:?}", v.key());
                let mut buffer = Buffer::new(target_fps);
                buffer.push_frametime(d.frametime);
                v.insert(buffer);
            }
        }
    }

    pub fn calculate_fps(buffer: &Buffer) -> Option<u32> {
        if buffer.frametimes.len() < BUFFER_MAX {
            return None;
        }

        let avg_time: Duration =
            buffer.frametimes.iter().sum::<Duration>() / BUFFER_MAX.try_into().unwrap();

        debug!("avg_time: {avg_time:?}");

        if avg_time < Duration::from_micros(8130) {
            // 123fps
            Some(144)
        } else if avg_time < Duration::from_micros(10638) {
            // 94 fps
            Some(120)
        } else if avg_time < Duration::from_micros(16129) {
            // 62 fps
            Some(90)
        } else if avg_time < Duration::from_micros(20408) {
            // 49 fps
            Some(60)
        } else if avg_time < Duration::from_micros(21739) {
            // 46 fps
            Some(48)
        } else if avg_time < Duration::from_micros(32258) {
            // 31 fps
            Some(45)
        } else if avg_time < Duration::from_micros(50000) {
            // 20 fps
            Some(30)
        } else {
            None
        }
    }

    pub fn calculate_jank_scale(buffer: &mut Buffer, target_fps: u32) {
        match buffer.jank_scale.entry(target_fps) {
            Entry::Occupied(mut o) => {
                let (normalized_scale, stamp) = o.get_mut();
                let normalized_elapsed_time = stamp.elapsed() * target_fps;

                if buffer.frametimes.len() < BUFFER_MAX
                    || normalized_elapsed_time < NORMALIZED_RECACULATE_JANK_SCALE
                {
                    return;
                }
                *stamp = Instant::now();

                let min_frametime = buffer.frametimes.iter().copied().min().unwrap();
                if min_frametime > Duration::from_secs(1) / target_fps {
                    return;
                }

                let normalized_min_frametime = min_frametime * target_fps;
                let normalized_fix_frametime =
                    Duration::from_secs(1).saturating_sub(normalized_min_frametime);
                let normalized_new_scale = NORMALIZED_BASIC_JANK_SCALE + normalized_fix_frametime;

                *normalized_scale = normalized_new_scale.min(Duration::from_millis(2500));
            }
            Entry::Vacant(v) => {
                v.insert((NORMALIZED_BASIC_JANK_SCALE, Instant::now()));
            }
        }
    }
}
