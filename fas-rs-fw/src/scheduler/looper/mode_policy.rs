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

use toml::Value;

use super::{Buffer, Looper};
use crate::{error::Result, node::Mode, Config, Error, PerformanceController};

#[derive(Debug)]
pub struct PolicyConfig {
    pub keep_count: u8,
    pub tolerant_frame_limit: Duration,
    pub tolerant_frame_jank: Duration,
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(mode: Mode, buffer: &Buffer, config: &Config) -> Result<PolicyConfig> {
        let variance = buffer.variance.unwrap_or_default();
        let target_fps = buffer.target_fps.unwrap_or_default();
        let current_fps = buffer.current_fps.unwrap_or_default() as u32;

        let tolerant_frame_offset = config.get_mode_conf(mode, "tolerant_frame_offset")?;
        let tolerant_frame_offset = match tolerant_frame_offset {
            Value::Float(f) => f,
            Value::Integer(i) => i as f64,
            _ => return Err(Error::ParseConfig),
        };
        let tolerant_frame_offset = tolerant_frame_offset / 1000.0; // to ms

        let keep_count;
        let tolerant_frame_limit = Duration::from_millis(1);
        let tolerant_frame_jank = Duration::from_millis(1);

        if target_fps - current_fps >= 3 {
            keep_count = 1;
        } else if variance > Duration::from_millis(10) {
            keep_count = 4;
        } else if variance > Duration::from_millis(5) {
            keep_count = 2;
        } else {
            keep_count = 1;
        }

        let tolerant_frame_jank_offseted =
            tolerant_frame_jank.as_secs_f64() + tolerant_frame_offset;
        let tolerant_frame_jank_offseted = tolerant_frame_jank_offseted.clamp(
            tolerant_frame_limit.as_secs_f64(),
            tolerant_frame_jank.as_secs_f64() + 0.005,
        );
        let tolerant_frame_jank = Duration::from_secs_f64(tolerant_frame_jank_offseted);

        Ok(PolicyConfig {
            keep_count,
            tolerant_frame_limit,
            tolerant_frame_jank,
        })
    }
}
