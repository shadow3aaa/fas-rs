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
mod info;

use std::{sync::atomic::Ordering, thread, time::Duration};

use anyhow::Result;
use binder::{get_interface, Strong};
#[cfg(debug_assertions)]
use log::debug;
use log::error;

use crate::{channel::CHANNEL, data::Data, IRemoteService::IRemoteService, IS_CHILD};
use info::Info;

pub unsafe fn thread() -> Result<()> {
    let mut info = Info::new();
    let Some(mut fas_service) = get_server_interface() else {
        return Ok(());
    };

    loop {
        let data = match CHANNEL.rx.recv() {
            Ok(d) => d,
            Err(e) => {
                error!("End analyze thread, reason: {e:?}");
                return Ok(());
            }
        };

        let last_stamp = info.stamps.entry(data.buffer).or_insert(data.instant);
        let frametime = data.instant - *last_stamp;
        *last_stamp = data.instant;

        if !send_data_to_server(&mut fas_service, frametime, data, &info) {
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
    data: Data,
    info: &Info,
) -> bool {
    fas_service.sendData(
        data.buffer as i64,
        info.pid,
        frametime.as_nanos() as i64,
    ).map_or_else(|_| get_server_interface().map_or(false, |service| {
        *fas_service = service;
        send_data_to_server(fas_service, frametime, data, info)
    }), |send| {
        #[cfg(debug_assertions)]
        {
            if !send {
                debug!("Exit analyze thread, since server prefer this is not a fas app anymore");
            }
        }
       send
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
