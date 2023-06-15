use std::error::Error;
use std::thread;
use std::time::Duration;

use super::Scheduler;
use crate::{Fps, FrameTime, TargetFps};
use crate::{VirtualFrameSensor, VirtualPerformanceController};

impl Scheduler {
    pub(super) fn process_unload(
        sensor: &dyn VirtualFrameSensor,
        controller: &dyn VirtualPerformanceController,
    ) -> Result<(), Box<dyn Error>> {
        sensor.pause()?;
        controller.plug_out()?;
        Ok(())
    }

    pub(super) fn process_load(
        sensor: &dyn VirtualFrameSensor,
        controller: &dyn VirtualPerformanceController,
        target_fps: TargetFps,
    ) -> Result<(), Box<dyn Error>> {
        if target_fps <= 10 {
            return Err("Target Fps should never be less than 10".into());
        }

        let frametimes = sensor.frametimes(10);
        let fps = sensor.fps(Duration::from_millis(400));

        if jank(frametimes, fps, target_fps) {
            controller.release();
        } else {
            controller.limit();
        }

        let sleep_time = Duration::from_secs(1)
            .saturating_mul(10)
            .checked_div(target_fps)
            .unwrap_or(Duration::from_millis(6));

        thread::sleep(sleep_time);

        Ok(())
    }
}

// 判断是否出现卡顿
fn jank(frametimes: Vec<FrameTime>, fps_vec: Vec<Fps>, target: TargetFps) -> bool {
    let avg_fps = fps_vec.iter().sum::<u32>() / fps_vec.len() as u32;

    if avg_fps < target - 3 {
        return true;
    }

    if let Some(target_frametime) = Duration::from_secs(1).checked_div(target) {
        frametimes
            .iter()
            .any(|ft| *ft > target_frametime.saturating_mul(11) / 10)
    } else {
        true
    }
}
