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
    pub scale: Duration,
    pub offset: f64,
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(mode: Mode, buffer: &Buffer, _config: &Config) -> PolicyConfig {
        let offset = {
            let current_fps = buffer.current_fps.unwrap_or_default();
            let target_fps = f64::from(buffer.target_fps.unwrap_or_default());
            (target_fps - current_fps).clamp(0.0, 0.7)
        };

        let scale = match mode {
            Mode::Powersave => Duration::from_millis(105),
            Mode::Balance => Duration::from_millis(100),
            Mode::Performance => Duration::from_millis(95),
            Mode::Fast => Duration::from_millis(90),
        };
        PolicyConfig { scale, offset }
    }
}
