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
use std::{collections::VecDeque, time::Duration};

#[derive(Debug)]
pub struct FrameWindow {
    len: usize,
    frametimes: VecDeque<Duration>,
}

impl FrameWindow {
    pub fn new(w: usize) -> Self {
        Self {
            len: w,
            frametimes: VecDeque::with_capacity(w),
        }
    }

    pub fn update(&mut self, d: Duration) {
        self.frametimes.push_front(d);
        self.frametimes.truncate(self.len);
    }

    pub fn last(&mut self) -> Option<&mut Duration> {
        if self.frametimes.len() < self.len {
            None
        } else {
            self.frametimes.front_mut()
        }
    }

    pub fn avg_normalized(&self, target_fps: f64) -> Option<Duration> {
        if self.frametimes.len() < self.len {
            None
        } else {
            let sum = self
                .frametimes
                .iter()
                .copied()
                .sum::<Duration>()
                .mul_f64(target_fps);
            Some(sum / self.len as u32)
        }
    }
}
