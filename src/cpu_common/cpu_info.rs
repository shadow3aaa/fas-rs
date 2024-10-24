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

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use anyhow::Result;

use super::{IGNORE_MAP, OFFSET_MAP};
use crate::file_handler::FileHandler;

#[derive(Debug)]
pub struct Info {
    pub policy: i32,
    path: PathBuf,
    pub freqs: Vec<isize>,
}

impl Info {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let policy = path.file_name().unwrap().to_str().unwrap()[6..].parse()?;

        let mut freqs: Vec<_> = fs::read_to_string(path.join("scaling_available_frequencies"))?
            .split_whitespace()
            .map(|f| f.parse().unwrap())
            .collect();

        freqs.sort_unstable();

        Ok(Self {
            policy,
            path: path.to_path_buf(),
            freqs,
        })
    }

    pub fn write_freq(&self, freq: isize, file_handler: &mut FileHandler) -> Result<()> {
        let freq = freq
            .saturating_add(
                OFFSET_MAP
                    .get()
                    .unwrap()
                    .get(&self.policy)
                    .unwrap()
                    .load(Ordering::Acquire),
            )
            .max(self.freqs.first().copied().unwrap());

        let max_freq_path = self.max_freq_path();
        let min_freq_path = self.min_freq_path();

        let freq = freq.to_string();

        if self.policy != 0
            && !IGNORE_MAP
                .get()
                .unwrap()
                .get(&self.policy)
                .unwrap()
                .load(Ordering::Acquire)
        {
            file_handler.write_with_workround(max_freq_path, &freq)?;
            file_handler.write_with_workround(min_freq_path, &freq)?;
        }
        Ok(())
    }

    pub fn reset_freq(&self, file_handler: &mut FileHandler) -> Result<()> {
        let max_freq_path = self.max_freq_path();
        let min_freq_path = self.min_freq_path();

        file_handler.write_with_workround(max_freq_path, self.freqs.last().unwrap().to_string())?;
        file_handler
            .write_with_workround(min_freq_path, self.freqs.first().unwrap().to_string())?;

        Ok(())
    }

    fn max_freq_path(&self) -> PathBuf {
        self.path.join("scaling_max_freq")
    }

    fn min_freq_path(&self) -> PathBuf {
        self.path.join("scaling_min_freq")
    }
}
