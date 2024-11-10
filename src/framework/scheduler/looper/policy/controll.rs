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

use super::{super::buffer::Buffer, ControllerParams};
use crate::framework::prelude::*;

pub fn calculate_control(
    buffer: &Buffer,
    config: &mut Config,
    mode: Mode,
    controller_params: ControllerParams,
    target_fps_offset: f64,
) -> Option<isize> {
    if unlikely(buffer.frametime_state.frametimes.len() < 60) {
        return None;
    }

    let target_fps = f64::from(buffer.target_fps_state.target_fps?);
    let target_fps = (target_fps + target_fps_offset).clamp(0.0, target_fps);
    let normalized_last_frame = if buffer.frametime_state.additional_frametime == Duration::ZERO {
        buffer.frametime_state.frametimes.front().copied()?
    } else {
        buffer.frametime_state.additional_frametime
    }
    .mul_f64(target_fps);

    #[cfg(debug_assertions)]
    debug!("normalized_last_frame: {normalized_last_frame:?}");

    let frame = normalized_last_frame;
    let margin = config.mode_config(mode).margin;
    let margin = Duration::from_millis(margin);
    let target = Duration::from_secs(1) + margin;

    Some(calculate_control_inner(controller_params, frame, target))
}

fn calculate_control_inner(
    controller_params: ControllerParams,
    current_frametime: Duration,
    target_frametime: Duration,
) -> isize {
    let error_p = (current_frametime.as_nanos() as f64 - target_frametime.as_nanos() as f64)
        * controller_params.kp;

    #[cfg(debug_assertions)]
    debug!("error_p {error_p}");

    error_p as isize
}
