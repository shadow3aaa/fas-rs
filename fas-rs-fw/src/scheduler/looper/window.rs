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
    target_fps: u32,
    len: usize,
    frametimes: VecDeque<Duration>,
}

impl FrameWindow {
    pub fn new(t: u32, w: usize) -> Self {
        Self {
            target_fps: t,
            len: w,
            frametimes: VecDeque::with_capacity(t as usize),
        }
    }

    pub fn update(&mut self, d: Duration) {
        let d = d * self.target_fps;
        self.frametimes.push_front(d);
        self.frametimes.truncate(self.target_fps as usize);
    }

    pub fn last(&mut self) -> Option<&mut Duration> {
        if self.frametimes.len() < self.target_fps as usize {
            None
        } else {
            self.frametimes.front_mut()
        }
    }

    pub fn avg(&self) -> Option<Duration> {
        if self.frametimes.len() < self.len {
            None
        } else {
            let sum = self.frametimes.iter().take(self.len).sum::<Duration>();
            Some(sum / self.len as u32)
        }
    }
}
