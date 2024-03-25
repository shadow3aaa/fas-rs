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
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use binder::{get_interface, Strong};
use libc::pid_t;
use log::error;

use crate::{channel::CHANNEL, IRemoteService::IRemoteService, IS_CHILD};

pub unsafe fn thread() -> Result<()> {
    let mut instant = Instant::now();
    let pid = libc::getpid();
    let Some(mut fas_service) = get_server_interface() else {
        return Ok(());
    };

    loop {
        let now = match CHANNEL.rx.recv() {
            Ok(d) => d,
            Err(e) => {
                error!("End analyze thread, reason: {e:?}");
                return Ok(());
            }
        };

        let frametime = now - instant;
        instant = now;

        if !send_data_to_server(&mut fas_service, frametime, pid) {
            return Ok(());
        }

        if IS_CHILD.load(Ordering::Acquire) {
            return Ok(());
        }
    }
}

fn send_data_to_server(
    fas_service: &mut Strong<dyn IRemoteService>,
    frametime: Duration,
    pid: pid_t,
) -> bool {
    fas_service
        .sendData(pid, frametime.as_nanos() as i64)
        .unwrap_or_else(|_| {
            get_server_interface().map_or(false, |service| {
                *fas_service = service;
                send_data_to_server(fas_service, frametime, pid)
            })
        })
}

fn get_server_interface() -> Option<Strong<dyn IRemoteService>> {
    loop {
        unsafe {
            if IS_CHILD.load(Ordering::Acquire) {
                return None;
            }
        }

        if let Ok(fas_service) = get_interface::<dyn IRemoteService>("fas_rs_server") {
            return Some(fas_service);
        }

        error!("Failed to get binder interface, fas-rs-server didn't started");
        thread::sleep(Duration::from_secs(1));
    }
}
