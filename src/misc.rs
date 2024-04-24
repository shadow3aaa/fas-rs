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
    backtrace::Backtrace,
    ffi::CString,
    fs,
    panic::PanicInfo,
    path::Path,
    process::{self, Command},
    ptr,
};

use libc::{mount, umount, umount2, MS_BIND, MS_REC};
use log::error;

use crate::framework::Result;

pub fn setprop<S: AsRef<str>>(k: S, v: S) {
    let key = k.as_ref();
    let value = v.as_ref();
    let _ = Command::new("setprop").args([key, value]).spawn();
}

pub fn daemon_panic_handler(panic_info: &PanicInfo) {
    setprop("fas-rs-server-started", "false");

    error!("fas-rs paniced! An unrecoverable error occurred!");

    if let Some(location) = panic_info.location() {
        error!(
            "panic location: in file {} at line {}",
            location.file(),
            location.line()
        );
    }

    if let Some(r) = panic_info.payload().downcast_ref::<&str>() {
        error!("reason: {r:#?}");
    } else {
        error!("reason: Unknown");
    }

    let backtrace = Backtrace::force_capture();
    error!("BACKTRACE: {backtrace}");

    error!("If you're sure this shouldn't happen, open an issue on https://github.com/shadow3aaa/fas-rs/issues");
    process::exit(-1);
}

pub fn lock_value<P: AsRef<Path>, S: AsRef<str>>(p: P, v: S) -> Result<()> {
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
            src_path.as_ptr().cast::<u8>(),
            dest_path.as_ptr().cast::<u8>(),
            ptr::null(),
            MS_BIND | MS_REC,
            ptr::null(),
        );
    }
}

fn unmount(file_system: &str) {
    let path = CString::new(file_system).unwrap();
    let _result = unsafe { umount(path.as_ptr()) };
}
