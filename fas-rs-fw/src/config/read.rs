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
    fs,
    path::Path,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use inotify::{Inotify, WatchMask};
use log::{debug, error};

use super::ConfData;

pub(super) fn wait_and_read(path: &Path, toml: &Arc<ConfData>, exit: &Arc<AtomicBool>) {
    let mut retry_count = 0;
    loop {
        if exit.load(Ordering::Acquire) {
            return;
        }

        if retry_count > 10 {
            error!("Too many read config retries");
            process::exit(1);
        }

        let ori = match fs::read_to_string(path) {
            Ok(s) => {
                retry_count = 0;
                s
            }
            Err(e) => {
                debug!("Failed to read file '{}': {e}", path.display());
                retry_count += 1;
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
        *toml.write() = toml::from_str(&ori).unwrap();

        // wait until file change
        let Ok(mut inotify) = Inotify::init() else {
            continue;
        };

        if inotify.watches().add(path, WatchMask::CLOSE_WRITE).is_ok() {
            let _ = inotify.read_events_blocking(&mut []);
        }
    }
}
