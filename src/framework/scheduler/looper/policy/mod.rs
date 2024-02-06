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

use std::time::{Duration, Instant};

#[cfg(debug_assertions)]
use log::debug;

use super::Buffer;
use crate::framework::{Config, Mode};

use config::PolicyConfig;
use extract::PolicyData;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub enum NormalEvent {
    Restrictable,
    None,
    Release,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub enum JankEvent {
    None,
    Jank,
    BigJank,
}

impl Buffer {
    pub fn normal_event(&mut self, config: &Config, mode: Mode) -> NormalEvent {
        let config = PolicyConfig::new(config, mode, self);
        let Some(policy_data) = PolicyData::extract(self) else {
            return NormalEvent::Release;
        };

        #[cfg(debug_assertions)]
        {
            debug!("policy config: {config:?}");
            debug!("policy data: {policy_data:?}");
        }

        self.frame_analyze(config, policy_data)
    }

    pub fn jank_event(&mut self, config: &Config, mode: Mode) -> JankEvent {
        let config = PolicyConfig::new(config, mode, self);
        let Some(policy_data) = PolicyData::extract(self) else {
            return JankEvent::BigJank;
        };

        #[cfg(debug_assertions)]
        {
            debug!("policy config: {config:?}");
            debug!("policy data: {policy_data:?}");
        }

        Self::jank_analyze(config, policy_data)
    }

    fn frame_analyze(&mut self, config: PolicyConfig, policy_data: PolicyData) -> NormalEvent {
        self.acc_frame.acc(policy_data.normalized_unit_frame);

        if self.acc_timer.elapsed() * policy_data.target_fps < Duration::from_secs(1) {
            return NormalEvent::None;
        }

        let result = if self.acc_frame.as_duration() >= config.scale {
            #[cfg(debug_assertions)]
            debug!("JANK: unit jank");

            NormalEvent::Release
        } else {
            let stability = self.calculate_stability();
            let mut stability = stability.clamp(1.0, 10.0);
            if stability.is_nan() {
                stability = 10.0;
            }

            if self.acc_timer.elapsed() * policy_data.target_fps
                < Duration::from_secs_f64(stability)
            {
                return NormalEvent::None;
            }

            #[cfg(debug_assertions)]
            debug!("JANK: no jank");

            NormalEvent::Restrictable
        };

        self.acc_frame.reset();
        self.acc_timer = Instant::now();

        result
    }

    fn jank_analyze(config: PolicyConfig, policy_data: PolicyData) -> JankEvent {
        let diff_avg = policy_data
            .normalized_avg_frame
            .saturating_sub(Duration::from_secs(1));
        let last_frame = policy_data.normalized_last_frame;

        if last_frame >= Duration::from_millis(1700) || diff_avg >= config.big_jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: big jank");

            JankEvent::BigJank
        } else if last_frame >= Duration::from_millis(5000) || diff_avg >= config.jank_scale {
            #[cfg(debug_assertions)]
            debug!("JANK: simp jank");

            JankEvent::Jank
        } else {
            JankEvent::None
        }
    }
}
