// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod buffer;
mod clean;
mod policy;
mod utils;

#[cfg(feature = "use_binder")]
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[cfg(feature = "use_ebpf")]
use frame_analyzer::Analyzer;
#[cfg(debug_assertions)]
use log::debug;
use log::info;

use super::{topapp::TimedWatcher, FasData};
#[cfg(feature = "use_binder")]
use crate::framework::error::Error;
use crate::{
    framework::{
        config::Config,
        error::Result,
        node::{Mode, Node},
        Extension,
    },
    Controller,
};

use buffer::Buffer;
use clean::Cleaner;

pub type Producer = i32; // pid
pub type Buffers = HashMap<Producer, Buffer>;

#[derive(PartialEq)]
enum State {
    NotWorking,
    Waiting,
    Working,
}

pub struct Looper {
    #[cfg(feature = "use_binder")]
    rx: Receiver<FasData>,
    #[cfg(feature = "use_ebpf")]
    analyzer: Analyzer,
    config: Config,
    node: Node,
    extension: Extension,
    mode: Mode,
    controller: Controller,
    windows_watcher: TimedWatcher,
    cleaner: Cleaner,
    buffers: Buffers,
    state: State,
    delay_timer: Instant,
}

impl Looper {
    pub fn new(
        #[cfg(feature = "use_binder")] rx: Receiver<FasData>,
        #[cfg(feature = "use_ebpf")] analyzer: Analyzer,
        config: Config,
        node: Node,
        extension: Extension,
        controller: Controller,
    ) -> Self {
        Self {
            #[cfg(feature = "use_binder")]
            rx,
            #[cfg(feature = "use_ebpf")]
            analyzer,
            config,
            node,
            extension,
            mode: Mode::Balance,
            controller,
            windows_watcher: TimedWatcher::new(),
            cleaner: Cleaner::new(),
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

            #[cfg(feature = "use_ebpf")]
            let _ = self.update_analyzer();
            self.retain_topapp();

            let target_fps = self
                .buffers
                .values()
                .filter(|b| b.last_update.elapsed() < Duration::from_secs(1))
                .filter_map(|b| b.target_fps)
                .max(); // 只处理目标fps最大的buffer

            #[cfg(feature = "use_binder")]
            let fas_data = self.recv_message()?;
            #[cfg(feature = "use_ebpf")]
            let fas_data = self.recv_message();

            if self.windows_watcher.visible_freeform_window() {
                self.disable_fas();
                continue;
            }

            if let Some(data) = fas_data {
                self.buffer_update(&data);
                self.do_policy(target_fps);
            }
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

    #[cfg(feature = "use_binder")]
    fn recv_message(&self) -> Result<Option<FasData>> {
        match self.rx.recv_timeout(Duration::from_secs(1)) {
            Ok(m) => Ok(Some(m)),
            Err(e) => {
                if e == RecvTimeoutError::Disconnected {
                    return Err(Error::Other("Binder Server Disconnected"));
                }

                Ok(None)
            }
        }
    }

    #[cfg(feature = "use_ebpf")]
    fn recv_message(&mut self) -> Option<FasData> {
        self.analyzer
            .recv_timeout(Duration::from_secs(1))
            .map(|(pid, frametime)| FasData { pid, frametime })
    }

    #[cfg(feature = "use_ebpf")]
    fn update_analyzer(&mut self) -> Result<()> {
        use crate::framework::utils::get_process_name;

        for pid in self.windows_watcher.topapp_pids().iter().copied() {
            let pkg = get_process_name(pid)?;
            if self.config.need_fas(&pkg) {
                self.analyzer.attach_app(pid)?;
            }
        }

        Ok(())
    }

    fn do_policy(&mut self, target_fps: Option<u32>) {
        if self.state != State::Working {
            #[cfg(debug_assertions)]
            debug!("Not running policy!");
            return;
        }

        let Some(event) = self
            .buffers
            .values_mut()
            .filter(|buffer| buffer.target_fps == target_fps)
            .filter_map(|buffer| buffer.event(&self.config, self.mode))
            .max()
        else {
            self.disable_fas();
            return;
        };

        let target_fps = target_fps.unwrap_or(120);

        let factor = Controller::scale_factor(target_fps, event.frame, event.target);
        self.controller.fas_update_freq(factor);
    }
}
