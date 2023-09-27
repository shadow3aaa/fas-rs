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

use super::Looper;
use crate::{error::Result, node::Node, Config, Error, PerformanceController};

#[derive(Debug)]
pub struct PolicyConfig {
    pub normal_keep_count: u8,
    pub jank_keep_count: u8,
    pub tolerant_jank: Duration,
    pub tolerant_big_jank: Duration,
    pub tolerant_unit: u32,
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config(config: &Config) -> Result<PolicyConfig> {
        let mode = Node::read_mode()?;

        let normal_keep_count = config
            .get_mode_conf(mode, "normal_keep_count")?
            .as_integer()
            .ok_or(Error::ParseConfig)? as u8;
        let jank_keep_count = config
            .get_mode_conf(mode, "jank_keep_count")?
            .as_integer()
            .ok_or(Error::ParseConfig)? as u8;

        let tolerant_jank = config
            .get_mode_conf(mode, "tolerant_jank")?
            .as_integer()
            .ok_or(Error::ParseConfig)?;
        let tolerant_jank = Duration::from_millis(tolerant_jank as u64);

        let tolerant_big_jank = config
            .get_mode_conf(mode, "tolerant_big_jank")?
            .as_integer()
            .ok_or(Error::ParseConfig)?;
        let tolerant_big_jank = Duration::from_millis(tolerant_big_jank as u64);

        let tolerant_unit = config
            .get_mode_conf(mode, "tolerant_unit")?
            .as_integer()
            .ok_or(Error::ParseConfig)? as u32;

        Ok(PolicyConfig {
            normal_keep_count,
            jank_keep_count,
            tolerant_jank,
            tolerant_big_jank,
            tolerant_unit,
        })
    }
}
