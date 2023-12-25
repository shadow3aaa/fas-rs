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
    collections::hash_map::HashMap,
    fs, thread,
    time::{Duration, Instant},
};

use binder::{get_interface, Strong};
#[cfg(debug_assertions)]
use log::debug;
use log::error;

use crate::{IRemoteService::IRemoteService, CHANNEL};

pub fn thread(process: &str) -> anyhow::Result<()> {
    let pid = unsafe { libc::getpid() };
    let tid = unsafe { libc::gettid() };
    let mut stamps = HashMap::new();
    let mut gc_timer = Instant::now();
    let mut fas_service = get_server_interface();

    let _ = fs::write("/dev/cpuset/background/tasks", tid.to_string());

    loop {
        let data = match CHANNEL.rx.recv() {
            Ok(d) => d,
            Err(e) => {
                error!("End analyze thread, reason: {e:?}");
                return Ok(());
            }
        };

        #[cfg(debug_assertions)]
        debug!("Rendering Data: {data:?}");

        let last_stamp = stamps.entry(data.buffer).or_insert(data.instant);
        let frametime = data.instant - *last_stamp;
        *last_stamp = data.instant;

        if gc_timer.elapsed() > Duration::from_millis(500) {
            stamps.retain(|p, _| {
                if p.is_null() {
                    let _ = fas_service.removeBuffer(*p as i64, pid);
                    false
                } else {
                    true
                }
            });

            gc_timer = Instant::now();
        }

        #[cfg(debug_assertions)]
        debug!("process: [{process}] framtime: [{frametime:?}]");

        if let Ok(send) = fas_service.sendData(
            data.buffer as i64,
            process,
            pid,
            frametime.as_nanos() as i64,
            data.cpu,
        ) {
            if !send {
                #[cfg(debug_assertions)]
                debug!("Exit analyze thread, since server prefer this is not a fas app anymore");
                return Ok(());
            }
        } else {
            fas_service = get_server_interface();
        }
    }
}

fn get_server_interface() -> Strong<dyn IRemoteService> {
    loop {
        if let Ok(fas_service) = get_interface::<dyn IRemoteService>("fas_rs_server") {
            return fas_service;
        }

        error!("Failed to get binder interface, fas-rs-server didn't started");
        thread::sleep(Duration::from_secs(1));
    }
}
