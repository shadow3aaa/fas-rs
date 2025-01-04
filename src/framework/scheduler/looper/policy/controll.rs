// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

use std::time::{Duration, Instant};

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

    let target_fps = (f64::from(buffer.target_fps_state.target_fps?) + target_fps_offset_thermal)
        .clamp(0.0, f64::from(buffer.target_fps_state.target_fps?));
    let normalized_last_frame = get_normalized_last_frame(buffer, target_fps)?;
    let adjusted_target_fps = adjust_target_fps(target_fps, controller_state);
    let adjusted_last_frame = get_adjusted_last_frame(buffer, adjusted_target_fps)?;

    #[cfg(debug_assertions)]
    {
        debug!("normalized_last_frame: {normalized_last_frame:?}");
        debug!("adjusted_last_frame: {adjusted_last_frame:?}");
    }

    let margin = Duration::from_millis(config.mode_config(mode).margin);
    let target_frametime = Duration::from_secs(1) + margin;

    Some(calculate_control_inner(
        target_fps,
        controller_state,
        normalized_last_frame,
        adjusted_last_frame,
        target_frametime,
    ))
}

fn get_normalized_last_frame(buffer: &Buffer, target_fps: f64) -> Option<Duration> {
    Some(
        if buffer.frametime_state.additional_frametime == Duration::ZERO {
            buffer.frametime_state.frametimes.front().copied()?
        } else {
            buffer.frametime_state.additional_frametime
        }
        .mul_f64(target_fps),
    )
}

fn get_adjusted_last_frame(buffer: &Buffer, adjusted_target_fps: f64) -> Option<Duration> {
    Some(
        if buffer.frametime_state.additional_frametime == Duration::ZERO {
            buffer.frametime_state.frametimes.front().copied()?
        } else {
            buffer.frametime_state.additional_frametime
        }
        .mul_f64(adjusted_target_fps),
    )
}

fn adjust_target_fps(target_fps: f64, controller_state: &mut ControllerState) -> f64 {
    if controller_state.usage_sample_timer.elapsed() >= Duration::from_secs(1) {
        controller_state.usage_sample_timer = Instant::now();
        let usage = controller_state.controller.usage_max();
        if usage <= 55.0 {
            controller_state.target_fps_offset -= 0.1;
        } else if usage >= 80.0 {
            controller_state.target_fps_offset += 0.1;
        }
    }

    controller_state.target_fps_offset = controller_state.target_fps_offset.clamp(-5.0, 0.0);
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
