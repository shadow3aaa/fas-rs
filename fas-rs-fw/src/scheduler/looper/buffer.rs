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

const BUFFER_LEN_SECS: usize = 3;

#[derive(Debug)]
pub struct Buffer {
    pub target_fps: Option<u32>,
    pub current_fps: f64,
    pub avg_time: Duration,
    pub frametimes: VecDeque<Duration>,
    pub frame_prepare: Duration,
    pub deviation: f64,
    pub windows: HashMap<u32, FrameWindow>,
    pub last_update: Instant,
    pub acc_frame: f64,
    pub acc_timer: Instant,
    target_fps_config: TargetFps,
    timer: Instant,
}

impl Buffer {
    pub fn new(t: TargetFps) -> Self {
        Self {
            target_fps: None,
            current_fps: 0.0,
            avg_time: Duration::ZERO,
            frametimes: VecDeque::new(),
            frame_prepare: Duration::ZERO,
            deviation: 0.0,
            windows: HashMap::new(),
            last_update: Instant::now(),
            acc_frame: 0.0,
            acc_timer: Instant::now(),
            timer: Instant::now(),
            target_fps_config: t,
        }
    }

    pub fn push_frametime(&mut self, d: Duration) {
        self.last_update = Instant::now();
        self.frame_prepare = Duration::ZERO;

        self.frametimes.push_front(d);
        self.frametimes
            .truncate(self.target_fps.unwrap_or(60) as usize * BUFFER_LEN_SECS);
        self.calculate_current_fps();
        self.calculate_deviation();

        if let Some(fps) = self.target_fps {
            self.windows
                .entry(fps)
                .or_insert_with(|| FrameWindow::new(5))
                .update(d);
        }

        if self.timer.elapsed() >= Duration::from_secs(BUFFER_LEN_SECS as u64) {
            self.timer = Instant::now();
            self.calculate_target_fps();
        }
    }

    pub fn frame_prepare(&mut self) {
        self.frame_prepare = self.last_update.elapsed();
        self.calculate_current_fps();
        self.calculate_target_fps();
    }

    fn calculate_current_fps(&mut self) {
        let avg_time: Duration = self
            .frametimes
            .iter()
            .sum::<Duration>()
            .saturating_add(self.frame_prepare)
            / self.frametimes.len().try_into().unwrap();
        #[cfg(debug_assertions)]
        debug!("avg_time: {avg_time:?}");

        self.avg_time = avg_time;

        let current_fps = 1.0 / avg_time.as_secs_f64();

        #[cfg(debug_assertions)]
        debug!("current_fps: {:.2}", current_fps);

        self.current_fps = current_fps;
    }

    fn calculate_target_fps(&mut self) {
        let target_fpses = match &self.target_fps_config {
            TargetFps::Value(t) => vec![*t],
            TargetFps::Array(arr) => arr.clone(),
        };

        if self.current_fps < (target_fpses[0].saturating_sub(10).max(10)).into() {
            self.target_fps = None;
            return;
        }

        for target_fps in target_fpses.iter().copied() {
            if self.current_fps <= f64::from(target_fps) + 3.0 {
                #[cfg(debug_assertions)]
                debug!(
                    "Matched target_fps: current: {:.2} target_fps: {target_fps}",
                    self.current_fps
                );

                self.target_fps = Some(target_fps);
                return;
            }
        }

        self.target_fps = target_fpses.last().copied();
    }

    pub fn calculate_deviation(&mut self) {
        if self.frametimes.is_empty() {
            return;
        }

        if let Some(target_fps) = self.target_fps {
            let avg = self.avg_time * target_fps;

            let standard_deviation: f64 = {
                let total: f64 = self
                    .frametimes
                    .iter()
                    .copied()
                    .map(|f| f.as_secs_f64() * f64::from(target_fps)) // normalization
                    .map(|f| (f - avg.as_secs_f64()).abs())
                    .sum();
                total / self.frametimes.len() as f64
            };

            #[cfg(debug_assertions)]
            debug!("standard deviation: {standard_deviation:.2}");

            self.deviation = standard_deviation;
        }
    }
}
