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
mod policy;
mod utils;
mod window;

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::{Duration, Instant},
};

use super::{topapp::TimedWatcher, BinderMessage, FasData};
use crate::{
    config::Config,
    error::{Error, Result},
    node::{Mode, Node},
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
    mode: Mode,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    start: bool,
    start_delayed: bool,
    delay_timer: Instant,
    last_limit: Instant,
}

impl<P: PerformanceController> Looper<P> {
    pub fn new(rx: Receiver<BinderMessage>, config: Config, node: Node, controller: P) -> Self {
        Self {
            rx,
            config,
            node,
            mode: Mode::Balance,
            controller,
            topapp_checker: TimedWatcher::new(),
            buffers: Buffers::new(),
            start: false,
            start_delayed: false,
            delay_timer: Instant::now(),
            last_limit: Instant::now(),
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            self.mode = self.node.get_mode()?;
            let target_fps = self
                .buffers
                .values()
                .filter(|b| b.last_update.elapsed() < Duration::from_secs(1))
                .filter_map(|b| b.target_fps)
                .max(); // 只处理目标fps最大的buffer

            let Some(message) = self.recv_message(target_fps)? else {
                continue;
            };

            let data = match message {
                BinderMessage::Data(d) => d,
                BinderMessage::RemoveBuffer(k) => {
                    self.buffers.remove(&k);
                    continue;
                }
            };

            self.consume_data(&data)?;
            if self.start_delayed {
                self.do_policy(target_fps)?;
            }
        }
    }

    fn recv_message(&mut self, target_fps: Option<u32>) -> Result<Option<BinderMessage>> {
        let timeout = target_fps.map_or(Duration::from_secs(1), |t| Duration::from_secs(10) / t);

        match self.rx.recv_timeout(timeout) {
            Ok(m) => Ok(Some(m)),
            Err(e) => {
                if e == RecvTimeoutError::Disconnected {
                    return Err(Error::Other("Binder Server Disconnected"));
                }

                self.retain_topapp()?;

                if self.start_delayed {
                    self.disable_fas()?;
                    self.buffers.values_mut().for_each(Buffer::frame_prepare);
                }

                Ok(None)
            }
        }
    }

    fn consume_data(&mut self, data: &FasData) -> Result<()> {
        self.buffer_update(data);
        self.retain_topapp()
    }

    fn do_policy(&mut self, target_fps: Option<u32>) -> Result<()> {
        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .map(|buffer| buffer.event(self.mode))
            .max()
        else {
            self.disable_fas()?;
            return Ok(());
        };

        let Some(target_fps) = target_fps else {
            return Ok(());
        };

        match event {
            Event::BigJank => {
                self.controller.release_max(self.mode, &self.config)?;
                self.last_limit = Instant::now();
            }
            Event::Jank => {
                self.controller.release(self.mode, &self.config)?;
                self.last_limit = Instant::now();
            }
            Event::Release => self.controller.release(self.mode, &self.config)?,
            Event::Restrictable => {
                if self.last_limit.elapsed() * target_fps > Duration::from_secs(1) {
                    self.last_limit = Instant::now();
                    self.controller.limit(self.mode, &self.config)?;
                }
            }
            Event::None => (),
        }

        Ok(())
    }
}
