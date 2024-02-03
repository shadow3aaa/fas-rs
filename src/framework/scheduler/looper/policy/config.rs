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
    pub scale: Duration,
    pub jank_scale: Duration,
    pub big_jank_scale: Duration,
}

impl PolicyConfig {
    pub fn new(config: &Config, mode: Mode, _buffer: &Buffer) -> Self {
        let scale_ms = config.mode_config(mode).scale_ms;
        let scale = Duration::from_millis(scale_ms);

        let jank_scale_ms = config.mode_config(mode).jank_scale_ms;
        let jank_scale = Duration::from_millis(jank_scale_ms);

        let big_jank_scale_ms = config.mode_config(mode).big_jank_scale_ms;
        let big_jank_scale = Duration::from_millis(big_jank_scale_ms);

        Self {
            scale,
            jank_scale,
            big_jank_scale,
        }
    }
}
