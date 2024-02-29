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
use std::{mem, sync::atomic::Ordering, time::Instant};

use anyhow::Result;
use dobby_api::Address;
use libc::{c_int, c_void};

use crate::{channel::CHANNEL, data::Data, hook::SymbolHooker, IS_CHILD, OLD_FUNC_PTR};

pub unsafe extern "C" fn at_fork() {
    IS_CHILD.store(true, Ordering::Release);
}

pub fn process_name<S: AsRef<str>>(process: S) -> String {
    let process = process.as_ref();
    process
        .split(':')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub unsafe fn hook() -> Result<()> {
    OLD_FUNC_PTR = SymbolHooker::new("/system/lib64/libgui.so")?
        .find_and_hook("android::Surface::queueBuffer(", post_hook as Address)?;
    Ok(())
}

/* Function signature(c++):
*  int Surface::queueBuffer(android_native_buffer_t* buffer, int fenceFd)
*
*  This function is called every time a new frame is added to the buffer */
pub unsafe extern "C" fn post_hook(android_native_buffer_t: *mut c_void, fence_id: c_int) -> c_int {
    let ori_fun: extern "C" fn(*mut c_void, c_int) -> c_int = mem::transmute(OLD_FUNC_PTR); // trans ptr to ori func
    let result = ori_fun(android_native_buffer_t, fence_id);

    let buffer = android_native_buffer_t;
    let instant = Instant::now();
    let data = Data { buffer, instant };

    let _ = CHANNEL.sx.try_send(data);

    result
}
