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
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use log::debug;

use super::window::FrameWindow;
use crate::config::TargetFps;

const BUFFER_MAX: usize = 144;

#[derive(Debug)]
pub struct Buffer {
    pub target_fps_config: TargetFps,
    pub target_fps: Option<u32>,
    pub frametimes: VecDeque<Duration>,
    pub windows: HashMap<u32, FrameWindow>,
    pub last_jank: Option<Instant>,
    pub last_limit: Option<Instant>,
    pub counter: u8,
}

impl Buffer {
    pub fn new(t: TargetFps) -> Self {
        Self {
            target_fps_config: t,
            target_fps: None,
            frametimes: VecDeque::with_capacity(BUFFER_MAX),
            windows: HashMap::new(),
            last_jank: None,
            last_limit: None,
            counter: 0,
        }
    }

    pub fn push_frametime(&mut self, d: Duration) {
        if self.frametimes.len() >= BUFFER_MAX {
            self.frametimes.pop_back();
        }

        self.frametimes.push_front(d);

        if let Some(target_fps) = self.calculate_fps() {
            self.target_fps = Some(target_fps);
            self.windows
                .entry(target_fps)
                .or_insert_with(|| FrameWindow::new(target_fps, 5))
                .update(d)
        } else {
            self.target_fps = None;
        }
    }

    fn calculate_fps(&self) -> Option<u32> {
        if self.frametimes.len() < BUFFER_MAX {
            return None;
        }

        if let TargetFps::Value(t) = self.target_fps_config {
            return Some(t);
        }

        let avg_time: Duration =
            self.frametimes.iter().sum::<Duration>() / BUFFER_MAX.try_into().unwrap();

        debug!("avg_time: {avg_time:?}");

        if avg_time < Duration::from_micros(8130) {
            // 123fps
            Some(144)
        } else if avg_time < Duration::from_micros(10638) {
            // 94 fps
            Some(120)
        } else if avg_time < Duration::from_micros(16129) {
            // 62 fps
            Some(90)
        } else if avg_time < Duration::from_micros(20408) {
            // 49 fps
            Some(60)
        } else if avg_time < Duration::from_micros(21739) {
            // 46 fps
            Some(48)
        } else if avg_time < Duration::from_micros(32258) {
            // 31 fps
            Some(45)
        } else if avg_time < Duration::from_micros(50000) {
            // 20 fps
            Some(30)
        } else {
            None
        }
    }
}
