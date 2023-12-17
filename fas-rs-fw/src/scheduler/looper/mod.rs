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

use super::{topapp::TimedWatcher, BinderMessage};
use crate::{
    config::Config,
    error::{Error, Result},
    node::Node,
    PerformanceController,
};

use buffer::Buffer;
use policy::Event;

pub type Producer = (i64, i32); // buffer, pid
pub type Buffers = HashMap<Producer, Buffer>; // Process, (jank_scale, total_jank_time_ns)

pub struct Looper<P: PerformanceController> {
    rx: Receiver<BinderMessage>,
    config: Config,
    node: Node,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    started: bool,
}

impl<P: PerformanceController> Looper<P> {
    pub fn new(rx: Receiver<BinderMessage>, config: Config, node: Node, controller: P) -> Self {
        Self {
            rx,
            config,
            node,
            controller,
            topapp_checker: TimedWatcher::new(),
            buffers: Buffers::new(),
            started: false,
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let target_fps = self.buffers.values().filter_map(|b| b.target_fps).max();
            let timeout = Duration::from_secs(10) / target_fps.unwrap_or(10);

            let message = match self.rx.recv_timeout(timeout) {
                Ok(m) => m,
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

            let data = match message {
                BinderMessage::Data(d) => d,
                BinderMessage::RemoveBuffer(k) => {
                    self.buffers.remove(&k);
                    continue;
                }
            };

            self.retain_topapp()?;
            self.buffer_update(&data);

            let event = self
                .buffers
                .values_mut()
                .filter(|b| b.target_fps == target_fps)
                .map(|b| {
                    Self::get_event(b, &self.config, &mut self.node).unwrap_or(Event::ReleaseMax)
                })
                .max()
                .unwrap_or(Event::None);

            match event {
                Event::ReleaseMax => {
                    self.controller.release_max(&self.config)?;
                }
                Event::Release => {
                    self.controller.release(&self.config)?;
                }
                Event::Limit => self.controller.limit(&self.config)?,
                Event::None => (),
            }
        }
    }
}
