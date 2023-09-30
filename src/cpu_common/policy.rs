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
    cell::{Cell, RefCell},
    cmp::Ordering,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::Result;
use likely_stable::LikelyOption;

use super::Freq;

#[derive(PartialEq, Eq)]
pub struct Policy {
    pub path: PathBuf,
    pub freqs: Vec<Freq>,
    pub is_little: Cell<bool>,
    gov_snapshot: RefCell<Option<String>>,
}

impl Policy {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Self> {
        let p = p.as_ref();

        let mut freqs: Vec<Freq> = fs::read_to_string(p.join("scaling_available_frequencies"))?
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();

        freqs.sort_unstable();

        Ok(Self {
            path: p.to_path_buf(),
            freqs,
            is_little: false.into(),
            gov_snapshot: RefCell::new(None),
        })
    }

    pub fn init_default(&self) -> Result<()> {
        let path = self.path.join("scaling_governor");
        if let Some(ref gov) = *self.gov_snapshot.borrow() {
            let _ = fs::set_permissions(&path, PermissionsExt::from_mode(0o644));
            fs::write(&path, gov)?;
            let _ = fs::set_permissions(&path, PermissionsExt::from_mode(0o444));
        }

        Ok(())
    }

    pub fn init_game(&self) -> Result<()> {
        if !self.is_little.get() {
            let path = self.path.join("scaling_governor");

            let cur_gov = fs::read_to_string(&path)?;
            self.gov_snapshot.replace(Some(cur_gov));

            let _ = fs::set_permissions(&path, PermissionsExt::from_mode(0o644));
            fs::write(&path, "performance")?;
            let _ = fs::set_permissions(&path, PermissionsExt::from_mode(0o444));
        }

        Ok(())
    }

    pub fn set_freq(&self, f: Freq) -> Result<()> {
        let path = self.path.join("scaling_max_freq");

        let _ = fs::set_permissions(&path, PermissionsExt::from_mode(0o644));
        fs::write(path, f.to_string())?;

        Ok(())
    }

    pub fn parse_policy<S: AsRef<str>>(p: S) -> Option<u8> {
        let p = p.as_ref();
        p.replace("policy", "").trim().parse().ok()
    }
}

impl Ord for Policy {
    fn cmp(&self, other: &Self) -> Ordering {
        let num_a: u8 = self
            .path
            .file_name()
            .and_then_likely(|f| Self::parse_policy(f.to_str()?))
            .unwrap();
        let num_b: u8 = other
            .path
            .file_name()
            .and_then_likely(|f| Self::parse_policy(f.to_str()?))
            .unwrap();
        num_a.cmp(&num_b)
    }
}

impl PartialOrd for Policy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
