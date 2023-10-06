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
    collections::VecDeque,
    time::Duration,
};

/* 尽管部分游戏渲染时间长度是离散的，但必然在某个长度上的均值贴合标准渲染时间
*  AutoFrameWindow的本质是一个自动变长的简单均值滑动窗口，它在取均值出现小于标准渲染时间时自动变长
*  因为这意味着还有帧出现在AutoFrameWindow的控制范围之外 */
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
            frametimes: VecDeque::with_capacity(w),
        }
    }

    pub fn update(&mut self, d: Duration) {
        let d = d * self.target_fps;
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

    pub fn get_avg(&self) -> Option<Duration> {
        if self.frametimes.len() < self.len {
            None
        } else {
            let sum = self.frametimes.iter().sum::<Duration>();
            Some(sum / self.len as u32)
        }
    }
}
