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

use super::{Freq, Insider};

const TARGET_USAGE_DEFAULT: f64 = 0.65;
const TARGET_USAGE_HYBRID: f64 = 0.5;

impl Insider {
    pub fn usage_policy(&self, usage: f64) -> Result<Freq> {
        let current_freq = self.current_freq()?;
        let current_cycles = current_freq as f64 * usage;
        
        let target_usage = if self.hybrid_mode() {
            TARGET_USAGE_HYBRID
        } else {
            TARGET_USAGE_DEFAULT
        };
        let target_freq = (current_cycles / target_usage) as Freq;

        Ok(target_freq)
    }
}
