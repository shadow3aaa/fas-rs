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
use std::fs;

use anyhow::Result;

use super::{super::Freq, Insider};

impl Insider {
    pub fn init_default(&self, userspace_governor: bool) -> Result<()> {
        self.unlock_min_freq(self.freqs[0])?;
        self.unlock_max_freq(self.freqs.last().copied().unwrap())?;

        if userspace_governor {
            self.lock_governor("performance")?;
        } else {
            self.reset_governor()?;
        }

        Ok(())
    }

    pub fn init_game(&self, fas_boost: bool) -> Result<()> {
        self.fas_boost.set(fas_boost);
        self.set_fas_freq(self.freqs.last().copied().unwrap())
    }

    pub fn set_fas_freq(&self, f: Freq) -> Result<()> {
        if f == self.cache.get() {
            return Ok(());
        }

        if self.fas_boost.get() {
            if self.cpus.contains(&0) {
                return Ok(());
            }

            self.lock_min_freq(f)?;
            let last_freq = self.freqs.last().copied().unwrap();
            self.lock_max_freq(last_freq)?;
        } else {
            let first_freq = self.freqs.first().copied().unwrap();
            self.lock_max_freq(f)?;
            self.lock_min_freq(first_freq)?;
        }

        self.cache.set(f);

        Ok(())
    }

    pub fn set_freq(&self, f: Freq) -> Result<()> {
        if f == self.cache.get() {
            return Ok(());
        }

        let first_freq = self.freqs.first().copied().unwrap();
        self.lock_max_freq(f)?;
        self.lock_min_freq(first_freq)?;
        self.cache.set(f);

        Ok(())
    }

    fn reset_governor(&self) -> Result<()> {
        if let Some(ref governor) = *self.gov_snapshot.borrow() {
            self.unlock_governor(governor)?;
        }

        Ok(())
    }

    pub fn set_fas_governor(&self, use_performance_governor: bool) -> Result<()> {
        if self.fas_boost.get() || !use_performance_governor {
            return self.reset_governor();
        }

        if !self.cpus.contains(&0) {
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
