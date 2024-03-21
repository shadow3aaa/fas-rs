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

#[derive(Debug, Clone, Copy)]
pub struct PolicyData {
    pub target_fps: u32,
    pub target_fps_prefixed: u32,
    pub current_fps: f64,
    pub normalized_last_frame: Duration,
}

impl PolicyData {
    pub fn extract(buffer: &Buffer) -> Option<Self> {
        let target_fps = buffer.target_fps?;
        let current_fps = buffer.current_fps;
        let target_fps_prefixed = target_fps * 119 / 120;

        let target_fps = buffer
            .current_fpses
            .iter()
            .copied()
            .map(|f| f as u32)
            .max()
            .unwrap_or(target_fps)
            .clamp(target_fps_prefixed, target_fps);

        let normalized_last_frame = buffer.frametimes.front().copied()? * target_fps;

        Some(Self {
            target_fps,
            target_fps_prefixed,
            current_fps,
            normalized_last_frame,
        })
    }
}
