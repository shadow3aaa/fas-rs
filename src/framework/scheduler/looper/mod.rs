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

use std::time::{Duration, Instant};

use frame_analyzer::Analyzer;
use likely_stable::{likely, unlikely};
#[cfg(debug_assertions)]
use log::debug;
use log::info;
use policy::{
    evolution::{evaluate_fitness, load_pid_params, mutate_params, open_database, Fitness},
    pid_controll::pid_control,
    PidParams,
};
use rusqlite::Connection;

use super::{topapp::TimedWatcher, FasData};
use crate::{
    cpu_temp_watcher::CpuTempWatcher,
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

#[derive(PartialEq)]
enum State {
    NotWorking,
    Waiting,
    Working,
}

struct EvolutionState {
    pid_params: PidParams,
    mutated_pid_params: PidParams,
    mutate_timer: Instant,
    fitness: Fitness,
}

impl EvolutionState {
    pub fn reset(&mut self, database: &Connection, pkg: &str) {
        self.pid_params = load_pid_params(database, pkg).unwrap_or_else(|_| PidParams::default());
        self.mutated_pid_params = self.pid_params;
        self.fitness = Fitness::MIN;
    }

    pub fn try_evolution(
        &mut self,
        buffer: &Buffer,
        cpu_temp_watcher: &CpuTempWatcher,
        config: &mut Config,
        mode: Mode,
    ) {
        if unlikely(self.mutate_timer.elapsed() > Duration::from_secs(1)) {
            self.mutate_timer = Instant::now();

            if let Some(fitness) = evaluate_fitness(buffer, cpu_temp_watcher, config, mode) {
                if fitness > self.fitness {
                    self.pid_params = self.mutated_pid_params;
                }

                self.fitness = fitness;
            }

            self.mutated_pid_params = mutate_params(self.pid_params);
        }
    }
}

struct FasState {
    mode: Mode,
    working_state: State,
    janked: bool,
    delay_timer: Instant,
    buffer: Option<Buffer>,
}

pub struct Looper {
    analyzer: Analyzer,
    cpu_temp_watcher: CpuTempWatcher,
    config: Config,
    node: Node,
    extension: Extension,
    controller: Controller,
    windows_watcher: TimedWatcher,
    cleaner: Cleaner,
    database: Connection,
    fas_state: FasState,
    evolution_state: EvolutionState,
}

impl Looper {
    pub fn new(
        analyzer: Analyzer,
        cpu_temp_watcher: CpuTempWatcher,
        config: Config,
        node: Node,
        extension: Extension,
        controller: Controller,
    ) -> Self {
        Self {
            analyzer,
            cpu_temp_watcher,
            config,
            node,
            extension,
            controller,
            windows_watcher: TimedWatcher::new(),
            cleaner: Cleaner::new(),
            database: open_database().unwrap(),
            fas_state: FasState {
                mode: Mode::Balance,
                buffer: None,
                working_state: State::NotWorking,
                delay_timer: Instant::now(),
                janked: false,
            },
            evolution_state: EvolutionState {
                pid_params: PidParams::default(),
                mutated_pid_params: PidParams::default(),
                mutate_timer: Instant::now(),
                fitness: Fitness::MIN,
            },
        }
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            self.switch_mode();

            let _ = self.update_analyzer();
            self.retain_topapp();

            let target_fps = self
                .fas_state
                .buffer
                .as_ref()
                .and_then(|buffer| buffer.target_fps_state.target_fps);
            let fas_data = self.recv_message(target_fps);

            if self.windows_watcher.visible_freeform_window() {
                self.disable_fas();
                continue;
            }

            if let Some(data) = fas_data {
                self.fas_state.janked = false;
                #[cfg(debug_assertions)]
                debug!("janked: {}", self.fas_state.janked);

                if let Some(state) = self.buffer_update(&data) {
                    match state {
                        BufferWorkingState::Usable => self.do_policy(),
                        BufferWorkingState::Unusable => self.disable_fas(),
                    }
                }
            } else if let Some(buffer) = self.fas_state.buffer.as_mut() {
                self.fas_state.janked = true;
                #[cfg(debug_assertions)]
                debug!("janked: {}", self.fas_state.janked);
                buffer.additional_frametime(&self.extension);
                self.do_policy();
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

    fn recv_message(&mut self, target_fps: Option<u32>) -> Option<FasData> {
        let target_frametime = target_fps.map(|fps| Duration::from_secs(1) / fps);

        let time = if unlikely(self.fas_state.working_state != State::Working) {
            Duration::from_millis(100)
        } else if unlikely(self.fas_state.janked) {
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
        if unlikely(self.fas_state.working_state != State::Working) {
            #[cfg(debug_assertions)]
            debug!("Not running policy!");
            return;
        }

        let control = if let Some(buffer) = &self.fas_state.buffer {
            self.evolution_state.try_evolution(
                buffer,
                &self.cpu_temp_watcher,
                &mut self.config,
                self.fas_state.mode,
            );

            pid_control(
                buffer,
                &mut self.config,
                self.fas_state.mode,
                self.evolution_state.mutated_pid_params,
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
