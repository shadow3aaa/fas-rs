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

use super::{buffer::calculate::StabilityLevel, Buffer};
use crate::framework::Mode;

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
    pub fn normal_event(&mut self, mode: Mode) -> NormalEvent {
        let Some(policy_data) = PolicyData::extract(self, mode) else {
            return NormalEvent::Release;
        };

        #[cfg(debug_assertions)]
        debug!("policy data: {policy_data:?}");

        self.frame_analyze(policy_data)
    }

    pub fn jank_event(&mut self, mode: Mode) -> JankEvent {
        let config = PolicyConfig::new(mode);
        let Some(policy_data) = PolicyData::extract(self, mode) else {
            return JankEvent::BigJank;
        };

        #[cfg(debug_assertions)]
        {
            debug!("policy config: {config:?}");
            debug!("policy data: {policy_data:?}");
        }

        Self::jank_analyze(config, policy_data)
    }

    fn frame_analyze(&mut self, policy_data: PolicyData) -> NormalEvent {
        self.acc_frame.acc(policy_data.normalized_unit_frame);

        if self.acc_timer.elapsed() * policy_data.target_fps < Duration::from_secs(1) {
            return NormalEvent::None;
        }

        let limit_delay = match self.calculate_stability() {
            StabilityLevel::High => Duration::from_millis(150),
            StabilityLevel::Mid => Duration::from_millis(250),
            StabilityLevel::Low => Duration::from_millis(450),
        };

        let timeout = self.acc_frame.timeout_dur();
        let result = if timeout > Duration::ZERO {
            #[cfg(debug_assertions)]
            debug!("unit small jank, timeout: {timeout:?}");

            NormalEvent::Release
        } else {
            if self.acc_timer.elapsed() < limit_delay {
                return NormalEvent::None;
            }

            #[cfg(debug_assertions)]
            debug!("no jank, timeout: {timeout:?}");

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
            debug!("big jank, last frame: {last_frame:?}, timeout: {diff_avg:?}");

            JankEvent::BigJank
        } else if last_frame >= Duration::from_millis(5000) || diff_avg >= config.jank_scale {
            #[cfg(debug_assertions)]
            debug!("jank, last frame: {last_frame:?}, timeout: {diff_avg:?}");

            JankEvent::Jank
        } else {
            JankEvent::None
        }
    }
}
