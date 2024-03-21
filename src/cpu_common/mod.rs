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
mod misc;
mod policy;

use std::{cmp, collections::HashSet, ffi::OsStr, fs, time::Duration};

use anyhow::Result;
#[cfg(debug_assertions)]
use log::debug;

use crate::framework::prelude::*;
use policy::Policy;

pub type Freq = usize; // khz

const BASE_STEP: Freq = 200_000;
const JANK_STEP: Freq = 500_000;

#[derive(Debug)]
pub struct CpuCommon {
    freqs: Vec<Freq>,
    fas_freq: Freq,
    cache: Freq,
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

        let mut freqs: Vec<_> = policies
            .iter()
            .flat_map(|p| p.freqs.iter().copied())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        freqs.sort_unstable();

        let fas_freq = freqs.last().copied().unwrap();
        let cache = fas_freq;

        Ok(Self {
            freqs,
            fas_freq,
            cache,
            policies,
        })
    }

    pub fn limit(&mut self, target_fps: u32, frame: Duration, target: Duration) {
        let target = target.as_nanos() as Freq;
        let frame = frame.as_nanos() as Freq;

        let step = BASE_STEP * (target - frame) / target;
        let step = step * 120 / target_fps as Freq;

        #[cfg(debug_assertions)]
        debug!("step: -{step}khz");

        self.fas_freq = cmp::max(
            self.fas_freq.saturating_sub(step),
            self.freqs.first().copied().unwrap(),
        );

        self.set_freq_cached(self.fas_freq);
    }

    pub fn release(&mut self, target_fps: u32, frame: Duration, target: Duration) {
        let target = target.as_nanos() as Freq;
        let frame = frame.as_nanos() as Freq;

        let step = BASE_STEP * (frame - target) / target;
        let step = step * 120 / target_fps as Freq;

        #[cfg(debug_assertions)]
        debug!("step: +{step}khz");

        self.fas_freq = cmp::min(
            self.fas_freq.saturating_add(step),
            self.freqs.last().copied().unwrap(),
        );

        self.set_freq_cached(self.fas_freq);
    }

    pub fn jank(&mut self) {
        let jank_freq = self
            .fas_freq
            .saturating_add(JANK_STEP)
            .min(self.freqs.last().copied().unwrap());

        self.set_freq_cached(jank_freq);
    }

    pub fn big_jank(&mut self) {
        let max_freq = self.freqs.last().copied().unwrap();
        self.set_freq_cached(max_freq);
    }

    pub fn init_game(&mut self, m: Mode, c: &Config, extension: &Extension) {
        self.reset_freq();

        extension.call_extentions(CallBacks::InitCpuFreq);

        for policy in &self.policies {
            let _ = policy.init_game(m, c);
        }
    }

    pub fn init_default(&mut self, c: &Config, extension: &Extension) {
        self.reset_freq();

        extension.call_extentions(CallBacks::ResetCpuFreq);

        for policy in &self.policies {
            let _ = policy.init_default(c);
        }
    }
}
