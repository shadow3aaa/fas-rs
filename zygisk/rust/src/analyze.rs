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
use std::time::Instant;

use binder::Strong;
use log::debug;

use crate::{IRemoteService::IRemoteService, CHANNEL};

pub fn thread(process: &str) -> anyhow::Result<()> {
    let fas_service: Strong<dyn IRemoteService> = binder::wait_for_interface("fas_rs_service")?;

    let mut stamp = Instant::now();

    loop {
        if let Err(e) = CHANNEL.rx.recv() {
            debug!("End analyze thread, reason: {e:?}");
            return Ok(());
        }

        let now = Instant::now();
        let frametime = now - stamp;
        stamp = now;

        debug!("process: [{process}] framtime: [{frametime:?}]");

        #[allow(clippy::cast_possible_truncation)]
        if !fas_service
            .sendFrameData(process, frametime.as_nanos() as i64)
            .unwrap_or(true)
        {
            debug!("Exit analyze thread, since server prefer this is not a fas app");
            return Ok(());
        }
    }
}
