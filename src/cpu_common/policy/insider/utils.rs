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
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use anyhow::Result;
use sys_mount::{unmount, UnmountFlags};

use super::{Freq, Insider};

impl Insider {
    pub fn lock_max_freq(&self, f: Freq) -> Result<()> {
        let path = self.path.join("scaling_max_freq");
        lock_write(path, f.to_string())
    }

    pub fn lock_min_freq(&self, f: Freq) -> Result<()> {
        let path = self.path.join("scaling_min_freq");
        lock_write(path, f.to_string())
    }

    pub fn lock_governor<S: AsRef<str>>(&self, g: S) -> Result<()> {
        let path = self.path.join("scaling_governor");
        let governor = g.as_ref();
        lock_write(path, governor)
    }

    pub fn unlock_max_freq(&self, f: Freq) -> Result<()> {
        let path = self.path.join("scaling_max_freq");
        unlock_write(path, f.to_string())
    }

    pub fn unlock_min_freq(&self, f: Freq) -> Result<()> {
        let path = self.path.join("scaling_min_freq");
        unlock_write(path, f.to_string())
    }

    pub fn unlock_governor<S: AsRef<str>>(&self, g: S) -> Result<()> {
        let path = self.path.join("scaling_governor");
        let governor = g.as_ref();
        unlock_write(path, governor)
    }
}

fn lock_write<S: AsRef<str>, P: AsRef<Path>>(p: P, s: S) -> Result<()> {
    let s = s.as_ref();
    let p = p.as_ref();

    let _ = fs::set_permissions(p, PermissionsExt::from_mode(0o644));
    fs::write(p, s)?;
    let _ = fs::set_permissions(p, PermissionsExt::from_mode(0o444));

    Ok(())
}

fn unlock_write<S: AsRef<str>, P: AsRef<Path>>(p: P, s: S) -> Result<()> {
    let s = s.as_ref();
    let p = p.as_ref();

    let _ = unmount(p, UnmountFlags::DETACH);
    let _ = fs::set_permissions(p, PermissionsExt::from_mode(0o644));
    fs::write(p, s)?;

    Ok(())
}
