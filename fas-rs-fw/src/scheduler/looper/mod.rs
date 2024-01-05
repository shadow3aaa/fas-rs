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
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    started: bool,
    last_jank: Instant,
    last_limit: Instant,
    last_release: Instant,
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
            last_jank: Instant::now(),
            last_limit: Instant::now(),
            last_release: Instant::now(),
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let mode = self.node.get_mode()?;
            let target_fps = self
                .buffers
                .values()
                .filter(|b| b.last_update.elapsed() < Duration::from_secs(1))
                .filter_map(|b| b.target_fps)
                .max(); // 只处理目标fps最大的buffer

            let Some(message) = self.recv_message(mode, target_fps)? else {
                continue;
            };

            let data = match message {
                BinderMessage::Data(d) => d,
                BinderMessage::RemoveBuffer(k) => {
                    self.buffers.remove(&k);
                    continue;
                }
            };

            self.consume_data(mode, &data)?;
            if self.started {
                self.do_policy(mode, &data, target_fps)?;
            }
        }
    }

    fn recv_message(
        &mut self,
        mode: Mode,
        target_fps: Option<u32>,
    ) -> Result<Option<BinderMessage>> {
        let timeout = target_fps.map_or(Duration::from_secs(1), |t| Duration::from_secs(10) / t);

        match self.rx.recv_timeout(timeout) {
            Ok(m) => Ok(Some(m)),
            Err(e) => {
                if e == RecvTimeoutError::Disconnected {
                    return Err(Error::Other("Binder Server Disconnected"));
                }

                self.retain_topapp(mode)?;

                if self.started {
                    self.controller.release_max(mode, &self.config)?; // 超时10帧时拉满频率以加快游戏加载
                }

                Ok(None)
            }
        }
    }

    fn consume_data(&mut self, mode: Mode, data: &FasData) -> Result<()> {
        self.buffer_update(data);
        self.retain_topapp(mode)
    }

    fn do_policy(&mut self, mode: Mode, data: &FasData, target_fps: Option<u32>) -> Result<()> {
        let producer = (data.buffer, data.pid);

        let Some(target_fps) = target_fps else {
            return Ok(());
        };

        let event = {
            let Some(buffer) = self.buffers.get_mut(&producer) else {
                return Ok(());
            };

            if buffer.target_fps != Some(target_fps) {
                return Ok(());
            }

            Self::get_event(mode, buffer)
        };

        if let Some(max_event) = self
            .buffers
            .values_mut()
            .map(|buffer| Self::get_event(mode, buffer))
            .max()
        {
            if event < max_event {
                return Ok(());
            }
        }

        match event {
            Event::BigJank => {
                self.controller.release_max(mode, &self.config)?;
                self.last_limit = Instant::now();
            }
            Event::Jank => {
                if self.last_jank.elapsed() * target_fps > Duration::from_secs(30) {
                    self.last_jank = Instant::now();
                    self.controller.release(mode, &self.config)?;
                }
            }
            Event::Release => {
                if self.last_release.elapsed() * target_fps > Duration::from_secs(1) {
                    self.last_release = Instant::now();
                    self.controller.release(mode, &self.config)?;
                }
            }
            Event::Restrictable => {
                if self.last_limit.elapsed() * target_fps > Duration::from_secs(1) {
                    self.last_limit = Instant::now();
                    self.controller.limit(mode, &self.config)?;
                }
            }
            Event::None => (),
        }

        Ok(())
    }
}
