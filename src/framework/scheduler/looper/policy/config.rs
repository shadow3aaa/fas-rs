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

use crate::framework::node::Mode;

#[derive(Debug, Clone, Copy)]
pub struct PolicyConfig {
    pub jank_scale: Duration,
    pub big_jank_scale: Duration,
}

impl PolicyConfig {
    pub const fn new(mode: Mode) -> Self {
        let (jank_scale, big_jank_scale) = match mode {
            Mode::Powersave | Mode::Balance => {
                (Duration::from_millis(58), Duration::from_millis(83))
            }
            Mode::Performance | Mode::Fast => {
                (Duration::from_millis(41), Duration::from_millis(58))
            }
        };

        Self {
            jank_scale,
            big_jank_scale,
        }
    }
}
