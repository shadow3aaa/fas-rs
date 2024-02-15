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

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::{Duration, Instant},
};

use log::info;

use super::{topapp::TimedWatcher, BinderMessage, FasData};
use crate::{
    framework::{
        config::Config,
        error::{Error, Result},
        node::{Mode, Node},
        Extension,
    },
    CpuCommon,
};

use buffer::Buffer;
use policy::{JankEvent, NormalEvent};

pub type Producer = (i64, i32); // buffer, pid
pub type Buffers = HashMap<Producer, Buffer>; // Process, (jank_scale, total_jank_time_ns)

#[derive(PartialEq)]
enum State {
    NotWorking,
    Waiting,
    Working,
}

pub struct Looper {
    rx: Receiver<BinderMessage>,
    config: Config,
    node: Node,
    extension: Extension,
    mode: Mode,
    controller: CpuCommon,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    state: State,
    delay_timer: Instant,
}

impl Looper {
    pub fn new(
        rx: Receiver<BinderMessage>,
        config: Config,
        node: Node,
        extension: Extension,
        controller: CpuCommon,
    ) -> Self {
        Self {
            rx,
            config,
            node,
            extension,
            mode: Mode::Balance,
            controller,
            topapp_checker: TimedWatcher::new(),
            buffers: Buffers::new(),
            state: State::NotWorking,
            delay_timer: Instant::now(),
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            self.switch_mode();
            let target_fps = self
                .buffers
                .values()
                .filter(|b| b.last_update.elapsed() < Duration::from_secs(1))
                .filter_map(|b| b.target_fps)
                .max(); // 只处理目标fps最大的buffer

            if let Some(message) = self.recv_message(target_fps)? {
                match message {
                    BinderMessage::Data(d) => {
                        self.consume_data(&d);
                        self.do_normal_policy(target_fps);
                    }
                    BinderMessage::RemoveBuffer(k) => {
                        self.buffers.remove(&k);
                    }
                }
            }

            self.do_jank_policy(target_fps);
        }
    }

    fn switch_mode(&mut self) {
        if let Ok(new_mode) = self.node.get_mode() {
            if self.mode != new_mode {
                info!(
                    "Switch mode: {} -> {}",
                    self.mode.to_string(),
                    new_mode.to_string()
                );
                self.mode = new_mode;

                if self.state == State::Working {
                    self.controller
                        .init_game(new_mode, &self.config, &self.extension);
                }
            }
        }
    }

    fn recv_message(&mut self, target_fps: Option<u32>) -> Result<Option<BinderMessage>> {
        let timeout = target_fps.map_or(Duration::from_secs(1), |t| Duration::from_secs(2) / t);

        match self.rx.recv_timeout(timeout) {
            Ok(m) => Ok(Some(m)),
            Err(e) => {
                if e == RecvTimeoutError::Disconnected {
                    return Err(Error::Other("Binder Server Disconnected"));
                }

                self.retain_topapp();

                Ok(None)
            }
        }
    }

    fn consume_data(&mut self, data: &FasData) {
        self.buffer_update(data);
        self.retain_topapp();
    }

    fn do_normal_policy(&mut self, target_fps: Option<u32>) {
        if self.state != State::Working {
            return;
        }

        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .map(|buffer| buffer.normal_event(self.mode))
            .max()
        else {
            return;
        };

        let Some(_target_fps) = target_fps else {
            return;
        };

        match event {
            NormalEvent::Release => self.controller.release(),
            NormalEvent::Restrictable => self.controller.limit(),
            NormalEvent::None => (),
        }
    }

    fn do_jank_policy(&mut self, target_fps: Option<u32>) {
        if self.state != State::Working {
            return;
        }

        self.buffers.values_mut().for_each(Buffer::frame_prepare);

        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .map(|buffer| buffer.jank_event(self.mode))
            .max()
        else {
            return;
        };

        match event {
            JankEvent::BigJank => self.controller.big_jank(),
            JankEvent::Jank => self.controller.jank(),
            JankEvent::None => (),
        }
    }
}
