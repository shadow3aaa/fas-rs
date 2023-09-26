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
use crate::{
    error::Result,
    node::{Mode, Node},
    PerformanceController,
};

const POWERSAVE: PolicyConfig = PolicyConfig {
    normal_keep_count: 8,
    jank_keep_count: 30,
    tolerant_jank: Duration::from_millis(900),
    tolerant_big_jank: Duration::from_millis(1850),
    tolerant_unit: Duration::from_millis(275),
};

const BALANCE: PolicyConfig = PolicyConfig {
    normal_keep_count: 8,
    jank_keep_count: 30,
    tolerant_jank: Duration::from_millis(800),
    tolerant_big_jank: Duration::from_millis(1750),
    tolerant_unit: Duration::from_millis(250),
};

const PERFORMANCE: PolicyConfig = PolicyConfig {
    normal_keep_count: 8,
    jank_keep_count: 30,
    tolerant_jank: Duration::from_millis(550),
    tolerant_big_jank: Duration::from_millis(1450),
    tolerant_unit: Duration::from_millis(195),
};

const FAST: PolicyConfig = PolicyConfig {
    normal_keep_count: 8,
    jank_keep_count: 30,
    tolerant_jank: Duration::from_millis(415),
    tolerant_big_jank: Duration::from_millis(1000),
    tolerant_unit: Duration::from_millis(172),
};

#[derive(Debug)]
pub struct PolicyConfig {
    pub normal_keep_count: u8,
    pub jank_keep_count: u8,
    pub tolerant_jank: Duration,
    pub tolerant_big_jank: Duration,
    pub tolerant_unit: Duration,
}

impl<P: PerformanceController> Looper<P> {
    pub fn policy_config() -> Result<PolicyConfig> {
        Ok(match Node::read_mode()? {
            Mode::Powersave => POWERSAVE,
            Mode::Balance => BALANCE,
            Mode::Performance => PERFORMANCE,
            Mode::Fast => FAST,
        })
    }
}
