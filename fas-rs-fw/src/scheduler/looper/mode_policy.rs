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
    pub scale: Duration,        // 触发控制器操作水位线
    pub target_fps_offset: f64, // 目标fps偏移量
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(mode: Mode, buffer: &Buffer, _config: &Config) -> PolicyConfig {
        let dispersion = buffer.dispersion_all.unwrap_or_default();
        let rhs = 1.0 / dispersion.clamp(0.3, 1.5);

        let scale = match mode {
            Mode::Powersave => Duration::from_millis(110),
            Mode::Balance => Duration::from_millis(100),
            Mode::Performance => Duration::from_millis(90),
            Mode::Fast => Duration::from_millis(80),
        }
        .mul_f64(rhs);

        let target_fps_offset = buffer.dispersion_normal.unwrap_or_default().min(0.7);

        PolicyConfig {
            scale,
            target_fps_offset,
        }
    }
}
