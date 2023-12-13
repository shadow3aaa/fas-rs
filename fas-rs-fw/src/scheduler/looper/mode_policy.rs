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
    pub scale: Duration,    // 触发控制器操作水位线
    pub step_min: Duration, // 累计器每次最少增加量
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(_mode: Mode, buffer: &Buffer, _config: &Config) -> PolicyConfig {
        let dispersion = buffer.dispersion.unwrap_or_default();
        let rhs = 1.0 / dispersion.clamp(0.5, 1.5);

        let step_min = Duration::from_millis(0);
        let scale = Duration::from_millis(10).mul_f64(rhs);

        /* match mode {
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
        } */

        PolicyConfig { scale, step_min }
    }
}
