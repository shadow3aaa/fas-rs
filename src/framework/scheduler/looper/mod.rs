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

use super::{topapp::TimedWatcher, BinderMessage, FasData};
use crate::framework::{
    config::Config,
    error::{Error, Result},
    node::{Mode, Node},
    PerformanceController,
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

pub struct Looper<P: PerformanceController> {
    rx: Receiver<BinderMessage>,
    config: Config,
    node: Node,
    mode: Mode,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    state: State,
    jank_state: JankEvent,
    delay_timer: Instant,
    last_limit: Instant,
    limit_delay: Duration,
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
            state: State::NotWorking,
            jank_state: JankEvent::None,
            delay_timer: Instant::now(),
            last_limit: Instant::now(),
            limit_delay: Duration::from_secs(1),
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let new_mode = self.node.get_mode()?;
            if self.mode != new_mode && self.state == State::Working {
                self.controller.init_game(new_mode, &self.config)?;
                self.mode = new_mode;
            }

            let target_fps = self
                .buffers
                .values()
                .filter(|b| b.last_update.elapsed() < Duration::from_secs(1))
                .filter_map(|b| b.target_fps)
                .max(); // 只处理目标fps最大的buffer

            if let Some(message) = self.recv_message(target_fps)? {
                match message {
                    BinderMessage::Data(d) => {
                        self.consume_data(&d)?;
                        self.do_normal_policy(target_fps)?;
                    }
                    BinderMessage::RemoveBuffer(k) => {
                        self.buffers.remove(&k);
                    }
                }
            }

            self.do_jank_policy(target_fps)?;
        }
    }

    fn recv_message(&mut self, target_fps: Option<u32>) -> Result<Option<BinderMessage>> {
        let timeout = target_fps.map_or(Duration::from_secs(1), |t| Duration::from_secs(2) / t);
        let timeout_error =
            target_fps.map_or(Duration::from_secs(5), |t| Duration::from_secs(10) / t);

        match self.rx.recv_timeout(timeout) {
            Ok(m) => Ok(Some(m)),
            Err(e) => {
                if e == RecvTimeoutError::Disconnected {
                    return Err(Error::Other("Binder Server Disconnected"));
                }

                self.retain_topapp()?;

                if self.state == State::Working && self.latest_update_elapsed() > timeout_error {
                    self.disable_fas()?;
                }

                Ok(None)
            }
        }
    }

    fn consume_data(&mut self, data: &FasData) -> Result<()> {
        self.buffer_update(data);
        self.retain_topapp()
    }

    fn do_normal_policy(&mut self, target_fps: Option<u32>) -> Result<()> {
        if self.state != State::Working {
            return Ok(());
        }

        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .map(|buffer| buffer.normal_event(&self.config, self.mode))
            .max()
        else {
            self.disable_fas()?;
            return Ok(());
        };

        let Some(target_fps) = target_fps else {
            return Ok(());
        };

        match event {
            NormalEvent::Release => {
                self.last_limit = Instant::now();
                self.controller.release(self.mode, &self.config)?;
            }
            NormalEvent::Restrictable => {
                if self.jank_state == JankEvent::None
                    && self.last_limit.elapsed() * target_fps > self.limit_delay
                {
                    self.last_limit = Instant::now();
                    self.limit_delay = Duration::from_secs(1);
                    self.controller.limit(self.mode, &self.config)?;
                }
            }
            NormalEvent::None => (),
        }

        Ok(())
    }

    fn do_jank_policy(&mut self, target_fps: Option<u32>) -> Result<()> {
        if self.state != State::Working {
            return Ok(());
        }

        self.buffers.values_mut().for_each(Buffer::frame_prepare);

        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .map(|buffer| buffer.jank_event(&self.config, self.mode))
            .max()
        else {
            self.disable_fas()?;
            return Ok(());
        };

        if self.jank_state != event {
            match event {
                JankEvent::BigJank => {
                    self.last_limit = Instant::now();
                    self.limit_delay = Duration::from_secs(5);
                    self.controller.big_jank(self.mode, &self.config)?;
                }
                JankEvent::Jank => {
                    self.last_limit = Instant::now();
                    self.limit_delay = Duration::from_secs(3);
                    self.controller.jank(self.mode, &self.config)?;
                }
                JankEvent::None => (),
            }

            self.jank_state = event;
        }

        Ok(())
    }
}
