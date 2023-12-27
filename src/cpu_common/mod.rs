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

use anyhow::Result;
use fas_rs_fw::prelude::*;

use policy::Policy;

pub type Freq = usize; // 单位: khz

#[derive(Debug)]
pub struct CpuCommon {
    freqs: Vec<Freq>,
    pos: Cell<usize>,
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

        Ok(Self {
            pos: Cell::new(freqs.len() - 1),
            freqs,
            policies,
        })
    }

    fn reset_freq(&self) {
        let pos = self.freqs.len() - 1;
        self.pos.set(pos);
        let freq = self.freqs[pos];

        for policy in &self.policies {
            let _ = policy.set_fas_freq(freq);
        }
    }
}

impl PerformanceController for CpuCommon {
    fn limit(&self, _m: Mode, _c: &Config) -> fas_rs_fw::Result<()> {
        let mut pos = self.pos.get();

        if pos > 0 {
            pos -= 1;
            self.pos.set(pos);

            let freq = self.freqs[pos];
            for policy in &self.policies {
                let _ = policy.set_fas_freq(freq);
            }
        }

        Ok(())
    }

    fn release(&self, _m: Mode, _c: &Config) -> fas_rs_fw::Result<()> {
        let mut pos = self.pos.get();

        if pos < self.freqs.len() - 1 {
            pos += 1;
            self.pos.set(pos);
            let freq = self.freqs[pos];
            for policy in &self.policies {
                let _ = policy.set_fas_freq(freq);
            }
        }

        Ok(())
    }

    fn release_max(&self, _m: Mode, _c: &Config) -> fas_rs_fw::Result<()> {
        let pos = self.freqs.len() - 1;
        let freq = self.freqs[pos];

        for policy in &self.policies {
            let _ = policy.set_fas_freq(freq);
        }

        Ok(())
    }

    fn init_game(&self, _m: Mode, _c: &Config) -> Result<(), fas_rs_fw::Error> {
        self.reset_freq();

        for policy in &self.policies {
            let _ = policy.init_game();
        }

        Ok(())
    }

    fn init_default(&self, _m: Mode, _c: &Config) -> Result<(), fas_rs_fw::Error> {
        self.reset_freq();

        for policy in &self.policies {
            let _ = policy.init_default();
        }

        Ok(())
    }
}
