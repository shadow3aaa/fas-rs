use std::time::Duration;

use likely_stable::unlikely;
#[cfg(debug_assertions)]
use log::debug;

use super::{super::buffer::Buffer, PidParams};
use crate::framework::prelude::*;

pub fn pid_control(
    buffer: &Buffer,
    config: &mut Config,
    mode: Mode,
    pid_params: PidParams,
) -> Option<isize> {
    if unlikely(buffer.frametimes.len() < 60) {
        return None;
    }
    let target_fps = buffer.target_fps?;
    let normalized_last_frame = if buffer.additional_frametime == Duration::ZERO {
        buffer.frametimes.front().copied()? * target_fps
    } else {
        buffer.additional_frametime * target_fps
    };

    #[cfg(debug_assertions)]
    debug!("normalized_last_frame: {normalized_last_frame:?}");

    let frame = normalized_last_frame;
    let margin = config.mode_config(mode).margin;
    let margin = Duration::from_millis(margin);
    let target = Duration::from_secs(1) + margin;

    Some(
        pid_control_inner(
            pid_params,
            buffer.avg_time * target_fps,
            frame,
            target,
            buffer.frametimes.iter().copied().take(30).sum::<Duration>() * target_fps,
            buffer.frametimes.iter().copied().take(60).sum::<Duration>() * target_fps,
        ) * 60
            / target_fps as isize,
    )
}

fn pid_control_inner(
    pid_params: PidParams,
    avg_time: Duration,
    current_frametime: Duration,
    target_frametime: Duration,
    last_30_frametimes_sum: Duration,
    last_60_frametimes_sum: Duration,
) -> isize {
    let error_p =
        (current_frametime.as_nanos() as f64 - target_frametime.as_nanos() as f64) * pid_params.kp;
    let error_i = (target_frametime.as_nanos() as f64)
        .mul_add(-30.0, last_30_frametimes_sum.as_nanos() as f64)
        * pid_params.ki;
    let mut error_d = (last_30_frametimes_sum.as_nanos() as f64)
        .mul_add(2.0, -(last_60_frametimes_sum.as_nanos() as f64))
        * pid_params.kd;

    if avg_time > target_frametime {
        error_d = error_d.max(0.0);
    } else {
        error_d = error_d.min(0.0);
    }

    #[cfg(debug_assertions)]
    {
        debug!("error_p {error_p}");
        debug!("error_i {error_i}");
        debug!("error_d {error_d}");
    }

    (error_p + error_i + error_d) as isize
}
