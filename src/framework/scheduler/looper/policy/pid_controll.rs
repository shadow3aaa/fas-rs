// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    if unlikely(buffer.frametime_state.frametimes.len() < 60) {
        return None;
    }

    let target_fps = buffer.target_fps_state.target_fps?;
    let normalized_last_frame = if buffer.frametime_state.additional_frametime == Duration::ZERO {
        buffer.frametime_state.frametimes.front().copied()? * target_fps
    } else {
        buffer.frametime_state.additional_frametime * target_fps
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
            frame,
            target,
            buffer
                .frametime_state
                .frametimes
                .iter()
                .copied()
                .take(30)
                .sum::<Duration>()
                * target_fps,
            buffer
                .frametime_state
                .frametimes
                .iter()
                .copied()
                .take(60)
                .sum::<Duration>()
                * target_fps,
        ) * 60
            / target_fps as isize,
    )
}

fn pid_control_inner(
    pid_params: PidParams,
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
    let error_d = (last_30_frametimes_sum.as_nanos() as f64)
        .mul_add(2.0, -(last_60_frametimes_sum.as_nanos() as f64))
        * pid_params.kd;

    #[cfg(debug_assertions)]
    {
        debug!("error_p {error_p}");
        debug!("error_i {error_i}");
        debug!("error_d {error_d}");
    }

    (error_p + error_i + error_d) as isize
}
