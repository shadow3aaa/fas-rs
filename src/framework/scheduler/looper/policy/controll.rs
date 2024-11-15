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

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use likely_stable::unlikely;
#[cfg(debug_assertions)]
use log::debug;

use super::super::buffer::Buffer;
use crate::framework::{prelude::*, scheduler::looper::ControllerState};

pub fn calculate_control(
    buffer: &Buffer,
    config: &mut Config,
    mode: Mode,
    controller_state: &mut ControllerState,
    target_fps_offset_thermal: f64,
) -> Option<isize> {
    if unlikely(buffer.frametime_state.frametimes.len() < 60) {
        return None;
    }

    let target_fps = f64::from(buffer.target_fps_state.target_fps?);
    let target_fps = (target_fps + target_fps_offset_thermal).clamp(0.0, target_fps);
    let normalized_last_frame = if buffer.frametime_state.additional_frametime == Duration::ZERO {
        buffer.frametime_state.frametimes.front().copied()?
    } else {
        buffer.frametime_state.additional_frametime
    }
    .mul_f64(target_fps);

    let adjusted_target_fps = adjust_target_fps(
        &buffer.frametime_state.current_fpses,
        target_fps,
        controller_state,
    );

    let adjusted_last_frame = if buffer.frametime_state.additional_frametime == Duration::ZERO {
        buffer.frametime_state.frametimes.front().copied()?
    } else {
        buffer.frametime_state.additional_frametime
    }
    .mul_f64(adjusted_target_fps);

    #[cfg(debug_assertions)]
    {
        debug!("normalized_last_frame: {normalized_last_frame:?}");
        debug!("adjusted_last_frame: {adjusted_last_frame:?}");
    }

    let margin = config.mode_config(mode).margin;
    let margin = Duration::from_millis(margin);
    let target_frametime = Duration::from_secs(1) + margin;

    Some(calculate_control_inner(
        target_fps,
        controller_state,
        normalized_last_frame,
        adjusted_last_frame,
        target_frametime,
    ))
}

fn adjust_target_fps(
    fpses: &VecDeque<f64>,
    target_fps: f64,
    controller_state: &mut ControllerState,
) -> f64 {
    if controller_state.adjust_timer.elapsed() >= Duration::from_millis(100) {
        controller_state.adjust_timer = Instant::now();
        let fpses: Vec<_> = fpses
            .iter()
            .copied()
            .filter(|fps| *fps >= target_fps - 3.0)
            .collect();

        if !fpses.is_empty() {
            let avg = fpses.iter().sum::<f64>() / fpses.len() as f64;
            let variance =
                fpses.iter().map(|fps| (avg - fps).powi(2)).sum::<f64>() / fpses.len() as f64;

            if variance > 0.02 {
                controller_state.target_fps_offset += 0.1;
            } else {
                controller_state.target_fps_offset -= 0.1;
            }

            controller_state.target_fps_offset =
                controller_state.target_fps_offset.clamp(-3.0, 0.0);
        }
    }

    target_fps + controller_state.target_fps_offset
}

fn calculate_control_inner(
    target_fps: f64,
    controller_state: &ControllerState,
    current_frametime: Duration,
    adjusted_frametime: Duration,
    target_frametime: Duration,
) -> isize {
    let error_p = if current_frametime > target_frametime {
        ((adjusted_frametime.as_nanos() as f64 - target_frametime.as_nanos() as f64)
            * controller_state.params.kp)
            .max(0.0)
    } else {
        (current_frametime.as_nanos() as f64 - target_frametime.as_nanos() as f64)
            * controller_state.params.kp
    };
    let error_p = error_p * 120.0 / target_fps;

    #[cfg(debug_assertions)]
    debug!("error_p {error_p}");

    error_p as isize
}
