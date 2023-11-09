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
#![allow(clippy::similar_names)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]

mod IRemoteService;
mod analyze;
mod error;
mod hook;

use std::{
    ffi::CStr,
    fs, mem, ptr,
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
    time::Instant,
};

use android_logger::{self, Config};
use dobby_api::Address;
use libc::{c_char, c_int, c_void};
#[cfg(debug_assertions)]
use log::debug;
use log::{error, LevelFilter};
use once_cell::sync::Lazy;
use toml::Value;

use error::Result;
use hook::SymbolHooker;

const CONFIG: &str = "/data/media/0/Android/fas-rs/games.toml";

static mut OLD_FUNC_PTR: Address = ptr::null_mut();
static CHANNEL: Lazy<Channel> = Lazy::new(|| {
    let (sx, rx) = mpsc::sync_channel(1024);
    Channel { sx, rx }
});

struct Channel {
    sx: SyncSender<(i64, Instant)>,
    rx: Receiver<(i64, Instant)>,
}

unsafe impl Sync for Channel {}
unsafe impl Send for Channel {}

#[no_mangle]
pub unsafe extern "C" fn _need_hook_(process: *const c_char) -> bool {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libgui-zygisk"),
    );

    let process = CStr::from_ptr(process);

    #[cfg(debug_assertions)]
    debug!("process: {process:?}");

    let Ok(process) = process.to_str() else {
        return false;
    };
    let process = process_name(process);

    let Ok(config) = fs::read_to_string(CONFIG) else {
        error!("Failed to read config file: {CONFIG}");
        return false;
    };

    let Ok(config) = toml::from_str::<Value>(&config) else {
        error!("Failed to parse config");
        return false;
    };

    let Some(list) = config.get("game_list") else {
        #[cfg(debug_assertions)]
        debug!("Didn't find game_list in config");
        return false;
    };

    #[cfg(debug_assertions)]
    debug!("{list:?}");
    list.get(&process).is_some()
}

#[no_mangle]
pub unsafe extern "C" fn _hook_handler_(process: *const c_char) -> bool {
    use IRemoteService::IRemoteService;

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libgui-zygisk"),
    );

    let process = CStr::from_ptr(process);
    let Ok(process) = process.to_str() else {
        return false;
    };
    let process = process_name(process);

    #[cfg(debug_assertions)]
    debug!("Try to hook process: {process}");

    if let Err(e) = thread::Builder::new()
        .name("libgui-analyze".into())
        .spawn(move || {
            if let Err(e) = hook() {
                error!("Failed to hook, reason: {e:#?}");
                return;
            }

            let Ok(fas_service) = binder::get_interface::<dyn IRemoteService>("fas_rs_server")
            else {
                error!("Failed to get binder interface, fas-rs-server didn't started");
                return;
            }; // get binder server interface

            analyze::thread(&fas_service, &process).unwrap_or_else(|e| error!("{e:?}"));
        })
    {
        error!("Failed to start analyze thread, reason: {e:?}");
    }

    true
}

fn process_name<S: AsRef<str>>(process: S) -> String {
    let process = process.as_ref();
    process
        .split(':')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

unsafe fn hook() -> Result<()> {
    OLD_FUNC_PTR = SymbolHooker::new("/system/lib64/libgui.so")?
        .find_and_hook("android::Surface::queueBuffer(", post_hook as Address)?;
    Ok(())
}

/* Function signature(c++):
*  int Surface::queueBuffer(android_native_buffer_t* buffer, int fenceFd)
*
*  This function is called every time a new frame is added to the buffer */
unsafe extern "C" fn post_hook(android_native_buffer_t: *mut c_void, fence_id: c_int) -> c_int {
    let ori_fun: extern "C" fn(*mut c_void, c_int) -> c_int = mem::transmute(OLD_FUNC_PTR); // trans ptr to ori func

    let result = ori_fun(android_native_buffer_t, fence_id);
    let _ = CHANNEL
        .sx
        .try_send((android_native_buffer_t as i64, Instant::now()));

    result
}
