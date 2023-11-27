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
mod binder;
mod policy;

use std::{
    cell::Cell,
    collections::HashSet,
    ffi::OsStr,
    fs,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Result;
use fas_rs_fw::prelude::*;

use binder::UperfExtension;
use policy::Policy;

pub type Freq = usize; // 单位: khz

#[derive(Debug)]
pub struct CpuCommon {
    freqs: Vec<Freq>,
    pos: Cell<usize>,
    policies: Vec<Policy>,
    fas_status: Option<Arc<AtomicBool>>,
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

        let mut freqs: Vec<_> = policies
            .iter()
            .flat_map(|p| p.freqs.iter().copied())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        freqs.sort_unstable();

        let mut fas_status = None;
        if let Ok(prop) = Command::new("getprop").arg("uperf_patched_fas_rs").output() {
            let prop = String::from_utf8_lossy(&prop.stdout).into_owned();

            if prop.trim() == "true" {
                let status = Arc::new(AtomicBool::new(false));
                fas_status = Some(status.clone());
                UperfExtension::run_server(status)?;

                for policy in &policies {
                    policy.uperf_ext.set(true);
                }
            }
        }

        Ok(Self {
            pos: Cell::new(freqs.len() - 1),
            freqs,
            policies,
            fas_status,
        })
    }
}

impl PerformanceController for CpuCommon {
    fn limit(&self, _c: &Config) -> fas_rs_fw::Result<()> {
        let mut pos = self.pos.get();

        if pos > 0 {
            pos -= 1;
            self.pos.set(pos);
        }

        let freq = self.freqs[pos];
        for policy in &self.policies {
            let _ = policy.set_fas_freq(freq);
            let _ = policy.set_fas_gov();
        }

        if let Some(ref status) = self.fas_status {
            status.store(true, Ordering::Release);
        }

        Ok(())
    }

    fn release(&self, _c: &Config) -> fas_rs_fw::Result<()> {
        let mut pos = self.pos.get();

        if pos < self.freqs.len() - 1 {
            pos += 1;
            self.pos.set(pos);
        }

        let freq = self.freqs[pos];
        for policy in &self.policies {
            let _ = policy.set_fas_freq(freq);
        }

        Ok(())
    }

    fn release_max(&self, _c: &Config) -> fas_rs_fw::Result<()> {
        let pos = self.freqs.len() - 1;
        self.pos.set(pos);

        let freq = self.freqs[pos];

        for policy in &self.policies {
            let _ = policy.set_fas_freq(freq);
            let _ = policy.reset_gov();
        }

        if let Some(ref status) = self.fas_status {
            status.store(false, Ordering::Release);
        }

        Ok(())
    }

    fn init_game(&self, c: &Config) -> Result<(), fas_rs_fw::Error> {
        self.release_max(c)?;

        for policy in &self.policies {
            let _ = policy.init_game();
        }

        if let Some(ref status) = self.fas_status {
            status.store(true, Ordering::Release);
        }

        Ok(())
    }

    fn init_default(&self, c: &Config) -> Result<(), fas_rs_fw::Error> {
        self.release_max(c)?;

        for policy in &self.policies {
            let _ = policy.init_default();
        }

        if let Some(ref status) = self.fas_status {
            status.store(false, Ordering::Release);
        }

        Ok(())
    }
}
