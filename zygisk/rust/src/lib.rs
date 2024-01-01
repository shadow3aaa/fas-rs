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
mod channel;
mod data;
mod hook;
mod utils;

use std::{ffi::CStr, fs, ptr, sync::atomic::AtomicBool, thread};

use android_logger::{self, Config};
use dobby_api::Address;
use libc::c_char;
#[cfg(debug_assertions)]
use log::debug;
use log::{error, LevelFilter};
use toml::Value;

const CONFIG: &str = "/data/media/0/Android/fas-rs/games.toml";

static mut OLD_FUNC_PTR: Address = ptr::null_mut();
static mut IS_CHILD: AtomicBool = AtomicBool::new(false);

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
    let process = utils::process_name(process);

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
pub unsafe extern "C" fn _hook_handler_(process: *const c_char) {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libgui-zygisk"),
    );

    let process = CStr::from_ptr(process);
    let Ok(process) = process.to_str() else {
        return;
    };
    let process = utils::process_name(process);

    #[cfg(debug_assertions)]
    debug!("Try to hook process: {process}");

    libc::pthread_atfork(None, None, Some(utils::at_fork));

    if let Err(e) = thread::Builder::new()
        .name("libgui-analyze".into())
        .spawn(move || {
            if let Err(e) = utils::hook() {
                error!("Failed to hook, reason: {e:#?}");
                return;
            }

            analyze::thread(process).unwrap_or_else(|e| error!("{e:?}"));
        })
    {
        error!("Failed to start analyze thread, reason: {e:?}");
    }
}
