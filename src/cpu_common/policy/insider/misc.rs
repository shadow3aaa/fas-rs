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
use std::{cmp, fs};

use anyhow::Result;

use super::{super::Freq, event_loop::State, Insider};

impl Insider {
    pub fn init_default(&mut self, userspace_governor: bool) -> Result<()> {
        self.unlock_min_freq(self.freqs.first().copied().unwrap())?;
        self.unlock_max_freq(self.freqs.last().copied().unwrap())?;
        self.userspace_governor = userspace_governor;
        self.state = State::Normal;

        if userspace_governor {
            self.lock_governor("performance")?;
        } else {
            self.reset_governor()?;
        }

        self.set_userspace_governor_freq(self.freqs.last().copied().unwrap())
    }

    pub fn init_game(&mut self, fas_boost: bool) -> Result<()> {
        self.fas_boost = fas_boost;
        self.state = State::Fas;
        let last_freq = self.freqs.last().copied().unwrap();
        self.set_fas_freq(last_freq)
    }

    pub fn set_fas_freq(&mut self, f: Freq) -> Result<()> {
        self.fas_freq = f;
        self.write_freq()
    }

    pub fn set_userspace_governor_freq(&mut self, f: Freq) -> Result<()> {
        self.governor_freq = f;
        self.write_freq()
    }

    pub fn use_builtin_governor(&self) -> bool {
        self.userspace_governor
            && (!self.use_performance_governor || self.is_little() || self.state == State::Normal)
    }

    fn is_little(&self) -> bool {
        self.cpus.contains(&0)
    }

    fn write_freq(&mut self) -> Result<()> {
        if self.fas_boost && self.state == State::Fas && !self.is_little() {
            self.write_freq_boost()
        } else {
            self.write_freq_nonboost()
        }
    }

    fn write_freq_nonboost(&mut self) -> Result<()> {
        if self.use_builtin_governor() {
            let freq = if self.is_little() {
                self.governor_freq
            } else {
                cmp::min(self.fas_freq, self.governor_freq)
            };
            let target = self.find_freq(freq);

            if self.cache == target {
                Ok(())
            } else {
                self.cache = target;
                self.lock_max_freq(target)?;
                self.lock_min_freq(self.freqs.first().copied().unwrap())
            }
        } else {
            if self.is_little() {
                return Ok(());
            }

            let target = self.find_freq(self.fas_freq);

            if self.cache == target {
                Ok(())
            } else {
                self.cache = target;
                self.lock_max_freq(target)?;
                self.lock_min_freq(self.freqs.first().copied().unwrap())
            }
        }
    }

    fn write_freq_boost(&mut self) -> Result<()> {
        if self.use_builtin_governor() {
            let freq = cmp::max(self.fas_freq, self.governor_freq);
            let target = self.find_freq(freq);

            if self.cache == target {
                Ok(())
            } else {
                self.cache = target;
                self.lock_max_freq(target)?;
                self.lock_min_freq(target)
            }
        } else {
            let target = self.find_freq(self.fas_freq);

            if self.cache == target {
                Ok(())
            } else {
                self.cache = target;
                self.lock_max_freq(self.freqs.last().copied().unwrap())?;
                self.lock_min_freq(target)
            }
        }
    }

    fn find_freq(&mut self, f: Freq) -> Freq {
        self.freqs
            .iter()
            .find(|target| **target >= f)
            .copied()
            .unwrap_or_else(|| self.freqs.last().copied().unwrap())
    }

    fn reset_governor(&self) -> Result<()> {
        if let Some(governor) = &self.gov_snapshot {
            self.unlock_governor(governor)?;
        }

        Ok(())
    }

    pub fn set_fas_governor(&mut self, use_performance_governor: bool) -> Result<()> {
        self.use_performance_governor = use_performance_governor;

        if self.fas_boost || !use_performance_governor {
            return self.reset_governor();
        }

        if !self.cpus.contains(&0) {
            let path = self.path.join("scaling_governor");
            let cur_gov = fs::read_to_string(path)?;

            if cur_gov.trim() != "performance" {
                self.gov_snapshot = Some(cur_gov);
            }

            self.lock_governor("performance")?;
        }

        Ok(())
    }
}
