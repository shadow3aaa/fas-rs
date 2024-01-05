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
pub mod config;
mod extract;

use std::time::Instant;

#[cfg(debug_assertions)]
use log::debug;

use super::{Buffer, Looper};
use crate::{Mode, PerformanceController};

use config::PolicyConfig;
use extract::PolicyData;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub enum Event {
    None,
    Restrictable,
    Release,
    Jank,
    BigJank,
}

impl<P: PerformanceController> Looper<P> {
    pub fn get_event(mode: Mode, buffer: &mut Buffer) -> Event {
        let config = PolicyConfig::new(mode, buffer);
        let Some(policy_data) = PolicyData::extract(buffer) else {
            return Event::None;
        };

        #[cfg(debug_assertions)]
        {
            debug!("policy config: {config:?}");
            debug!("policy data: {policy_data:?}");
        }

        let result = Self::frame_analyze(buffer, config, policy_data);
        if let Some(event) = Self::jank_analyze(policy_data) {
            return event;
        }

        result
    }

    fn frame_analyze(buffer: &mut Buffer, config: PolicyConfig, policy_data: PolicyData) -> Event {
        let diff = policy_data.normalized_avg_frame.as_secs_f64() - 1.0;
        buffer.acc_frame += diff;

        if buffer.acc_timer.elapsed() * policy_data.target_fps < config.acc_dur {
            return Event::None;
        }

        let scale = config.scale.as_secs_f64();
        let result = if buffer.acc_frame >= scale {
            #[cfg(debug_assertions)]
            debug!("JANK: unit jank");

            Event::Release
        } else {
            #[cfg(debug_assertions)]
            debug!("JANK: no jank");

            Event::Restrictable
        };

        buffer.acc_frame = 0.0;
        buffer.acc_timer = Instant::now();
        buffer.calculate_deviation();

        result
    }

    fn jank_analyze(policy_data: PolicyData) -> Option<Event> {
        if policy_data.normalized_frame > policy_data.normalized_big_jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: big jank");

            Some(Event::BigJank)
        } else if policy_data.normalized_frame > policy_data.normalized_jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: simp jank");

            Some(Event::Jank)
        } else {
            None
        }
    }
}
