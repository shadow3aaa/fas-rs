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
use cpu_cycles_reader::Cycles;

use super::Insider;

impl Insider {
    pub fn normal_policy(&mut self, max_cycles: Cycles) {
        let target_freq = max_cycles * 100 / 65; // target usage: 65
                                                 // target freq = cycles / target_usage

        let min = Cycles::from_khz(self.freqs.first().copied().unwrap() as i64);
        let max = Cycles::from_khz(self.freqs.last().copied().unwrap() as i64);

        let target_freq = target_freq.clamp(min, max);
        let _ = self.set_userspace_governor_freq(target_freq.as_khz() as usize);
    }
}
