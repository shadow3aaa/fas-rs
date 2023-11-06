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

#[cfg(debug_assertions)]
use log::debug;

use super::window::FrameWindow;
use crate::config::TargetFps;

const BUFFER_MAX: usize = 144;

#[derive(Debug)]
pub struct Buffer {
    pub target_fps: Option<u32>,
    pub current_fps: Option<f64>,
    target_fps_config: TargetFps,
    timer: Instant,
    pub variance: Option<Duration>,
    pub frametimes: VecDeque<Duration>,
    pub windows: HashMap<u32, FrameWindow>,
    pub last_jank: Option<Instant>,
    pub counter: u8,
}

impl Buffer {
    pub fn new(t: TargetFps) -> Self {
        Self {
            target_fps: None,
            current_fps: None,
            target_fps_config: t,
            timer: Instant::now(),
            variance: None,
            frametimes: VecDeque::with_capacity(BUFFER_MAX),
            windows: HashMap::new(),
            last_jank: None,
            counter: 0,
        }
    }

    pub fn push_frametime(&mut self, d: Duration) {
        let cur_len = self.frametimes.len();
        if cur_len >= BUFFER_MAX {
            self.frametimes.pop_back();
        }

        self.frametimes.push_front(d);

        if cur_len < BUFFER_MAX {
            return;
        }

        if let Some(fps) = self.target_fps {
            self.windows
                .entry(fps)
                .or_insert_with(|| FrameWindow::new(fps, 5))
                .update(d);

            if self.timer.elapsed() * fps > Duration::from_secs(30) {
                self.calculate_fps();
                self.calculate_variance();
                self.timer = Instant::now();
            }
        } else {
            self.calculate_fps();
        }
    }

    fn calculate_fps(&mut self) {
        if self.frametimes.len() < BUFFER_MAX {
            return;
        }

        let avg_time: Duration =
            self.frametimes.iter().sum::<Duration>() / BUFFER_MAX.try_into().unwrap();
        #[cfg(debug_assertions)]
        debug!("avg_time: {avg_time:?}");

        let current_fps = 1.0 / avg_time.as_secs_f64();
        self.current_fps = Some(current_fps);
        #[cfg(debug_assertions)]
        debug!("current_fps: {:.2}", current_fps);

        let target_fpses = match &self.target_fps_config {
            TargetFps::Value(t) => {
                self.target_fps = Some(*t);
                return;
            }
            TargetFps::Array(arr) => arr,
        };

        for target_fps in target_fpses.iter().copied() {
            let target_frametime = Duration::from_secs(1) / (target_fps + 2);
            if avg_time > target_frametime {
                self.target_fps = Some(target_fps);
                return;
            }
        }

        self.target_fps = target_fpses.last().copied();
    }

    fn calculate_variance(&mut self) {
        let Some(target_fps) = self.target_fps else {
            return;
        };

        let cur_len = self.frametimes.len();

        if cur_len < target_fps as usize {
            return;
        }

        let variance = self
            .frametimes
            .iter()
            .take(target_fps as usize)
            .map(|t| (*t * target_fps).as_secs_f64())
            .map(|t| (t - 1.0).powi(2))
            .sum::<f64>()
            / cur_len as f64;
        let variance = Duration::from_secs_f64(variance);

        #[cfg(debug_assertions)]
        debug!("variance: {variance:?}");

        self.variance = Some(variance);
    }
}
