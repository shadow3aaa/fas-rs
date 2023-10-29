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
    collections::hash_map::{Entry, HashMap},
    process,
    time::{Duration, Instant},
};

use binder::Strong;
#[cfg(debug_assertions)]
use log::debug;
use log::error;

use crate::{IRemoteService::IRemoteService, CHANNEL};

pub fn thread(fas_service: &Strong<dyn IRemoteService>, process: &str) -> anyhow::Result<()> {
    let pid = process::id() as i32;
    let mut buffer_stamp: HashMap<_, Instant> = HashMap::new();

    loop {
        buffer_stamp.retain(|_, s| s.elapsed() < Duration::from_secs(10)); // buffer gc

        let (buffer_ptr, stamp) = match CHANNEL.rx.recv() {
            Ok(o) => o,
            Err(e) => {
                error!("End analyze thread, reason: {e:?}");
                return Ok(());
            }
        };

        if buffer_ptr.is_null() {
            continue;
        }

        let frametime = match buffer_stamp.entry(buffer_ptr) {
            Entry::Occupied(mut o) => o.insert(stamp).elapsed(),
            Entry::Vacant(v) => {
                v.insert(stamp);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        debug!("process: [{process}] framtime: [{frametime:?}]");

        if Ok(false) == fas_service.sendData(process, pid, frametime.as_nanos() as i64) {
            #[cfg(debug_assertions)]
            debug!("Exit analyze thread, since server prefer this is not a fas app anymore");
            return Ok(());
        }
    }
}
