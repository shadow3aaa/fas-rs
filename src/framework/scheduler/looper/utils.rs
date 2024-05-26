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

use std::{
    collections::hash_map::Entry,
    time::{Duration, Instant},
};

use log::info;

use super::{super::FasData, Buffer, Looper, State};
use crate::framework::{api::ApiV0, utils::get_process_name};

const DELAY_TIME: Duration = Duration::from_secs(3);

impl Looper {
    pub fn retain_topapp(&mut self) {
        self.buffers.retain(|pid, buffer| {
            if self.topapp_watcher.is_topapp(*pid) {
                true
            } else {
                #[cfg(feature = "use_ebpf")]
                let _ = self.analyzer.detach_app(*pid);
                let pkg = buffer.pkg.clone();
                self.extension
                    .tigger_extentions(ApiV0::UnloadFas(*pid, pkg));
                false
            }
        });

        if self.buffers.is_empty() {
            self.disable_fas();
        } else {
            self.enable_fas();
        }
    }

    pub fn disable_fas(&mut self) {
        match self.state {
            State::Working => {
                self.state = State::NotWorking;
                self.cleaner.undo_cleanup();
                self.controller.init_default(&self.config, &self.extension);
                self.extension.tigger_extentions(ApiV0::StopFas);
            }
            State::Waiting => self.state = State::NotWorking,
            State::NotWorking => (),
        }
    }

    pub fn enable_fas(&mut self) {
        match self.state {
            State::NotWorking => {
                self.state = State::Waiting;
                self.delay_timer = Instant::now();
                self.extension.tigger_extentions(ApiV0::StartFas);
            }
            State::Waiting => {
                if self.delay_timer.elapsed() > DELAY_TIME {
                    self.state = State::Working;
                    self.cleaner.cleanup();
                    self.controller.init_game(&self.config, &self.extension);
                }
            }
            State::Working => (),
        }
    }

    pub fn buffer_update(&mut self, d: &FasData) {
        if !self.topapp_watcher.is_topapp(d.pid) || d.frametime.is_zero() {
            return;
        }

        let producer = d.pid;
        let frametime = d.frametime;

        for (process, buffer) in &mut self.buffers {
            if *process != producer {
                buffer.frame_prepare(); // 其它buffer计算额外超时时间
            }
        }

        match self.buffers.entry(producer) {
            Entry::Occupied(mut o) => {
                o.get_mut().push_frametime(frametime);
            }
            Entry::Vacant(v) => {
                let Ok(pkg) = get_process_name(d.pid) else {
                    return;
                };
                let Some(target_fps) = self.config.target_fps(&pkg) else {
                    return;
                };

                info!("New fas buffer on: [{pkg}]");

                self.extension
                    .tigger_extentions(ApiV0::LoadFas(d.pid, pkg.clone()));

                let mut buffer = Buffer::new(target_fps, pkg);
                buffer.push_frametime(frametime);
                v.insert(buffer);
            }
        }
    }
}
