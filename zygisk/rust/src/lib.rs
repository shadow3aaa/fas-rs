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
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_precision_loss)]

mod IRemoteService;
mod analyze;
mod error;
mod hook;

use std::{
    ffi::CStr,
    mem, process, ptr,
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
};

use android_logger::{self, Config};
use dobby_api::Address;
use libc::{c_char, c_int, c_void};
use log::{debug, error, LevelFilter};
use once_cell::sync::Lazy;

use error::Result;
use hook::SymbolHooker;

static mut OLD_FUNC_PTR: Address = ptr::null_mut();
static CHANNEL: Lazy<Channel> = Lazy::new(|| {
    let (sx, rx) = mpsc::sync_channel(1024);
    Channel { sx, rx }
});

struct Channel {
    sx: SyncSender<()>,
    rx: Receiver<()>,
}

unsafe impl Sync for Channel {}

#[no_mangle]
pub unsafe extern "C" fn hook_handler(process: *const c_char) {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libgui analyze"),
    );

    let process = CStr::from_ptr(process);
    let Ok(process) = process.to_str() else {
        return;
    };
    let process = process.to_string();

    debug!("process: [{}], pid: [{}]", process, process::id());

    if process.contains("zygote") {
        return;
    }

    if let Err(e) = hook() {
        error!("Failed to hook, reason: {e:#?}");
        return;
    }

    debug!("Hooked");

    if let Err(e) = thread::Builder::new()
        .name("libgui-analyze".into())
        .spawn(move || analyze::thread(&process))
    {
        error!("Failed to start analyze thread, reason: {e:?}");
    }
}

unsafe fn hook() -> Result<()> {
    OLD_FUNC_PTR = SymbolHooker::new("/system/lib64/libgui.so")?
        .find_and_hook("android::Surface::queueBuffer(", post_hook as Address)?;
    Ok(())
}

// int Surface::queueBuffer(android_native_buffer_t* buffer, int fenceFd)
unsafe extern "C" fn post_hook(android_native_buffer_t: c_void, fence_id: c_int) -> c_int {
    let ori_fun: extern "C" fn(c_void, c_int) -> c_int = mem::transmute(OLD_FUNC_PTR);
    let result = ori_fun(android_native_buffer_t, fence_id);

    let _ = CHANNEL.sx.try_send(());

    result
}
