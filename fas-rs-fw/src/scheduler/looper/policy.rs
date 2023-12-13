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
    ReleaseMax,
    Release,
    Limit,
    None,
}

impl<P: PerformanceController> Looper<P> {
    pub fn get_event(buffer: &mut Buffer, config: &Config, node: &mut Node) -> Result<Event> {
        let Some(target_fps) = buffer.target_fps else {
            return Ok(Event::ReleaseMax);
        };

        let mode = node.get_mode()?;
        let policy = Self::policy_config(mode, buffer, config);
        #[cfg(debug_assertions)]
        debug!("mode policy: {policy:?}");

        let Some(window) = buffer.windows.get_mut(&target_fps) else {
            return Ok(Event::ReleaseMax);
        };

        let Some(normalized_avg_frame) = window.avg() else {
            return Ok(Event::ReleaseMax);
        };

        let Some(normalized_frame) = window.last() else {
            return Ok(Event::ReleaseMax);
        };

        let normalized_big_jank_scale = Duration::from_secs(5);
        let normalized_jank_scale = Duration::from_millis(1700);

        #[cfg(debug_assertions)]
        {
            debug!("target_fps: {target_fps}");
            debug!("normalized frametime: {normalized_frame:?}");
            debug!("normalized avg frametime: {normalized_avg_frame:?}");
            debug!("simple jank scale: {normalized_jank_scale:?}");
            debug!("big jank scale: {normalized_big_jank_scale:?}");
        }

        if *normalized_frame > normalized_big_jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: big jank");

            buffer.limit_acc = Duration::ZERO;
            Ok(Event::ReleaseMax)
        } else if *normalized_frame > normalized_jank_scale {
            if let Some(stamp) = buffer.last_jank {
                let normalized_last_jank = stamp.elapsed() * target_fps;
                if normalized_last_jank < Duration::from_secs(30) {
                    return Ok(Event::None);
                }
            } // one jank is allow in 30 frames at least

            buffer.last_jank = Some(Instant::now());
            buffer.limit_acc = Duration::ZERO;

            #[cfg(debug_assertions)]
            debug!("JANK: simp jank");

            Ok(Event::Release)
        } else if normalized_avg_frame > Duration::from_secs(1) {
            let diff = normalized_avg_frame - Duration::from_secs(1);

            if buffer.release_acc < policy.scale {
                buffer.release_acc = buffer.release_acc.saturating_add(diff);
                return Ok(Event::None);
            }

            buffer.release_acc = buffer
                .release_acc
                .saturating_sub(policy.scale)
                .min(Duration::from_secs(1));

            #[cfg(debug_assertions)]
            debug!("JANK: unit jank");

            Ok(Event::Release)
        } else if normalized_avg_frame <= Duration::from_secs(1) {
            let diff = Duration::from_secs(1) - normalized_avg_frame;

            if buffer.limit_acc < policy.scale
                && buffer.limit_counter < (target_fps / 6).max(5) as usize
            {
                buffer.limit_acc = buffer.limit_acc.saturating_add(diff);
                buffer.limit_counter += 1;
                return Ok(Event::None);
            }

            buffer.limit_counter = 0;
            buffer.limit_acc = buffer
                .limit_acc
                .saturating_sub(policy.scale)
                .min(Duration::from_secs(1));

            /* if let Some(stamp) = buffer.last_limit {
                let normalized_last_limit = stamp.elapsed() * target_fps;
                if normalized_last_limit < Duration::from_secs(3) {
                    return Ok(Event::None);
                }
            } // one limit is allow in 3 frames at least

            buffer.last_limit = Some(Instant::now()); */

            #[cfg(debug_assertions)]
            debug!("JANK: no jank");

            Ok(Event::Limit)
        } else {
            Ok(Event::None)
        }
    }
}
