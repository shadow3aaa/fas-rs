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

use likely_stable::{likely, unlikely};
#[cfg(debug_assertions)]
use log::debug;

use super::buffer::Buffer;
use crate::framework::prelude::*;

const KP: f64 = 0.0001;
const KI_UP: f64 = 0.000_005;
const KI_DOWN: f64 = 0.0004;
const KD: f64 = 0.000_002_5;

pub fn pid_control(buffer: &Buffer, config: &mut Config, mode: Mode) -> Option<isize> {
    if unlikely(buffer.frametimes.len() < 60) {
        return None;
    }
    let target_fps = buffer.target_fps?;
    let target_fps_prefixed = {
        let fpses: Vec<_> = buffer
            .current_fpses
            .iter()
            .copied()
            .filter(|fps| {
                *fps >= f64::from(target_fps) * 119.7 / 120.0 && *fps <= f64::from(target_fps)
            })
            .collect();
        let count = fpses.len();
        let prefixed = fpses.into_iter().sum::<f64>() / count as f64;
        if likely(prefixed.is_normal()) {
            prefixed
        } else {
            f64::from(target_fps)
        }
    };
    let normalized_last_frame = if buffer.additional_frametime == Duration::ZERO {
        buffer
            .frametimes
            .front()
            .copied()?
            .mul_f64(target_fps_prefixed)
    } else {
        buffer.additional_frametime.mul_f64(target_fps_prefixed)
    };

    #[cfg(debug_assertions)]
    debug!("normalized_last_frame: {normalized_last_frame:?}");

    let frame = normalized_last_frame;
    let margin = config.mode_config(mode).margin;
    let margin = Duration::from_millis(margin);
    let target = Duration::from_secs(1) + margin;

    Some(
        pid_control_inner(
            buffer.avg_time.mul_f64(target_fps_prefixed),
            frame,
            target,
            buffer
                .frametimes
                .iter()
                .copied()
                .take(30)
                .sum::<Duration>()
                .mul_f64(target_fps_prefixed),
            buffer
                .frametimes
                .iter()
                .copied()
                .take(60)
                .sum::<Duration>()
                .mul_f64(target_fps_prefixed),
        ) * 60
            / target_fps as isize,
    )
}

fn pid_control_inner(
    avg_time: Duration,
    current_frametime: Duration,
    target_frametime: Duration,
    last_30_frametimes_sum: Duration,
    last_60_frametimes_sum: Duration,
) -> isize {
    let error_p = (current_frametime.as_nanos() as f64 - target_frametime.as_nanos() as f64) * KP;
    let error_i = (target_frametime.as_nanos() as f64)
        .mul_add(-30.0, last_30_frametimes_sum.as_nanos() as f64)
        * if avg_time > target_frametime {
            KI_UP
        } else {
            KI_DOWN
        };
    let mut error_d = (last_30_frametimes_sum.as_nanos() as f64)
        .mul_add(2.0, -(last_60_frametimes_sum.as_nanos() as f64))
        * KD;

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
