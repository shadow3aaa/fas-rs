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
mod policy;
mod utils;

use std::{
    collections::hash_map::HashMap,
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use super::{topapp::TimedWatcher, FasData};
use crate::{
    config::Config,
    error::{Error, Result},
    PerformanceController,
};

pub type Buffers = HashMap<Process, (isize, isize)>; // Process, (jank_scale, total_jank_time_ns)
pub type Process = (String, i32); // process, pid

pub struct Looper<P: PerformanceController> {
    rx: Receiver<FasData>,
    config: Config,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    started: bool,
    jank_counter: usize,
}

impl<P: PerformanceController> Looper<P> {
    pub fn new(rx: Receiver<FasData>, config: Config, controller: P) -> Result<Self> {
        Ok(Self {
            rx,
            config,
            controller,
            topapp_checker: TimedWatcher::new()?,
            buffers: Buffers::new(),
            started: false,
            jank_counter: 0,
        })
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let timeout = self
                .buffers
                .values()
                .map(|(s, _)| s)
                .copied()
                .max()
                .unwrap_or_else(|| Duration::from_secs(1).as_nanos() as isize);
            let timeout = timeout.min(Duration::from_secs(1).as_nanos() as isize);
            let timeout = Duration::from_nanos(timeout as u64); // 获取buffer中最大的标准帧时间作为接收超时时间

            let data = match self.rx.recv_timeout(timeout) {
                Ok(d) => d,
                Err(e) => {
                    if e == RecvTimeoutError::Disconnected {
                        return Err(Error::Other("Binder Disconnected"));
                    }

                    if self.started {
                        self.buffers
                            .values_mut()
                            .for_each(|(_, j)| *j += Duration::from_secs(1).as_nanos() as isize);
                    }

                    self.retain_topapp();
                    self.buffer_policy()?;

                    continue;
                }
            };

            self.retain_topapp();
            if !self.topapp_checker.is_topapp(data.pid)? {
                continue;
            }

            self.buffer_update(&data);
            self.buffer_policy()?;

            // debug!("{:#?}", self.buffers);
        }
    }
}
