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
use std::time::Duration;

use super::super::buffer::Buffer;
use crate::framework::Mode;

use smallvec::SmallVec;

#[derive(Debug, Clone, Copy)]
pub struct PolicyData {
    pub target_fps: u32,
    pub normalized_last_frame: Duration,
    pub normalized_unit_frame: Duration,
    pub normalized_avg_frame: Duration,
}

impl PolicyData {
    pub fn extract(buffer: &Buffer, mode: Mode) -> Option<Self> {
        let target_fps = buffer.target_fps?;
        let target_fps_prefixed = match mode {
            Mode::Powersave | Mode::Balance => target_fps * 115 / 120,
            Mode::Performance | Mode::Fast => target_fps * 117 / 120,
        };

        let frames: SmallVec<[Duration; 5]> = buffer.frametimes.iter().copied().take(5).collect();
        let len = frames.len();
        let frame = frames.into_iter().sum::<Duration>() / len as u32;

        let normalized_last_frame = buffer.frametimes.front().copied()? * target_fps_prefixed;
        let normalized_avg_frame = buffer.avg_time * target_fps;
        let normalized_unit_frame = frame * target_fps_prefixed;

        Some(Self {
            target_fps,
            normalized_last_frame,
            normalized_unit_frame,
            normalized_avg_frame,
        })
    }
}
