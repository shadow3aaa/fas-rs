// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

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
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
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
