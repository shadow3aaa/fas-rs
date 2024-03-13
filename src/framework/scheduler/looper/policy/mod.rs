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
mod extract;

use std::time::{Duration, Instant};

#[cfg(debug_assertions)]
use log::debug;

use super::Buffer;
use crate::framework::Mode;

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
    pub fn normal_event(&mut self, mode: Mode) -> Option<NormalEvent> {
        let policy_data = PolicyData::extract(self, mode)?;

        #[cfg(debug_assertions)]
        debug!("policy data: {policy_data:?}");

        Some(self.frame_analyze(policy_data))
    }

    pub fn jank_event(&self, mode: Mode) -> Option<JankEvent> {
        let policy_data = PolicyData::extract(self, mode)?;

        Some(Self::jank_analyze(policy_data))
    }

    fn frame_analyze(&mut self, policy_data: PolicyData) -> NormalEvent {
        self.acc_frame.acc(policy_data.normalized_unit_frame);

        if self.acc_timer.elapsed() * policy_data.target_fps < Duration::from_secs(1) {
            return NormalEvent::None;
        }

        self.acc_timer = Instant::now();

        let timeout = self.acc_frame.timeout_dur();
        let result = if timeout > Duration::ZERO {
            #[cfg(debug_assertions)]
            debug!("unit small jank, timeout: {timeout:?}");

            NormalEvent::Release
        } else if self.recent_timeout(policy_data) {
            NormalEvent::None
        } else {
            #[cfg(debug_assertions)]
            debug!("no jank, timeout: {timeout:?}");

            NormalEvent::Restrictable
        };

        self.acc_frame.reset();
        result
    }

    fn recent_timeout(&self, policy_data: PolicyData) -> bool {
        let target_fps = policy_data.target_fps_prefixed;
        self.frametimes
            .iter()
            .take(10)
            .copied()
            .map(|f| f * target_fps)
            .sum::<Duration>()
            >= Duration::from_secs(10)
    }

    fn jank_analyze(policy_data: PolicyData) -> JankEvent {
        let target_fps = policy_data.target_fps;
        let last_frame = policy_data.normalized_last_frame;
        let avg_fps = policy_data.current_fps;

        if avg_fps + 5.0 <= target_fps.into() || last_frame >= Duration::from_millis(5000) {
            #[cfg(debug_assertions)]
            debug!("big jank, last frame: {last_frame:?}");

            JankEvent::BigJank
        } else if avg_fps + 3.0 <= target_fps.into() || last_frame >= Duration::from_millis(1700) {
            #[cfg(debug_assertions)]
            debug!("jank, last frame: {last_frame:?}");

            JankEvent::Jank
        } else {
            JankEvent::None
        }
    }
}
