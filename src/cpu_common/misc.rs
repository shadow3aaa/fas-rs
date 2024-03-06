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
use super::{CpuCommon, Freq};

impl CpuCommon {
    pub fn reset_freq(&mut self) {
        let last_freq = self.freqs.last().copied().unwrap();

        self.cache = last_freq;
        self.fas_freq = last_freq;
        self.set_freq_cached(last_freq);
        self.smooth.reset();
        self.jump.reset();
    }

    fn set_freq(&mut self, f: Freq) {
        self.smooth.update(f);

        for policy in &self.policies {
            let _ = policy.set_fas_freq(f);
        }
    }

    pub fn set_freq_cached(&mut self, f: Freq) {
        if f != self.cache {
            self.cache = f;
            self.set_freq(f);
        }
    }

    pub fn set_limit_freq(&mut self, f: Freq) {
        let clamped_freq = f.max(self.smooth.avg().unwrap_or(f));
        self.fas_freq = (f + clamped_freq) / 2;

        self.smooth.update(self.fas_freq);

        for policy in &self.policies {
            let _ = policy.set_fas_freq(self.fas_freq);
        }
    }
}
