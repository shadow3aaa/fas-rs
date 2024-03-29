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

#[cfg(debug_assertions)]
use log::debug;
use log::info;

use super::{topapp::TimedWatcher, FasData};
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

pub type Producer = i32; // pid
pub type Buffers = HashMap<Producer, Buffer>;

#[derive(PartialEq)]
enum State {
    NotWorking,
    Waiting,
    Working,
}

pub struct Looper {
    rx: Receiver<FasData>,
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
        rx: Receiver<FasData>,
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

            #[cfg(debug_assertions)]
            debug!("{:?}", self.buffers.keys());

            let target_fps = self
                .buffers
                .values()
                .filter(|b| b.last_update.elapsed() < Duration::from_secs(1))
                .filter_map(|b| b.target_fps)
                .max(); // 只处理目标fps最大的buffer

            if let Some(data) = self.recv_message()? {
                self.consume_data(&data);
                let producer = data.pid;
                self.do_normal_policy(producer, target_fps);
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
                    self.controller.init_game(&self.extension);
                }
            }
        }
    }

    fn recv_message(&mut self) -> Result<Option<FasData>> {
        match self.rx.recv_timeout(Duration::from_secs(1)) {
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

    fn do_normal_policy(&mut self, _producer: Producer, target_fps: Option<u32>) {
        if self.state != State::Working {
            return;
        }

        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .filter_map(|buffer| buffer.normal_event(&self.config, self.mode))
            .max()
        else {
            self.disable_fas();
            return;
        };

        let target_fps = target_fps.unwrap_or(120);

        match event {
            NormalEvent::Release(frame, target) => {
                self.controller.release(target_fps, frame, target);
            }
            NormalEvent::Restrictable(frame, target) => {
                self.controller.limit(target_fps, frame, target);
            }
        }
    }

    fn do_jank_policy(&mut self, target_fps: Option<u32>) -> Option<JankEvent> {
        if self.state != State::Working {
            return None;
        }

        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .filter_map(|buffer| buffer.jank_event())
            .max()
        else {
            self.disable_fas();
            return None;
        };

        match event {
            JankEvent::BigJank => self.controller.big_jank(),
            JankEvent::Jank => self.controller.jank(),
            JankEvent::None => (),
        }

        Some(event)
    }
}
