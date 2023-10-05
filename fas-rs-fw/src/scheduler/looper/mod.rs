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
mod buffer;
mod mode_policy;
mod policy;
mod utils;
mod window;

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use log::debug;

use super::{topapp::TimedWatcher, FasData};
use crate::{
    config::Config,
    error::{Error, Result},
    PerformanceController,
};

use buffer::Buffer;
use window::FrameWindowData;

pub type Buffers = HashMap<Process, Buffer>; // Process, (jank_scale, total_jank_time_ns)
pub type Process = (String, i32); // process, pid

pub struct Looper<P: PerformanceController> {
    rx: Receiver<FasData>,
    config: Config,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    started: bool,
}

impl<P: PerformanceController> Looper<P> {
    pub fn new(rx: Receiver<FasData>, config: Config, controller: P) -> Self {
        Self {
            rx,
            config,
            controller,
            topapp_checker: TimedWatcher::new(),
            buffers: Buffers::new(),
            started: false,
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let data = match self.rx.recv_timeout(Duration::from_secs(1)) {
                Ok(d) => d,
                Err(e) => {
                    if e == RecvTimeoutError::Disconnected {
                        return Err(Error::Other("Binder Server Disconnected"));
                    }

                    self.retain_topapp()?;

                    if self.started {
                        self.controller.release_max(&self.config)?;
                    }

                    continue;
                }
            };

            self.retain_topapp()?;

            let Some(frame) = self.buffer_update(&data) else {
                continue;
            };
            let frame = match frame {
                FrameWindowData::Avg(f) => f,
                FrameWindowData::NotEnough => {
                    self.controller.release_max(&self.config)?;
                    continue;
                }
            };

            if let Some(buffer) = self.buffers.get_mut(&(data.pkg.clone(), data.pid)) {
                Self::do_policy(buffer, frame, &self.controller, &self.config)?;
            }

            debug!("{:#?}", self.buffers);
        }
    }
}
