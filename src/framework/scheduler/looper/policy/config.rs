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
use crate::framework::{node::Mode, Config};

#[derive(Debug, Clone, Copy)]
pub struct PolicyConfig {
    pub acc_dur: Duration,
    pub scale: f64,
    pub jank_scale: f64,
    pub big_jank_scale: f64,
}

impl PolicyConfig {
    pub fn new(config: &Config, mode: Mode, buffer: &Buffer) -> Self {
        let target_fps = buffer.target_fps.unwrap_or(10);
        let target_fps = f64::from(target_fps);
        let acc_dur = 1.0 / buffer.deviation;
        let acc_dur = acc_dur.clamp(1.0, 10.0);

        let scale = config.mode_config(mode).scale;
        let scale = acc_dur * scale / target_fps;

        let jank_scale = config.mode_config(mode).jank_scale;
        let jank_scale = jank_scale / target_fps;

        let big_jank_scale = config.mode_config(mode).big_jank_scale;
        let big_jank_scale = big_jank_scale / target_fps;

        Self {
            acc_dur: Duration::from_secs_f64(acc_dur),
            scale,
            jank_scale,
            big_jank_scale,
        }
    }
}
