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
    fs::{self, set_permissions},
    os::unix::fs::PermissionsExt,
    path::Path,
    thread,
    time::Duration,
};

macro_rules! disable {
    ($($path: literal),*) => {
        {
            $(
                write_and_lock(Path::new($path), "0");
            )*
        }
    }
}

pub fn cleaner() {
    loop {
        disable!(
            "/sys/module/mtk_fpsgo/parameters/perfmgr_enable",
            "/sys/module/perfmgr/parameters/perfmgr_enable",
            "/sys/module/perfmgr_policy/parameters/perfmgr_enable",
            "/sys/module/perfmgr_mtk/perfmgr_enable"
        );

        thread::sleep(Duration::from_secs(10));
    }
}

fn write_and_lock(path: &Path, value: &str) {
    let _ = set_permissions(path, PermissionsExt::from_mode(0o644));
    let _ = fs::write(path, value);
    let _ = set_permissions(path, PermissionsExt::from_mode(0o444));
}
