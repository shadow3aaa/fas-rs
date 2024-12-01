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

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use anyhow::{Context, Result};
use sysinfo::{Cpu, CpuRefreshKind, RefreshKind, System};

use super::IGNORE_MAP;
use crate::file_handler::FileHandler;

#[derive(Debug)]
pub struct Info {
    pub policy: i32,
    path: PathBuf,
    pub cur_freq: isize,
    pub freqs: Vec<isize>,
    sys: System,
    cpus: Vec<i32>,
}

impl Info {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .context("Invalid file name")?;
        let policy_str = file_name.get(6..).context("Invalid policy format")?;
        let policy = policy_str
            .parse::<i32>()
            .context("Failed to parse policy")?;

        let cpus = fs::read_to_string(path.join("affected_cpus"))
            .context("Failed to read affected_cpus")
            .unwrap()
            .split_whitespace()
            .map(|c| {
                c.parse::<i32>()
                    .context("Failed to parse affected_cpus")
                    .unwrap()
            })
            .collect();

        let freqs_content = fs::read_to_string(path.join("scaling_available_frequencies"))
            .context("Failed to read frequencies")?;
        let mut freqs: Vec<isize> = freqs_content
            .split_whitespace()
            .map(|f| f.parse::<isize>().context("Failed to parse frequency"))
            .collect::<Result<_>>()?;
        freqs.sort_unstable();

        let sys = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::new().with_cpu_usage()),
        );

        Ok(Self {
            policy,
            path,
            cur_freq: *freqs.last().context("No frequencies available")?,
            freqs,
            sys,
            cpus,
        })
    }

    pub fn write_freq(&mut self, freq: isize, file_handler: &mut FileHandler) -> Result<()> {
        let min_freq = *self.freqs.first().context("No frequencies available")?;
        let max_freq = *self.freqs.last().context("No frequencies available")?;

        let adjusted_freq = freq.clamp(min_freq, max_freq);
        self.cur_freq = adjusted_freq;
        let adjusted_freq = adjusted_freq.to_string();

        if !IGNORE_MAP
            .get()
            .context("IGNORE_MAP not initialized")?
            .get(&self.policy)
            .context("Policy ignore flag not found")?
            .load(Ordering::Acquire)
        {
            file_handler.write_with_workround(self.max_freq_path(), &adjusted_freq)?;
            file_handler.write_with_workround(self.min_freq_path(), &adjusted_freq)?;
        }
        Ok(())
    }

    pub fn reset_freq(&self, file_handler: &mut FileHandler) -> Result<()> {
        let min_freq = self
            .freqs
            .first()
            .context("No frequencies available")?
            .to_string();
        let max_freq = self
            .freqs
            .last()
            .context("No frequencies available")?
            .to_string();

        file_handler.write_with_workround(self.max_freq_path(), &max_freq)?;
        file_handler.write_with_workround(self.min_freq_path(), &min_freq)?;
        Ok(())
    }

    fn max_freq_path(&self) -> PathBuf {
        self.path.join("scaling_max_freq")
    }

    fn min_freq_path(&self) -> PathBuf {
        self.path.join("scaling_min_freq")
    }

    pub fn cpu_usage(&self) -> impl Iterator<Item = f32> + '_ {
        self.sys
            .cpus()
            .iter()
            .enumerate()
            .filter(|(id, _)| self.cpus.contains(&(*id as i32)))
            .map(|(_, cpu)| cpu)
            .map(Cpu::cpu_usage)
    }

    pub fn refresh_cpu_usage(&mut self) {
        self.sys.refresh_cpu_usage();
    }
}
