//! 放置[`fas_rs_fw::VirtualFrameSensor`]的实现
//! 还有一些用于这些实现的实用函数

pub mod mtk_fpsgo;

use std::cell::Cell;
use std::time::{Duration, Instant};

use fas_rs_fw::prelude::*;

// 一种常见的现象
// 如果传感器实现实际读取的是帧vsync间隔而不是真正的帧渲染时间
// 假如此时屏幕刷新率 ＞ 目标帧率
// 设 目标渲染时间 = 1s / 目标帧率
// 屏幕刷新间隔 = 1s / 屏幕刷新率
// 那么vsync间隔就会变成
// 屏幕刷新间隔，目标渲染时间 + 屏幕刷新间隔，目标渲染时间
// 三者的随机组合
// 此函数消去前两个，但是存在误判可能，酌情使用
pub(crate) struct IgnoreFrameTime {
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

    fn ign(&self, frametime: FrameTime, target_fps: TargetFps) -> Option<FrameTime> {
        let now = Instant::now();

        if now - self.timer.get() >= Duration::from_secs(5) {
            self.timer.set(now);
            self.refresh_rate.set(Self::get_refresh_rate());
        }

        if self.refresh_rate.get().unwrap() == target_fps {
            return Some(frametime);
        }

        let target_frametime = Duration::from_secs(1) / target_fps;
        let refresh_time = Duration::from_secs(1) / self.refresh_rate.get().unwrap();
        let total_ign_time = target_frametime.saturating_add(refresh_time);

        if frametime < target_frametime
            || (frametime >= total_ign_time.saturating_mul(9).checked_div(10)?
                && frametime <= total_ign_time.saturating_mul(10).checked_div(9)?)
        {
            None
        } else {
            Some(frametime)
        }
    }

    fn get_refresh_rate() -> Option<Fps> {
        use std::process::Command;

        let dumpsys_data = Command::new("dumpsys")
            .arg("SurfaceFlinger")
            .output()
            .expect("Err : Failed to execute dumpsys SurfaceView");
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
