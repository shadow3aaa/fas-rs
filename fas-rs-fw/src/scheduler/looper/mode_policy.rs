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

use super::Looper;
use crate::{error::Result, node::Mode, Config, Error, PerformanceController};

#[derive(Debug)]
pub struct PolicyConfig {
    pub jank_keep_count: u8,
    pub normal_keep_count: u8,
    pub tolerant_frame_limit: f64,
    pub tolerant_frame_jank: f64,
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(mode: Mode, variance: Duration, config: &Config) -> Result<PolicyConfig> {
        let tolerant_frame_offset = config.get_mode_conf(mode, "tolerant_frame_offset")?;
        let tolerant_frame_offset = match tolerant_frame_offset {
            Value::Float(f) => f,
            Value::Integer(i) => i as f64,
            _ => return Err(Error::ParseConfig),
        };

        let jank_keep_count;
        let normal_keep_count;
        let tolerant_frame_limit;
        let tolerant_frame_jank;

        if variance > Duration::from_millis(10) {
            jank_keep_count = 1;
            normal_keep_count = 0;

            tolerant_frame_limit = 1.3;
            tolerant_frame_jank = 1.8;
        } else if variance > Duration::from_millis(6) {
            jank_keep_count = 3;
            normal_keep_count = 2;

            tolerant_frame_limit = 1.0;
            tolerant_frame_jank = 1.5;
        } else if variance > Duration::from_millis(3) {
            jank_keep_count = 3;
            normal_keep_count = 2;

            tolerant_frame_limit = 0.75;
            tolerant_frame_jank = 1.25;
        } else {
            jank_keep_count = 5;
            normal_keep_count = 3;

            tolerant_frame_limit = 0.5;
            tolerant_frame_jank = 1.0;
        }

        let tolerant_frame_jank = tolerant_frame_jank + tolerant_frame_offset;

        Ok(PolicyConfig {
            jank_keep_count,
            normal_keep_count,
            tolerant_frame_limit,
            tolerant_frame_jank,
        })
    }
}
