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
use std::time::{Duration, Instant};

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

use buffer::{Buffer, BufferState};
use clean::Cleaner;

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
    buffer: Option<Buffer>,
    state: State,
    delay_timer: Instant,
    janked: bool,
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
            buffer: None,
            state: State::NotWorking,
            delay_timer: Instant::now(),
            janked: false,
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            self.switch_mode();

            #[cfg(feature = "use_ebpf")]
            let _ = self.update_analyzer();
            self.retain_topapp();

            let target_fps = self.buffer.as_ref().and_then(|b| b.target_fps);

            #[cfg(feature = "use_binder")]
            let fas_data = self.recv_message(target_fps)?;
            #[cfg(feature = "use_ebpf")]
            let fas_data = self.recv_message(target_fps);

            if self.windows_watcher.visible_freeform_window() {
                self.disable_fas();
                continue;
            }

            if let Some(data) = fas_data {
                self.janked = false;
                #[cfg(debug_assertions)]
                debug!("janked: {}", self.janked);
                if let Some(state) = self.buffer_update(&data) {
                    match state {
                        BufferState::Usable => self.do_policy(target_fps),
                        BufferState::Unusable => self.disable_fas(),
                    }
                }
            } else if let Some(buffer) = self.buffer.as_mut() {
                self.janked = true;
                #[cfg(debug_assertions)]
                debug!("janked: {}", self.janked);
                buffer.additional_frametime();
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
    fn recv_message(&self, target_fps: Option<u32>) -> Result<Option<FasData>> {
        let target_frametime = target_fps.map(|fps| Duration::from_secs(1) / fps);

        let time = if self.state != State::Working {
            Duration::from_millis(100)
        } else if self.janked {
            target_frametime.map_or(Duration::from_millis(100), |time| time / 4)
        } else {
            target_frametime.map_or(Duration::from_millis(100), |time| time * 2)
        };

        match self.rx.recv_timeout(target_frametime.unwrap_or(time)) {
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
    fn recv_message(&mut self, target_fps: Option<u32>) -> Option<FasData> {
        let target_frametime = target_fps.map(|fps| Duration::from_secs(1) / fps);

        let time = if self.state != State::Working {
            Duration::from_millis(100)
        } else if self.janked {
            target_frametime.map_or(Duration::from_millis(100), |time| time / 4)
        } else {
            target_frametime.map_or(Duration::from_millis(100), |time| time * 2)
        };

        self.analyzer
            .recv_timeout(time)
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
            .buffer
            .as_ref()
            .and_then(|buffer| buffer.event(&mut self.config, self.mode))
        else {
            self.disable_fas();
            return;
        };

        let target_fps = target_fps.unwrap_or(120);

        let factor = Controller::scale_factor(target_fps, event.frame, event.target, self.janked);
        if let Some(process) = self.buffer.as_ref().map(|b| b.pid) {
            self.controller
                .fas_update_freq(process, factor, self.janked);
        }
    }
}
