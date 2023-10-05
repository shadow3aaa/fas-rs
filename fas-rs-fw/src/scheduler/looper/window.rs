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
    time::{Duration, Instant},
};

use log::debug;

/* 尽管部分游戏渲染时间长度是离散的，但必然在某个长度上的均值贴合标准渲染时间
*  AutoFrameWindow的本质是一个自动变长的简单均值滑动窗口，它在取均值出现小于标准渲染时间时自动变长
* 因为这意味着还有帧出现在AutoFrameWindow的控制范围之外 */
#[derive(Debug)]
pub struct AutoFrameWindow {
    target_fps: u32,
    win_len: usize,
    last_reduce: Instant,
    pub frametimes: VecDeque<Duration>,
}

pub enum FrameWindowData {
    Avg(Duration),
    NotEnough,
}

impl AutoFrameWindow {
    pub fn new(t: u32) -> Self {
        Self {
            target_fps: t,
            win_len: 5,
            last_reduce: Instant::now(),
            frametimes: VecDeque::with_capacity(5),
        }
    }

    pub fn update(&mut self, d: Duration) -> FrameWindowData {
        let d = d * self.target_fps;
        self.frametimes.push_front(d);
        self.frametimes.truncate(self.win_len);

        let Some(avg) = self.get_avg() else {
            return FrameWindowData::NotEnough;
        };

        // let normalized_last_reduce = self.last_reduce.elapsed() * self.target_fps;
        if self.last_reduce.elapsed() > Duration::from_secs(1) {
            self.win_len = self.win_len.saturating_sub(1);
            self.win_len = self.win_len.max(5);

            self.last_reduce = Instant::now();

            debug!(
                "Auto resize the frame window length, current length: {}",
                self.win_len
            );
        }

        if avg < Duration::from_millis(950) {
            self.win_len = self.win_len.saturating_add(1);
            self.win_len = self.win_len.min(self.target_fps as usize / 2);

            debug!(
                "Auto resize the frame window length, current length: {}",
                self.win_len
            );
        }

        FrameWindowData::Avg(avg)
    }

    fn get_avg(&self) -> Option<Duration> {
        let cur_len = self.frametimes.len();

        if cur_len < self.win_len {
            None
        } else {
            let sum = self.frametimes.iter().sum::<Duration>();
            Some(sum / cur_len as u32)
        }
    }
}
