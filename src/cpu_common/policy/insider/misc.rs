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

use anyhow::Result;

use super::{super::Freq, event_loop::State, Insider};

impl Insider {
    pub fn init_default(&mut self, userspace_governor: bool) -> Result<()> {
        self.unlock_min_freq(self.freqs.first().copied().unwrap())?;
        self.unlock_max_freq(self.freqs.last().copied().unwrap())?;
        self.userspace_governor = userspace_governor;
        self.state = State::Normal;

        self.set_userspace_governor_freq(self.freqs.last().copied().unwrap())
    }

    pub fn init_game(&mut self) -> Result<()> {
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

    pub fn always_userspace_governor(&self) -> bool {
        self.userspace_governor && self.state == State::Normal
    }

    pub fn is_little(&self) -> bool {
        self.cpus.contains(&0)
    }

    fn write_freq(&mut self) -> Result<()> {
        let freq = match self.state {
            State::Normal => {
                if self.always_userspace_governor() {
                    self.governor_freq
                } else {
                    return Ok(());
                }
            }
            State::Fas => {
                if self.is_little() {
                    if self.always_userspace_governor() {
                        self.governor_freq
                    } else {
                        return Ok(());
                    }
                } else {
                    self.fas_freq
                }
            }
        };

        let target = self.find_freq(freq);

        if self.cache == target {
            Ok(())
        } else {
            self.cache = target;
            self.lock_max_freq(target)?;
            self.lock_min_freq(target)
        }
    }

    fn find_freq(&self, f: Freq) -> Freq {
        self.freqs
            .iter()
            .find(|target| **target >= f)
            .copied()
            .unwrap_or_else(|| self.freqs.last().copied().unwrap())
    }
}
