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

        self.set_freq(last_freq);
        self.smooth.reset();
        self.jump.reset();
    }

    pub fn set_freq(&mut self, f: Freq) {
        self.smooth.update(f);

        for policy in &self.policies {
            let _ = policy.set_fas_freq(f);
        }
    }

    pub fn set_freq_cached(&mut self, f: Freq) {
        if f == self.fas_freq {
            self.smooth.update(f);
        } else {
            self.fas_freq = f;
            self.set_freq(f);
        }
    }

    pub fn set_limit_freq(&mut self, f: Freq) {
        self.smooth.update(f);
        let avg = self
            .smooth
            .avg()
            .unwrap_or_else(|| self.freqs.last().copied().unwrap());
        self.fas_freq = avg;

        for policy in &self.policies {
            let _ = policy.set_fas_freq(avg);
        }
    }
}
