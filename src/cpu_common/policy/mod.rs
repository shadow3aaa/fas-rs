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
mod force_bound;
mod utils;

use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use likely_stable::LikelyOption;

use super::Freq;
use crate::{error::Error, framework::prelude::*};
use force_bound::Bounder;

#[derive(Debug, PartialEq, Eq)]
pub struct Policy {
    pub little: bool,
    pub num: u8,
    pub path: PathBuf,
    pub freqs: Vec<Freq>,
    fas_boost: Cell<bool>,
    gov_snapshot: RefCell<Option<String>>,
    force_bound: Option<Bounder>,
}

impl Ord for Policy {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num.cmp(&other.num)
    }
}

impl PartialOrd for Policy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Policy {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Self> {
        let path = p.as_ref();

        let mut freqs: Vec<Freq> = fs::read_to_string(path.join("scaling_available_frequencies"))?
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        freqs.sort_unstable();
        let num = path
            .file_name()
            .and_then_likely(OsStr::to_str)
            .and_then_likely(|p| p.replace("policy", "").trim().parse().ok())
            .ok_or(Error::Other("Failed to parse cpufreq policy num"))?;

        let force_bound = Bounder::new();

        Ok(Self {
            little: false,
            num,
            path: path.to_path_buf(),
            freqs,
            fas_boost: Cell::new(false),
            gov_snapshot: RefCell::new(None),
            force_bound,
        })
    }

    pub fn init_default(&self) -> Result<()> {
        if let Some(ref bounder) = self.force_bound {
            bounder.force_freq(
                self.num,
                self.freqs.first().copied().unwrap(),
                self.freqs.last().copied().unwrap(),
            )?;
        }

        self.unlock_min_freq(self.freqs[0])?;
        self.unlock_max_freq(self.freqs.last().copied().unwrap())?;
        self.reset_gov()
    }

    pub fn init_game(&self, m: Mode, c: &Config) -> Result<()> {
        self.fas_boost.set(c.mode_config(m).fas_boost);

        self.set_fas_gov(m, c)?;
        self.set_fas_freq(self.freqs.last().copied().unwrap())
    }

    pub fn set_fas_freq(&self, f: Freq) -> Result<()> {
        if self.fas_boost.get() {
            if self.little {
                return Ok(());
            }

            self.lock_min_freq(f)?;
            let last_freq = self.freqs.last().copied().unwrap();
            self.lock_max_freq(last_freq)?;

            if let Some(ref bounder) = self.force_bound {
                bounder.force_freq(self.num, f, last_freq)?;
            }
        } else {
            self.lock_max_freq(f)?;
            let first_freq = self.freqs.first().copied().unwrap();
            self.lock_min_freq(first_freq)?;

            if let Some(ref bounder) = self.force_bound {
                bounder.force_freq(self.num, first_freq, f)?;
            }
        }

        Ok(())
    }

    fn reset_gov(&self) -> Result<()> {
        if let Some(ref governor) = *self.gov_snapshot.borrow() {
            self.unlock_governor(governor)?;
        }

        Ok(())
    }

    pub fn set_fas_gov(&self, mode: Mode, c: &Config) -> Result<()> {
        if self.fas_boost.get() || !c.mode_config(mode).use_performance_governor {
            return self.reset_gov();
        }

        if !self.little {
            let path = self.path.join("scaling_governor");
            let cur_gov = fs::read_to_string(path)?;

            if cur_gov.trim() != "performance" {
                self.gov_snapshot.replace(Some(cur_gov));
            }

            self.lock_governor("performance")?;
        }

        Ok(())
    }
}
