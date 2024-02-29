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

#[cfg(any(not(target_os = "android"), not(target_arch = "aarch64")))]
compile_error!("Only for aarch64 android");

mod IRemoteService;
mod analyze;
mod channel;
mod data;
mod hook;
mod utils;

use std::{ffi::CStr, fs, path::Path, ptr, sync::atomic::AtomicBool, thread};

use android_logger::Config;
use dobby_api::Address;

use binder::get_interface;
use libc::c_char;
use log::{error, LevelFilter};

static mut OLD_FUNC_PTR: Address = ptr::null_mut();
static mut IS_CHILD: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub unsafe extern "C" fn need_hook(process: *const c_char) -> bool {
    use IRemoteService::IRemoteService;

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libgui-zygisk"),
    );

    let Ok(server_pid) = fs::read_to_string("/dev/fas_rs/pid") else {
        return false;
    };

    let comm = Path::new("/proc").join(server_pid).join("comm");
    if let Ok(comm) = fs::read_to_string(comm) {
        if comm.trim() != "fas-rs" {
            return false;
        }
    }

    let process = CStr::from_ptr(process);
    let Ok(process) = process.to_str() else {
        return false;
    };
    let process = utils::process_name(process);

    get_interface::<dyn IRemoteService>("fas_rs_server")
        .map_or(false, |service| service.needFas(&process).unwrap_or(false))
}

#[no_mangle]
pub unsafe extern "C" fn hook_handler() {
    libc::pthread_atfork(None, None, Some(utils::at_fork));

    let _ = thread::Builder::new()
        .name("libgui-analyze".into())
        .spawn(move || {
            if let Err(e) = utils::hook() {
                error!("Failed to hook, reason: {e:#?}");
                return;
            }

            analyze::thread().unwrap_or_else(|e| error!("{e:?}"));
        });
}
