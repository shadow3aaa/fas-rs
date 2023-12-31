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
    pub normalized_big_jank_scale: Duration,
    pub normalized_jank_scale: Duration,
    pub normalized_frame: Duration,
    pub normalized_avg_frame: Duration,
}

impl PolicyData {
    pub fn extract(buffer: &Buffer) -> Option<Self> {
        let target_fps = buffer.target_fps?;
        let window = buffer.windows.get(&target_fps)?;

        let window_fps = window.fps().max(f64::from(target_fps));
        let normalized_prepare_frame = buffer.frame_prepare * target_fps;
        let normalized_avg_frame = window.avg_normalized(window_fps)? + normalized_prepare_frame;
        let last_frame = buffer.frametimes.front().copied()?;
        let normalized_frame = last_frame * target_fps + normalized_prepare_frame;

        let normalized_big_jank_scale = Duration::from_secs(5);
        let normalized_jank_scale = Duration::from_millis(1700);

        Some(Self {
            target_fps,
            normalized_big_jank_scale,
            normalized_jank_scale,
            normalized_frame,
            normalized_avg_frame,
        })
    }
}
