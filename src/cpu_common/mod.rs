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
mod policy;

use std::{cell::Cell, collections::HashSet, ffi::OsStr, fs};

use crate::framework::prelude::*;
use anyhow::Result;

use policy::Policy;

pub type Freq = usize; // 单位: khz

#[derive(Debug)]
pub struct CpuCommon {
    freqs: Vec<Freq>,
    fas_freq: Cell<Freq>,
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

        let last_freq = freqs.last().copied().unwrap();
        let fas_freq = Cell::new(last_freq);

        Ok(Self {
            freqs,
            fas_freq,
            policies,
        })
    }

    fn reset_freq(&self, extension: &Extension) {
        let last_freq = self.freqs.last().copied().unwrap();
        self.fas_freq.set(last_freq);

        extension.call_extentions(CallBacks::WriteCpuFreq(last_freq));

        for policy in &self.policies {
            let _ = policy.set_fas_freq(last_freq);
        }
    }

    pub fn limit(&self, extension: &Extension) {
        let current_freq = self.fas_freq.get();
        let limited_freq = current_freq.saturating_sub(50000).max(self.freqs[0]);
        self.fas_freq.set(limited_freq);

        extension.call_extentions(CallBacks::WriteCpuFreq(limited_freq));

        for policy in &self.policies {
            let _ = policy.set_fas_freq(limited_freq);
        }
    }

    pub fn release(&self, extension: &Extension) {
        let current_freq = self.fas_freq.get();
        let released_freq = current_freq
            .saturating_add(50000)
            .min(self.freqs.last().copied().unwrap());
        self.fas_freq.set(released_freq);

        extension.call_extentions(CallBacks::WriteCpuFreq(released_freq));

        for policy in &self.policies {
            let _ = policy.set_fas_freq(released_freq);
        }
    }

    pub fn jank(&self, extension: &Extension) {
        let current_freq = self.fas_freq.get();
        let released_freq = current_freq
            .saturating_add(100_000)
            .min(self.freqs.last().copied().unwrap());
        self.fas_freq.set(released_freq);

        extension.call_extentions(CallBacks::WriteCpuFreq(released_freq));

        for policy in &self.policies {
            let _ = policy.set_fas_freq(released_freq);
        }
    }

    pub fn big_jank(&self, extension: &Extension) {
        let max_freq = self.freqs.last().copied().unwrap();

        extension.call_extentions(CallBacks::WriteCpuFreq(max_freq));

        for policy in &self.policies {
            let _ = policy.set_fas_freq(max_freq);
        }
    }

    pub fn init_game(&self, m: Mode, c: &Config, extension: &Extension) {
        self.reset_freq(extension);

        extension.call_extentions(CallBacks::InitCpuFreq);

        for policy in &self.policies {
            let _ = policy.init_game(m, c);
        }
    }

    pub fn init_default(&self, c: &Config, extension: &Extension) {
        self.reset_freq(extension);

        extension.call_extentions(CallBacks::ResetCpuFreq);

        for policy in &self.policies {
            let _ = policy.init_default(c);
        }
    }
}
