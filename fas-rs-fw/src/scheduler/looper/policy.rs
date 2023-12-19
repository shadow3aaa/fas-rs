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

#[cfg(debug_assertions)]
use log::debug;

use super::{Buffer, Looper};
use crate::{error::Result, node::Node, Config, PerformanceController};

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub enum Event {
    None,
    Limit,
    Release,
    ReleaseMax,
}

impl<P: PerformanceController> Looper<P> {
    pub fn get_event(buffer: &mut Buffer, config: &Config, node: &mut Node) -> Result<Event> {
        let mode = node.get_mode()?;
        let policy = Self::policy_config(mode, buffer, config);
        #[cfg(debug_assertions)]
        debug!("mode policy: {policy:?}");

        let Some(target_fps) = buffer.target_fps else {
            return Ok(Event::ReleaseMax);
        };
        let target_fps_offseted = f64::from(target_fps) - policy.target_fps_offset;

        let Some(window) = buffer.windows.get_mut(&target_fps) else {
            return Ok(Event::ReleaseMax);
        };

        let Some(normalized_avg_frame) = window.avg_normalized(target_fps_offseted) else {
            return Ok(Event::ReleaseMax);
        };

        let Some(last_frame) = window.last() else {
            return Ok(Event::ReleaseMax);
        };
        let normalized_frame = last_frame.mul_f64(target_fps_offseted);

        let normalized_big_jank_scale = Duration::from_secs(5);
        let normalized_jank_scale = Duration::from_millis(1700);

        #[cfg(debug_assertions)]
        {
            debug!("target_fps: {target_fps}");
            debug!("target_fps_offseted: {:.2}", target_fps_offseted);
            debug!("normalized frametime: {normalized_frame:?}");
            debug!("normalized avg frametime: {normalized_avg_frame:?}");
            debug!("simple jank scale: {normalized_jank_scale:?}");
            debug!("big jank scale: {normalized_big_jank_scale:?}");
        }

        if normalized_frame > normalized_big_jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: big jank");

            if buffer.diff_secs_acc < 0.0 {
                buffer.diff_secs_acc = 0.0;
            }

            return Ok(Event::ReleaseMax);
        } else if normalized_frame > normalized_jank_scale {
            if let Some(stamp) = buffer.last_jank {
                if stamp.elapsed() * target_fps < Duration::from_secs(30) {
                    return Ok(Event::None);
                }
            }

            buffer.last_jank = Some(Instant::now());

            if buffer.diff_secs_acc < 0.0 {
                buffer.diff_secs_acc = 0.0;
            }

            #[cfg(debug_assertions)]
            debug!("JANK: simp jank");

            return Ok(Event::Release);
        }

        let diff = normalized_avg_frame.as_secs_f64() - 1.0;
        buffer.diff_secs_acc += diff;

        let scale = policy.scale.as_secs_f64();
        if buffer.diff_secs_acc >= scale {
            buffer.diff_secs_acc = 0.0;

            #[cfg(debug_assertions)]
            debug!("JANK: unit jank");

            Ok(Event::Release)
        } else if buffer.diff_secs_acc <= -scale {
            buffer.diff_secs_acc = 0.0;

            #[cfg(debug_assertions)]
            debug!("JANK: no jank");

            Ok(Event::Limit)
        } else {
            Ok(Event::None)
        }
    }
}
