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
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

use super::Freq;

const CPUFREQ_DEBUG: &str = "/proc/cpudvfs/cpufreq_debug";

#[derive(Debug, PartialEq, Eq)]
pub struct Bounder {
    freq_debug: PathBuf,
}

impl Bounder {
    pub fn new() -> Option<Self> {
        let path = Path::new(CPUFREQ_DEBUG);

        if path.exists() {
            Some(Self {
                freq_debug: path.to_path_buf(),
            })
        } else {
            None
        }
    }

    pub fn force_freq(&self, num: u8, l: Freq, r: Freq) -> Result<()> {
        let message = format!("{num} {l} {r}");
        fs::write(&self.freq_debug, message)?;
        Ok(())
    }
}
