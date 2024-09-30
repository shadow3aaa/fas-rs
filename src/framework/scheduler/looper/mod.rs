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

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use frame_analyzer::Analyzer;
use likely_stable::unlikely;
#[cfg(debug_assertions)]
use log::debug;
use log::info;
use policy::{
    evolution::{evaluate_fitness, mutate_params, open_database},
    pid_controll::pid_control,
    PidParams,
};
use rusqlite::Connection;

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

use buffer::{Buffer, BufferState};
use clean::Cleaner;

#[derive(PartialEq)]
enum State {
    NotWorking,
    Waiting,
    Working,
}

pub struct Looper {
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
    database: Connection,
    pid_params: PidParams,
    mutated_pid_params: PidParams,
    fitness: f64,
    control_history: VecDeque<isize>,
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
            database: open_database().unwrap(),
            pid_params: PidParams::default(),
            mutated_pid_params: PidParams::default(),
            fitness: f64::MIN,
            control_history: VecDeque::with_capacity(30),
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            self.switch_mode();

            let _ = self.update_analyzer();
            self.retain_topapp();

            let target_fps = self.buffer.as_ref().and_then(|b| b.target_fps);
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
                        BufferState::Usable => self.do_policy(),
                        BufferState::Unusable => self.disable_fas(),
                    }
                }
            } else if let Some(buffer) = self.buffer.as_mut() {
                self.janked = true;
                #[cfg(debug_assertions)]
                debug!("janked: {}", self.janked);
                buffer.additional_frametime();
                self.do_policy();
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

    fn recv_message(&mut self, target_fps: Option<u32>) -> Option<FasData> {
        let target_frametime = target_fps.map(|fps| Duration::from_secs(1) / fps);

        let time = if unlikely(self.state != State::Working) {
            Duration::from_millis(100)
        } else if unlikely(self.janked) {
            target_frametime.map_or(Duration::from_millis(100), |time| time / 4)
        } else {
            target_frametime.map_or(Duration::from_millis(100), |time| time * 2)
        };

        self.analyzer
            .recv_timeout(time)
            .map(|(pid, frametime)| FasData { pid, frametime })
    }

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

    fn do_policy(&mut self) {
        if unlikely(self.state != State::Working) {
            #[cfg(debug_assertions)]
            debug!("Not running policy!");
            return;
        }

        if let Some(fitness) = self.buffer.as_ref().and_then(|buffer| {
            evaluate_fitness(buffer, &mut self.config, self.mode, &self.control_history)
        }) {
            if fitness > self.fitness {
                self.pid_params = self.mutated_pid_params;
            }

            self.fitness = fitness;
        }

        self.mutated_pid_params = mutate_params(self.pid_params);
        let Some(control) = self.buffer.as_ref().and_then(|buffer| {
            pid_control(buffer, &mut self.config, self.mode, self.mutated_pid_params)
        }) else {
            self.disable_fas();
            return;
        };

        #[cfg(debug_assertions)]
        debug!("control: {control}khz");

        self.controller.fas_update_freq(control);

        if self.control_history.len() >= 30 {
            self.control_history.pop_back();
        }
        self.control_history.push_front(control);
    }
}
