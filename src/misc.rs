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
    backtrace::Backtrace,
    panic::PanicInfo,
    process::{self, Command},
};

use log::error;

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
