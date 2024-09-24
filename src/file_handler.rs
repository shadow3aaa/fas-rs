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
    collections::{hash_map::Entry, HashMap},
    fs::{set_permissions, File},
    io::{self, prelude::*, ErrorKind},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::Result;
use sys_mount::{unmount, UnmountFlags};

#[derive(Debug)]
pub struct FileHandler {
    files: HashMap<PathBuf, File>,
}

impl FileHandler {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn read_to_string(&mut self, path: impl AsRef<Path>) -> Result<String> {
        let mut string = String::new();
        match self.files.entry(path.as_ref().to_path_buf()) {
            Entry::Occupied(mut entry) => {
                let mut string = String::new();
                entry.get_mut().rewind()?;
                entry.get().read_to_string(&mut string)?;
            }
            Entry::Vacant(entry) => {
                let mut file = File::open(path.as_ref())?;
                file.read_to_string(&mut string)?;
                entry.insert(file);
            }
        }

        Ok(string)
    }

    pub fn write_with_workround(
        &mut self,
        path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> Result<()> {
        if let Err(e) = self.write(path.as_ref(), content.as_ref()) {
            match e.kind() {
                ErrorKind::PermissionDenied => {
                    set_permissions(path.as_ref(), PermissionsExt::from_mode(0o644))?;
                    self.write(path, content)?;
                    Ok(())
                }
                ErrorKind::InvalidInput => Ok(()),
                _ => Err(e.into()),
            }
        } else {
            Ok(())
        }
    }

    pub fn write(&mut self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
        match self.files.entry(path.as_ref().to_path_buf()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().write_all(content.as_ref())?;
            }
            Entry::Vacant(entry) => {
                let _ = unmount(path.as_ref(), UnmountFlags::DETACH);
                set_permissions(path.as_ref(), PermissionsExt::from_mode(0o644))?;
                let mut file = File::create(path)?;
                file.write_all(content.as_ref())?;
                entry.insert(file);
            }
        }

        Ok(())
    }
}
