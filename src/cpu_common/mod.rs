// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod policy;

use std::{ffi::OsStr, fs, time::Duration};

use anyhow::Result;
#[cfg(debug_assertions)]
use log::debug;

use crate::framework::prelude::*;
use api::ApiV0;
use policy::Policy;

pub type Freq = usize; // khz

const BASE_STEP: Freq = 700_000;
const JANK_STEP: Freq = 500_000;
const BIG_JANK_STEP: Freq = 800_000;

#[derive(Debug)]
pub struct CpuCommon {
    policies: Vec<Policy>,
}

impl CpuCommon {
    pub fn new(c: &Config) -> Result<Self> {
        let policies: Vec<_> = fs::read_dir("/sys/devices/system/cpu/cpufreq")?
            .filter_map(|d| Some(d.ok()?.path()))
            .filter(|p| p.is_dir())
            .filter(|p| {
                p.file_name()
                    .and_then(OsStr::to_str)
                    .unwrap()
                    .contains("policy")
            })
            .map(|p| Policy::new(c, p))
            .map(Result::unwrap)
            .collect();

        Ok(Self { policies })
    }

    pub fn limit(&self, target_fps: u32, frame: Duration, target: Duration) {
        let target = target.as_nanos() as Freq;
        let frame = frame.as_nanos() as Freq;

        let step = BASE_STEP * (target - frame) / target;
        let step = step * 120 / target_fps as Freq;

        #[cfg(debug_assertions)]
        debug!("step: -{step}khz");

        self.decrease_fas_freq(step);
    }

    pub fn release(&self, target_fps: u32, frame: Duration, target: Duration) {
        let target = target.as_nanos() as Freq;
        let frame = frame.as_nanos() as Freq;

        let step = BASE_STEP * (frame - target) / target;
        let step = step * 120 / target_fps as Freq;

        #[cfg(debug_assertions)]
        debug!("step: +{step}khz");

        self.increase_fas_freq(step);
    }

    pub fn jank(&self) {
        self.increase_fas_freq(JANK_STEP);
    }

    pub fn big_jank(&self) {
        self.increase_fas_freq(BIG_JANK_STEP);
    }

    pub fn init_game(&self, extension: &Extension) {
        extension.tigger_extentions(ApiV0::InitCpuFreq);

        for policy in &self.policies {
            let _ = policy.init_game();
        }
    }

    pub fn init_default(&self, config: &Config, extension: &Extension) {
        extension.tigger_extentions(ApiV0::ResetCpuFreq);

        for policy in &self.policies {
            let _ = policy.init_default(config);
        }
    }

    fn increase_fas_freq(&self, step: Freq) {
        for policy in &self.policies {
            let _ = policy.increase_fas_freq(step);
        }
    }

    fn decrease_fas_freq(&self, step: Freq) {
        for policy in &self.policies {
            let _ = policy.decrease_fas_freq(step);
        }
    }
}
