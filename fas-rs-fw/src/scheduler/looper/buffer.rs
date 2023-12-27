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

#[derive(Debug)]
pub struct Buffer {
    pub ready: bool,
    pub target_fps: Option<u32>,
    pub current_fps: Option<f64>,
    target_fps_config: TargetFps,
    pub last_update: Instant,
    pub frametimes: VecDeque<Duration>,
    pub deviation: f64,
    pub windows: HashMap<u32, FrameWindow>,
    pub last_jank: Option<Instant>,
    pub last_limit: Option<Instant>,
    pub acc_frame: f64,
    pub acc_timer: Instant,
    timer: Instant,
}

impl Buffer {
    pub fn new(t: TargetFps) -> Self {
        Self {
            ready: false,
            target_fps: None,
            current_fps: None,
            target_fps_config: t,
            last_update: Instant::now(),
            frametimes: VecDeque::new(),
            deviation: 0.0,
            windows: HashMap::new(),
            last_jank: None,
            last_limit: None,
            acc_frame: 0.0,
            acc_timer: Instant::now(),
            timer: Instant::now(),
        }
    }

    pub fn push_frametime(&mut self, d: Duration) {
        self.last_update = Instant::now();

        self.frametimes.push_front(d);
        self.frametimes
            .truncate(self.target_fps.unwrap_or(60) as usize * 3);
        self.calculate_current_fps();
        self.calculate_deviation();

        if let Some(fps) = self.target_fps {
            self.ready = true;
            self.windows
                .entry(fps)
                .or_insert_with(|| FrameWindow::new(5))
                .update(d);
        } else {
            self.ready = false;
        }

        if self.timer.elapsed() >= Duration::from_secs(1) {
            self.calculate_target_fps();
            self.timer = Instant::now();
        }
    }

    fn calculate_current_fps(&mut self) {
        let avg_time: Duration =
            self.frametimes.iter().sum::<Duration>() / self.frametimes.len().try_into().unwrap();
        #[cfg(debug_assertions)]
        debug!("avg_time: {avg_time:?}");

        let current_fps = 1.0 / avg_time.as_secs_f64();
        self.current_fps = Some(current_fps);
        #[cfg(debug_assertions)]
        debug!("current_fps: {:.2}", current_fps);
    }

    fn calculate_target_fps(&mut self) {
        let Some(current_fps) = self.current_fps else {
            self.target_fps = None;
            return;
        };

        let avg_time = Duration::from_secs(1).div_f64(current_fps);

        let target_fpses = match &self.target_fps_config {
            TargetFps::Value(t) => {
                self.target_fps = Some(*t);
                return;
            }
            TargetFps::Array(arr) => {
                if arr.len() == 1 {
                    self.target_fps = arr.first().copied();
                    return;
                }

                arr
            }
        };

        if current_fps < (target_fpses[0].saturating_sub(10).max(10)).into() {
            self.target_fps = None;
            return;
        }

        for target_fps in target_fpses.iter().copied() {
            let target_frametime = Duration::from_secs(1) / (target_fps + 2);
            if avg_time >= target_frametime {
                self.target_fps = Some(target_fps);
                return;
            }
        }

        self.target_fps = target_fpses.last().copied();
    }

    fn calculate_deviation(&mut self) {
        if self.frametimes.is_empty() {
            return;
        }

        let Some(current_fps) = self.current_fps else {
            return;
        };

        if let Some(target_fps) = self.target_fps {
            if current_fps < f64::from(target_fps) - 1.0 {
                return;
            }

            let standard_deviation: f64 = {
                let total: f64 = self
                    .frametimes
                    .iter()
                    .copied()
                    .map(|f| f.as_secs_f64() * f64::from(target_fps)) // normalization
                    .map(|f| (f - 1.0).abs().powi(2))
                    .sum();
                let variance = total / self.frametimes.len() as f64;
                variance.sqrt()
            };

            #[cfg(debug_assertions)]
            debug!("standard deviation: {standard_deviation:.2}");

            self.deviation = standard_deviation;
        }
    }
}
