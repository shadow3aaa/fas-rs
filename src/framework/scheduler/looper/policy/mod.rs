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

use super::Buffer;
use crate::Mode;

use config::PolicyConfig;
use extract::PolicyData;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub enum Event {
    Restrictable,
    None,
    Release,
    Jank,
    BigJank,
}

impl Buffer {
    pub fn event(&mut self, mode: Mode) -> Event {
        let config = PolicyConfig::new(mode, self);
        let Some(policy_data) = PolicyData::extract(self) else {
            return Event::None;
        };

        #[cfg(debug_assertions)]
        {
            debug!("policy config: {config:?}");
            debug!("policy data: {policy_data:?}");
        }

        let result = self.frame_analyze(config, policy_data);
        if let Some(event) = self.jank_analyze(policy_data) {
            return event;
        }

        result
    }

    fn frame_analyze(&mut self, config: PolicyConfig, policy_data: PolicyData) -> Event {
        let diff = policy_data.normalized_avg_frame.as_secs_f64() - 1.0;
        self.acc_frame += diff;

        if self.acc_timer.elapsed() * policy_data.target_fps < config.acc_dur {
            return Event::None;
        }

        let scale = config.scale.as_secs_f64();
        let result = if self.acc_frame >= scale {
            #[cfg(debug_assertions)]
            debug!("JANK: unit jank");

            Event::Release
        } else {
            #[cfg(debug_assertions)]
            debug!("JANK: no jank");

            Event::Restrictable
        };

        self.acc_frame = 0.0;
        self.acc_timer = Instant::now();

        result
    }

    fn jank_analyze(&mut self, policy_data: PolicyData) -> Option<Event> {
        if policy_data.normalized_frame > policy_data.normalized_big_jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: big jank");

            self.acc_frame = 0.0;

            Some(Event::BigJank)
        } else if policy_data.normalized_frame > policy_data.normalized_jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: simp jank");

            self.acc_frame = 0.0;

            Some(Event::Jank)
        } else {
            None
        }
    }
}
