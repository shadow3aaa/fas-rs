/* Copyright 2023 shadow3aaa@gitbub.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License. */
//! 放置[`fas_rs_fw::VirtualFrameSensor`]的实现
//! 还有一些用于这些实现的实用函数

pub mod dumpsys;
pub mod mtk_fpsgo;

use std::{
    cell::Cell,
    time::{Duration, Instant},
};

use fas_rs_fw::prelude::*;

// 如果传感器实现实际读取的是帧vsync间隔而不是真正的帧渲染时间时需要用这个修正
// 为了方便实现trait已经做了内部可变处理
pub struct IgnoreFrameTime {
    refresh_rate: Cell<Option<Fps>>,
    timer: Cell<Instant>,
}

impl IgnoreFrameTime {
    fn new() -> Self {
        let timer = Cell::new(Instant::now());
        let refresh_rate = Cell::new(Self::get_refresh_rate());
        Self {
            refresh_rate,
            timer,
        }
    }

    fn ign(&self, frametime: FrameTime, target_fps: TargetFps) -> FrameTime {
        let now = Instant::now();
        if now - self.timer.get() >= Duration::from_secs(5) {
            self.timer.set(now);
            self.refresh_rate.set(Self::get_refresh_rate());
        }

        if let Some(refresh_rate) = self.refresh_rate.get() {
            if refresh_rate != target_fps {
                let target_frametime = Duration::from_secs(1) / target_fps;
                let refresh_time = Duration::from_secs(1) / refresh_rate;
                let total_ign_time = target_frametime.saturating_add(refresh_time);

                if frametime.as_millis() >= total_ign_time.as_millis() {
                    return frametime - refresh_time;
                } else if frametime.as_millis() < target_frametime.as_millis() {
                    return frametime + refresh_time;
                }
            }
        }

        frametime
    }

    fn get_refresh_rate() -> Option<Fps> {
        use std::process::Command;

        let dumpsys_data = Command::new("dumpsys")
            .arg("SurfaceFlinger")
            .output()
            .ok()?;
        let dumpsys_data = String::from_utf8_lossy(&dumpsys_data.stdout).into_owned();

        let parse_line = dumpsys_data
            .lines()
            .find(|line| line.contains("refresh-rate"))?;
        Some(
            parse_line
                .split(':')
                .nth(1)?
                .split('.')
                .next()?
                .trim()
                .parse()
                .unwrap(),
        )
    }
}
