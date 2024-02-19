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
mod smooth;

use std::{collections::HashSet, ffi::OsStr, fs};

use crate::framework::prelude::*;
use anyhow::Result;

use policy::Policy;
use smooth::Smooth;

pub type Freq = usize; // 单位: khz

#[derive(Debug)]
pub struct CpuCommon {
    freqs: Vec<Freq>,
    fas_freq: Freq,
    smooth: Smooth,
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

        Ok(Self {
            freqs,
            fas_freq,
            smooth: Smooth::new(),
            policies,
        })
    }

    fn reset_freq(&mut self) {
        let last_freq = self.freqs.last().copied().unwrap();
        self.set_freq(last_freq);
    }
    
    fn set_freq(&mut self, f: Freq) {
        self.smooth.update(f);
        for policy in &self.policies {
            let _ = policy.set_fas_freq(f);
        }
    }
    
    fn set_freq_cached(&mut self, f: Freq) {
        if f == self.fas_freq {
            self.smooth.update(f);
        } else {
            self.fas_freq = f;
            self.set_freq(f);
        }
    }
    
    fn set_limit_freq(&mut self, f: Freq) {
        self.smooth.update(f);
        let avg = self.smooth.avg().unwrap_or_else(|| self.freqs.last().copied().unwrap());
        self.fas_freq = avg;
        
        for policy in &self.policies {
            let _ = policy.set_fas_freq(avg);
        }
    }

    pub fn limit(&mut self) {
        let current_freq = self.fas_freq;
        let limited_freq = current_freq
            .saturating_sub(50000)
            .max(self.freqs.first().copied().unwrap());
           
        self.set_limit_freq(limited_freq);
    }

    pub fn release(&mut self) {
        let current_freq = self.fas_freq;
        let released_freq = current_freq
            .saturating_add(50000)
            .min(self.freqs.last().copied().unwrap());
        self.set_freq_cached(released_freq);
    }

    pub fn jank(&mut self) {
        let current_freq = self.fas_freq;
        let released_freq = current_freq
            .saturating_add(50000)
            .min(self.freqs.last().copied().unwrap());

        self.set_freq(released_freq);
    }

    pub fn big_jank(&mut self) {
        let max_freq = self.freqs.last().copied().unwrap();
        self.set_freq(max_freq);
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
