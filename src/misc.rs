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
use std::{ffi::CString, fs, ptr};

use libc::{mount, umount, umount2, MS_BIND, MS_REC};

pub fn lock_value<S: AsRef<str>>(v: S, p: S) {
    let value = v.as_ref();
    let path = p.as_ref();

    unmount(path);
    let _ = fs::write(path, value);
    let mount_path = format!("/cache/mount_mask_{value}");
    let _ = fs::write(&mount_path, value);

    mount_bind(&mount_path, path);
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
