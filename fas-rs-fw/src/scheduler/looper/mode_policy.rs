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
use std::time::Duration;

use super::{Buffer, Looper};
use crate::{node::Mode, Config, PerformanceController};

#[derive(Debug)]
pub struct PolicyConfig {
    pub scale_time: Duration, // 控制频率总变化速度，越大越慢
    pub tolerant_frame: Duration,
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(mode: Mode, buffer: &Buffer, _config: &Config) -> PolicyConfig {
        let basic_scale;
        let basic_step;

        match mode {
            Mode::Powersave => {
                basic_scale = 0.05;
                basic_step = 0.025;
            }
            Mode::Balance => {
                basic_scale = 0.03;
                basic_step = 0.015;
            }
            Mode::Performance => {
                basic_scale = 0.02;
                basic_step = 0.01;
            }
            Mode::Fast => {
                basic_scale = 0.01;
                basic_step = 0.005;
            }
        }

        let dispersion = buffer.dispersion.unwrap_or_default();
        let rhs = 2.5 / dispersion.clamp(0.1, 1.0);
        let scale = basic_scale * rhs;

        /* let rhs = (buffer.current_fps.unwrap_or_default()
            - f64::from(buffer.target_fps.unwrap_or_default()))
            / 3.0;
        let rhs = 1.0 - rhs.abs();
        let rhs = rhs.clamp(0.5, 1.0); */

        let step = basic_step;

        let scale_time = Duration::from_secs_f64(scale);
        let tolerant_frame = Duration::from_secs_f64(step);

        PolicyConfig {
            scale_time,
            tolerant_frame,
        }
    }
}
