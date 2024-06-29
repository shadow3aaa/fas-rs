// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::Duration;

#[cfg(debug_assertions)]
use log::debug;

use super::buffer::Buffer;
use crate::framework::prelude::*;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub struct FrameEvent {
    pub frame: Duration,
    pub target: Duration,
}

impl Buffer {
    pub fn event(&self, config: &Config, mode: Mode) -> Option<FrameEvent> {
        let normalized_last_frame = self.frametimes.front().copied()? * self.target_fps?;

        #[cfg(debug_assertions)]
        debug!("normalized_last_frame: {normalized_last_frame:?}");

        let frame = normalized_last_frame;
        let margin = config.mode_config(mode).margin;
        let margin = Duration::from_millis(margin);
        let target = Duration::from_secs(1) + margin;

        Some(FrameEvent { frame, target })
    }
}
