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

use std::time::{Duration, Instant};

use likely_stable::unlikely;
use log::info;

use super::{
    super::FasData,
    buffer::BufferWorkingState,
    policy::evolution::{open_database, save_pid_params},
    Buffer, Looper, State,
};
use crate::{
    api::{v1::ApiV1, v2::ApiV2, v3::ApiV3},
    framework::{api::ApiV0, utils::get_process_name},
};

const DELAY_TIME: Duration = Duration::from_secs(3);

impl Looper {
    pub fn retain_topapp(&mut self) {
        if let Some(buffer) = self.fas_state.buffer.as_ref() {
            if !self
                .windows_watcher
                .topapp_pids()
                .contains(&buffer.package_info.pid)
            {
                let _ = self.analyzer.detach_app(buffer.package_info.pid);
                let pkg = buffer.package_info.pkg.clone();
                if save_pid_params(&self.database, &pkg, self.evolution_state.pid_params).is_err() {
                    self.database = open_database().unwrap();
                }
                self.extension
                    .trigger_extentions(ApiV0::UnloadFas(buffer.package_info.pid, pkg.clone()));
                self.extension
                    .trigger_extentions(ApiV1::UnloadFas(buffer.package_info.pid, pkg.clone()));
                self.extension
                    .trigger_extentions(ApiV2::UnloadFas(buffer.package_info.pid, pkg.clone()));
                self.extension
                    .trigger_extentions(ApiV3::UnloadFas(buffer.package_info.pid, pkg));
                self.fas_state.buffer = None;
            }
        }

        if self.fas_state.buffer.is_none() {
            self.disable_fas();
        } else {
            self.enable_fas();
        }
    }

    pub fn disable_fas(&mut self) {
        match self.fas_state.working_state {
            State::Working => {
                self.fas_state.working_state = State::NotWorking;
                self.cleaner.undo_cleanup();
                self.controller.init_default(&self.extension);
                if let Some(buffer) = &self.fas_state.buffer {
                    let _ = self.analyzer.detach_app(buffer.package_info.pid);
                }
                self.extension.trigger_extentions(ApiV0::StopFas);
                self.extension.trigger_extentions(ApiV1::StopFas);
                self.extension.trigger_extentions(ApiV2::StopFas);
                self.extension.trigger_extentions(ApiV3::StopFas);
            }
            State::Waiting => self.fas_state.working_state = State::NotWorking,
            State::NotWorking => (),
        }
    }

    pub fn enable_fas(&mut self) {
        match self.fas_state.working_state {
            State::NotWorking => {
                self.fas_state.working_state = State::Waiting;
                self.fas_state.delay_timer = Instant::now();
                self.extension.trigger_extentions(ApiV0::StartFas);
                self.extension.trigger_extentions(ApiV1::StartFas);
                self.extension.trigger_extentions(ApiV2::StartFas);
                self.extension.trigger_extentions(ApiV3::StartFas);
            }
            State::Waiting => {
                if self.fas_state.delay_timer.elapsed() > DELAY_TIME {
                    self.fas_state.working_state = State::Working;
                    self.cleaner.cleanup();
                    self.controller.init_game(&self.extension);
                }
            }
            State::Working => (),
        }
    }

    pub fn buffer_update(&mut self, data: &FasData) -> Option<BufferWorkingState> {
        if unlikely(
            !self.windows_watcher.topapp_pids().contains(&data.pid) || data.frametime.is_zero(),
        ) {
            return None;
        }

        let pid = data.pid;
        let frametime = data.frametime;

        if let Some(buffer) = self.fas_state.buffer.as_mut() {
            buffer.push_frametime(frametime, &self.extension);
            Some(buffer.buffer_state.working_state)
        } else {
            let Ok(pkg) = get_process_name(data.pid) else {
                return None;
            };
            let target_fps = self.config.target_fps(&pkg)?;

            info!("New fas buffer on: [{pkg}]");

            self.evolution_state.reset(&self.database, &pkg);

            self.extension
                .trigger_extentions(ApiV0::LoadFas(pid, pkg.clone()));
            self.extension
                .trigger_extentions(ApiV1::LoadFas(pid, pkg.clone()));
            self.extension
                .trigger_extentions(ApiV2::LoadFas(pid, pkg.clone()));
            self.extension
                .trigger_extentions(ApiV3::LoadFas(pid, pkg.clone()));

            let mut buffer = Buffer::new(target_fps, pid, pkg);
            buffer.push_frametime(frametime, &self.extension);

            self.fas_state.buffer = Some(buffer);

            Some(BufferWorkingState::Unusable)
        }
    }
}
