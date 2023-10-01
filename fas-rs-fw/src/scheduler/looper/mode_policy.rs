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

use toml::Value;

use super::Looper;
use crate::{error::Result, node::Node, Config, Error, PerformanceController};

#[derive(Debug)]
pub struct PolicyConfig {
    pub jank_rec_count: u8,
    // pub normal_keep_count: u8,
    pub tolerant_frame_limit: f64,
    pub tolerant_frame_jank: f64,
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(config: &Config) -> Result<PolicyConfig> {
        let mode = Node::read_mode()?;

        let jank_rec_count = config
            .get_mode_conf(mode, "jank_rec_count")?
            .as_integer()
            .ok_or(Error::ParseConfig)? as u8;
        /* let normal_keep_count = config
        .get_mode_conf(mode, "normal_keep_count")?
        .as_integer()
        .ok_or(Error::ParseConfig)? as u8; */

        let tolerant_frame_limit = config.get_mode_conf(mode, "tolerant_frame_limit")?;
        let tolerant_frame_limit = match tolerant_frame_limit {
            Value::Float(f) => f,
            Value::Integer(i) => i as f64,
            _ => return Err(Error::ParseConfig),
        };

        let tolerant_frame_jank = config.get_mode_conf(mode, "tolerant_frame_jank")?;
        let tolerant_frame_jank = match tolerant_frame_jank {
            Value::Float(f) => f,
            Value::Integer(i) => i as f64,
            _ => return Err(Error::ParseConfig),
        };

        Ok(PolicyConfig {
            jank_rec_count,
            // normal_keep_count,
            tolerant_frame_limit,
            tolerant_frame_jank,
        })
    }
}
