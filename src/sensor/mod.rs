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
//! 放置[`fas_rs_fw::VirtualFrameSensor`]的实现
//! 还有一些用于这些实现的实用函数

pub mod dumpsys;
pub mod mtk_fpsgo;

use std::{
    cell::Cell,
    time::{Duration, Instant},
};

use fas_rs_fw::prelude::*;

use log::debug;

// 如果传感器实现实际读取的是帧vsync间隔而不是真正的帧渲染时间时需要用这个修正
// 为了方便实现trait已经做了内部可变处理
pub struct FrameTimeFixer {
    refresh_rate: Cell<Option<Fps>>,
    timer: Cell<Instant>,
}

impl FrameTimeFixer {
    fn new() -> Self {
        let timer = Cell::new(Instant::now());
        let refresh_rate = Cell::new(Self::get_refresh_rate());
        Self {
            refresh_rate,
            timer,
        }
    }

    fn fix(&self, mut frametime: FrameTime, target_fps: TargetFps) -> FrameTime {
        let now = Instant::now();
        if now - self.timer.get() >= Duration::from_secs(5) {
            self.timer.set(now);
            self.refresh_rate.set(Self::get_refresh_rate());

            if let Some(rate) = self.refresh_rate.get() {
                debug!("Got screen refresh rate: {rate}");
            }
        }

        debug!("Frametime before fix: {frametime:?}");

        if let Some(refresh_rate) = self.refresh_rate.get() {
            if refresh_rate != target_fps {
                frametime = Self::do_fix(frametime, target_fps, refresh_rate).unwrap_or(frametime);
            }
        }

        debug!("Frametime after fix: {frametime:?}");

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

    fn do_fix(f: FrameTime, t: TargetFps, r: Fps) -> Option<FrameTime> {
        let target_frametime = Duration::from_secs(1) / t;
        let _refresh_time = Duration::from_secs(1) / r;

        let unfixed_ms = u32::try_from(f.as_millis()).ok()?;
        let target_ms = u32::try_from(target_frametime.as_millis()).ok()?;

        // 缩放ms部分为目标ms，记录缩放比例，应用到ms小数部分
        let f = f - Duration::from_millis(unfixed_ms.into());
        let f = (f * target_ms).checked_div(unfixed_ms)?;
        Some(f + Duration::from_millis(target_ms.into()))
    }
}
