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

use crate::framework::{prelude::*, Result as FrameworkResult};
use anyhow::Result;

use policy::Policy;

pub type Freq = usize; // 单位: khz

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FasMode {
    Limit,
    Boost,
}

#[derive(Debug)]
pub struct CpuCommon {
    freqs: Vec<Freq>,
    fas_freq: Cell<Freq>,
    mode: Cell<FasMode>,
    policies: Vec<Policy>,
}

impl CpuCommon {
    pub fn new() -> Result<Self> {
        let mut policies: Vec<_> = fs::read_dir("/sys/devices/system/cpu/cpufreq")?
            .filter_map(|d| Some(d.ok()?.path()))
            .filter(|p| p.is_dir())
            .filter(|p| {
                p.file_name()
                    .and_then(OsStr::to_str)
                    .unwrap()
                    .contains("policy")
            })
            .map(Policy::new)
            .map(Result::unwrap)
            .collect();

        policies.sort_unstable();
        if policies.len() > 2 {
            policies[0].little = true;
        }

        let mut freqs: Vec<_> = policies
            .iter()
            .flat_map(|p| p.freqs.iter().copied())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        freqs.sort_unstable();

        let last_freq = freqs.last().copied().unwrap();
        let fas_freq = Cell::new(last_freq);

        let mode = Cell::new(FasMode::Limit);

        Ok(Self {
            freqs,
            fas_freq,
            mode,
            policies,
        })
    }

    fn reset_freq(&self) {
        let last_freq = self.freqs.last().copied().unwrap();
        self.fas_freq.set(last_freq);

        for policy in &self.policies {
            let _ = policy.set_fas_freq(last_freq, self.mode.get());
        }
    }

    fn switch_mode(&self, m: Mode) -> Result<()> {
        let target_mode = match m {
            Mode::Fast => FasMode::Boost,
            _ => FasMode::Limit,
        };

        if self.mode.get() == target_mode {
            return Ok(());
        }

        self.mode.set(target_mode);
        for policy in &self.policies {
            policy.set_fas_gov(target_mode)?;
        }

        Ok(())
    }
}

impl PerformanceController for CpuCommon {
    fn limit(&self, m: Mode, _c: &Config) -> FrameworkResult<()> {
        let _ = self.switch_mode(m);

        let current_freq = self.fas_freq.get();
        let limited_freq = current_freq.saturating_sub(50000).max(self.freqs[0]);
        self.fas_freq.set(limited_freq);

        for policy in &self.policies {
            let _ = policy.set_fas_freq(limited_freq, self.mode.get());
        }

        Ok(())
    }

    fn release(&self, m: Mode, _c: &Config) -> FrameworkResult<()> {
        let _ = self.switch_mode(m);

        let current_freq = self.fas_freq.get();
        let released_freq = current_freq
            .saturating_add(50000)
            .min(self.freqs.last().copied().unwrap());
        self.fas_freq.set(released_freq);

        for policy in &self.policies {
            let _ = policy.set_fas_freq(released_freq, self.mode.get());
        }

        Ok(())
    }

    fn release_max(&self, m: Mode, _c: &Config) -> FrameworkResult<()> {
        let _ = self.switch_mode(m);

        let max_freq = self.freqs.last().copied().unwrap();
        self.fas_freq.set(max_freq);

        for policy in &self.policies {
            let _ = policy.set_fas_freq(max_freq, self.mode.get());
        }

        Ok(())
    }

    fn init_game(&self, m: Mode, _c: &Config) -> FrameworkResult<()> {
        let _ = self.switch_mode(m);
        self.reset_freq();

        for policy in &self.policies {
            let _ = policy.init_game(self.mode.get());
        }

        Ok(())
    }

    fn init_default(&self, _m: Mode, _c: &Config) -> FrameworkResult<()> {
        self.reset_freq();

        for policy in &self.policies {
            let _ = policy.init_default();
        }

        Ok(())
    }
}
