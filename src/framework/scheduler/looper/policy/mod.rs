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

use std::time::Duration;

#[cfg(debug_assertions)]
use log::debug;

use super::buffer::Buffer;
use crate::framework::prelude::*;

use extract::PolicyData;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub enum NormalEvent {
    Restrictable(Duration, Duration),
    Release(Duration, Duration),
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub enum JankEvent {
    None,
    Jank,
    BigJank,
}

impl Buffer {
    pub fn normal_event(&self, config: &Config, mode: Mode) -> Option<NormalEvent> {
        let policy_data = PolicyData::extract(self)?;

        #[cfg(debug_assertions)]
        debug!("policy data: {policy_data:?}");

        Some(Self::frame_analyze(policy_data, config, mode))
    }

    pub fn jank_event(&self) -> Option<JankEvent> {
        let policy_data = PolicyData::extract(self)?;
        Some(Self::jank_analyze(policy_data))
    }

    fn frame_analyze(policy_data: PolicyData, config: &Config, mode: Mode) -> NormalEvent {
        let frame = policy_data.normalized_last_frame;
        let margin = config.mode_config(mode).margin;
        let margin = Duration::from_millis(margin);
        let target = Duration::from_secs(1) + margin;

        if frame > target {
            #[cfg(debug_assertions)]
            debug!("unit small jank, frame: {frame:?}");

            NormalEvent::Release(frame, target)
        } else {
            #[cfg(debug_assertions)]
            debug!("no jank, frame: {frame:?}");

            NormalEvent::Restrictable(frame, target)
        }
    }

    fn jank_analyze(policy_data: PolicyData) -> JankEvent {
        let target_fps = policy_data.target_fps;
        let last_frame = policy_data.normalized_last_frame;
        let avg_fps = policy_data.current_fps;

        if avg_fps <= (target_fps * 115 / 120).into() || last_frame >= Duration::from_millis(5000) {
            #[cfg(debug_assertions)]
            debug!("big jank, last frame: {last_frame:?}");

            JankEvent::BigJank
        } else if avg_fps <= (target_fps * 117 / 120).into()
            || last_frame >= Duration::from_millis(1700)
        {
            #[cfg(debug_assertions)]
            debug!("jank, last frame: {last_frame:?}");

            JankEvent::Jank
        } else {
            JankEvent::None
        }
    }
}
