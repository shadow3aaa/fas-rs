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

use super::super::Buffer;
use crate::node::Mode;

#[derive(Debug, Clone, Copy)]
pub struct PolicyConfig {
    pub scale: Duration,
    pub acc_dur: Duration,
}

impl PolicyConfig {
    pub fn new(mode: Mode, buffer: &Buffer) -> Self {
        let target_fps = buffer.target_fps.unwrap_or(10);
        let target_fps = f64::from(target_fps);
        let acc_dur = 1.0 / buffer.deviation;
        let acc_dur = acc_dur.clamp(1.0, 10.0);
        let acc_dur = Duration::from_secs_f64(acc_dur);

        let allow_frame = match mode {
            Mode::Powersave => 1.0,
            Mode::Balance => 0.8,
            Mode::Performance => 0.5,
            Mode::Fast => 0.3,
        };
        let scale = acc_dur.mul_f64(allow_frame / target_fps);

        Self { scale, acc_dur }
    }
}
