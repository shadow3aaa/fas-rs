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

use std::{collections::HashMap, ffi::CString, fs, path::Path, ptr};

use libc::{mount, umount, umount2, MS_BIND, MS_REC};

use crate::framework::error::Result;

fn lock_value<P: AsRef<Path>, S: AsRef<str>>(p: P, v: S) -> Result<()> {
    let value = v.as_ref();
    let path = p.as_ref();

    let path = format!("{}", path.display());
    let mount_path = format!("/cache/mount_mask_{value}");

    unmount(&path);

    fs::write(&path, value)?;
    fs::write(&mount_path, value)?;

    mount_bind(&mount_path, &path);

    Ok(())
}

fn mount_bind(src_path: &str, dest_path: &str) {
    let src_path = CString::new(src_path).unwrap();
    let dest_path = CString::new(dest_path).unwrap();

    unsafe {
        umount2(dest_path.as_ptr(), libc::MNT_DETACH);

        mount(
            src_path.as_ptr().cast(),
            dest_path.as_ptr().cast(),
            ptr::null(),
            MS_BIND | MS_REC,
            ptr::null(),
        );
    }
}

fn unmount(file_system: &str) {
    let path = CString::new(file_system).unwrap();
    let _ = unsafe { umount(path.as_ptr()) };
}

macro_rules! lock_values {
    ($map: expr, ($($path: literal),*), $value: literal) => {
        $(
            if let Ok(last_value) = fs::read_to_string($path) {
                $map.insert($path, last_value);
            }

            let _ = lock_value($path, $value);
        )*
    }
}

pub struct Cleaner {
    map: HashMap<&'static str, String>,
}

impl Cleaner {
    pub fn new() -> Self {
        let map = HashMap::new();

        Self { map }
    }

    pub fn cleanup(&mut self) {
        lock_values!(
            self.map,
            (
                "/sys/module/mtk_fpsgo/parameters/perfmgr_enable",
                "/sys/module/perfmgr/parameters/perfmgr_enable",
                "/sys/module/perfmgr_policy/parameters/perfmgr_enable",
                "/sys/module/perfmgr_mtk/parameters/perfmgr_enable",
                "/sys/module/migt/parameters/glk_fbreak_enable"
            ),
            "0"
        );

        lock_values!(
            self.map,
            (
                "/sys/module/migt/parameters/glk_disable",
                "/proc/game_opt/disable_cpufreq_limit"
            ),
            "1"
        );
    }

    pub fn undo_cleanup(&self) {
        for (path, value) in &self.map {
            unmount(path);
            let _ = fs::write(path, value);
        }
    }
}
