use std::{error::Error, thread, time::Duration};

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

        let frametimes = sensor.frametimes(10, target_fps);
        let fps = sensor.fps(Duration::from_millis(400));

        if jank(frametimes, fps, target_fps) {
            controller.release();
        } else {
            controller.limit();
        }

        let sleep_time = Duration::from_secs(10) / target_fps; // 等待10帧
        thread::sleep(sleep_time);

        Ok(())
    }
}

// 判断是否出现卡顿
fn jank(frametime: Vec<FrameTime>, fps: Vec<Fps>, target_fps: TargetFps) -> bool {
    if fps.is_empty() || frametime.is_empty() {
        return true;
    }

    let avg_fps = fps.iter().sum::<u32>() / fps.len() as u32;
    let target_frametime =
        (Duration::from_secs(1) / target_fps).saturating_add(Duration::from_micros(100));

    // debug
    /* println!("fps: {}, target: {}", &avg_fps, &target_fps);
    println!("frametime: {:#?}, target: {:#?}", &frametime.iter().max(), &target_frametime);
    println!("Jank: {}", avg_fps <= target_fps - 3 || frametime
            .iter()
            .any(|ft| *ft > target_frametime)); */

    avg_fps <= target_fps - 3 || frametime.iter().any(|ft| *ft > target_frametime)
}
