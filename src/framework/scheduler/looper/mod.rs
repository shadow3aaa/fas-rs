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
mod thermal;
mod utils;

use std::time::{Duration, Instant};

use frame_analyzer::Analyzer;
use likely_stable::{likely, unlikely};
#[cfg(debug_assertions)]
use log::debug;
use log::info;
use policy::{controll::calculate_control, ControllerParams};
use thermal::Thermal;

use super::{topapp::TimedWatcher, FasData};
use crate::{
    framework::{
        config::Config,
        error::Result,
        node::{Mode, Node},
        Extension,
    },
    Controller,
};

use buffer::{Buffer, BufferWorkingState};
use clean::Cleaner;

const CONTROLLER_PARAMS: ControllerParams = ControllerParams { kp: 0.0006 };

#[derive(PartialEq)]
enum State {
    NotWorking,
    Waiting,
    Working,
}

struct FasState {
    mode: Mode,
    working_state: State,
    delay_timer: Instant,
    buffer: Option<Buffer>,
}

struct AnalyzerState {
    analyzer: Analyzer,
    restart_counter: u8,
    restart_timer: Instant,
}

pub struct Looper {
    analyzer_state: AnalyzerState,
    config: Config,
    node: Node,
    extension: Extension,
    controller: Controller,
    therminal: Thermal,
    windows_watcher: TimedWatcher,
    cleaner: Cleaner,
    fas_state: FasState,
}

impl Looper {
    pub fn new(
        analyzer: Analyzer,
        config: Config,
        node: Node,
        extension: Extension,
        controller: Controller,
    ) -> Self {
        Self {
            analyzer_state: AnalyzerState {
                analyzer,
                restart_counter: 0,
                restart_timer: Instant::now(),
            },
            config,
            node,
            extension,
            controller,
            therminal: Thermal::new().unwrap(),
            windows_watcher: TimedWatcher::new(),
            cleaner: Cleaner::new(),
            fas_state: FasState {
                mode: Mode::Balance,
                buffer: None,
                working_state: State::NotWorking,
                delay_timer: Instant::now(),
            },
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            self.switch_mode();

            let _ = self.update_analyzer();
            self.retain_topapp();

            let fas_data = self.recv_message();

            if self.windows_watcher.visible_freeform_window() {
                self.disable_fas();
                continue;
            }

            if let Some(data) = fas_data {
                if let Some(state) = self.buffer_update(&data) {
                    match state {
                        BufferWorkingState::Usable => self.do_policy(),
                        BufferWorkingState::Unusable => self.disable_fas(),
                    }
                }
            } else if let Some(buffer) = self.fas_state.buffer.as_mut() {
                #[cfg(debug_assertions)]
                debug!("janked !");
                buffer.additional_frametime(&self.extension);

                match buffer.state.working_state {
                    BufferWorkingState::Unusable => {
                        self.restart_analyzer();
                        self.disable_fas();
                    }
                    BufferWorkingState::Usable => {
                        self.do_policy();
                    }
                }
            }
        }
    }

    fn switch_mode(&mut self) {
        if let Ok(new_mode) = self.node.get_mode() {
            if likely(self.fas_state.mode != new_mode) {
                info!(
                    "Switch mode: {} -> {}",
                    self.fas_state.mode.to_string(),
                    new_mode.to_string()
                );
                self.fas_state.mode = new_mode;

                if self.fas_state.working_state == State::Working {
                    self.controller.init_game(&self.extension);
                }
            }
        }
    }

    fn recv_message(&mut self) -> Option<FasData> {
        self.analyzer_state
            .analyzer
            .recv_timeout(Duration::from_millis(100))
            .map(|(pid, frametime)| FasData { pid, frametime })
    }

    fn update_analyzer(&mut self) -> Result<()> {
        use crate::framework::utils::get_process_name;

        for pid in self.windows_watcher.topapp_pids().iter().copied() {
            let pkg = get_process_name(pid)?;
            if self.config.need_fas(&pkg) {
                self.analyzer_state.analyzer.attach_app(pid)?;
            }
        }

        Ok(())
    }

    fn restart_analyzer(&mut self) {
        if self.analyzer_state.restart_counter == 1 {
            if self.analyzer_state.restart_timer.elapsed() >= Duration::from_secs(1) {
                self.analyzer_state.restart_timer = Instant::now();
                self.analyzer_state.restart_counter = 0;
                self.analyzer_state.analyzer.detach_apps();
                let _ = self.update_analyzer();
            }
        } else {
            self.analyzer_state.restart_counter += 1;
        }
    }

    fn do_policy(&mut self) {
        if unlikely(self.fas_state.working_state != State::Working) {
            #[cfg(debug_assertions)]
            debug!("Not running policy!");
            return;
        }

        let control = if let Some(buffer) = &self.fas_state.buffer {
            let target_fps_offset = self
                .therminal
                .target_fps_offset(&mut self.config, self.fas_state.mode);
            calculate_control(
                buffer,
                &mut self.config,
                self.fas_state.mode,
                CONTROLLER_PARAMS,
                target_fps_offset,
            )
            .unwrap_or_default()
        } else {
            return;
        };

        #[cfg(debug_assertions)]
        debug!("control: {control}khz");

        self.controller.fas_update_freq(control);
    }
}
