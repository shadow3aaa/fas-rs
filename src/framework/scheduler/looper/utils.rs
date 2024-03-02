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
use std::{
    collections::hash_map::Entry,
    time::{Duration, Instant},
};

use log::info;

use super::{super::FasData, Buffer, Looper, State};
use crate::framework::{utils::get_process_name, CallBacks};

impl Looper {
    pub fn retain_topapp(&mut self) {
        self.buffers.retain(|(_, p), _| {
            if self.topapp_checker.is_topapp(*p) {
                true
            } else {
                let pkg = get_process_name(*p).unwrap();
                self.extension
                    .call_extentions(CallBacks::UnloadFas(*p, pkg));
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
                self.extension.call_extentions(CallBacks::StopFas);
                self.controller.init_default(&self.config, &self.extension);
                self.state = State::NotWorking;
            }
            State::Waiting => self.state = State::NotWorking,
            State::NotWorking => (),
        }
    }

    pub fn enable_fas(&mut self) {
        match self.state {
            State::NotWorking => {
                self.extension.call_extentions(CallBacks::StartFas);
                self.delay_timer = Instant::now();
                self.state = State::Waiting;
            }
            State::Waiting => {
                if self.delay_timer.elapsed() > Duration::from_secs(10) {
                    self.controller
                        .init_game(self.mode, &self.config, &self.extension);
                    self.state = State::Working;
                }
            }
            State::Working => (),
        }
    }

    pub fn buffer_update(&mut self, d: &FasData) {
        if !self.topapp_checker.is_topapp(d.pid) || d.frametime.is_zero() {
            return;
        }

        let producer = (d.buffer, d.pid);
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
                let pkg = get_process_name(d.pid).unwrap();
                let target_fps = self.config.target_fps(&pkg).unwrap();

                info!("New fas buffer on: [{pkg}]");

                self.extension
                    .call_extentions(CallBacks::LoadFas(d.pid, pkg));

                let mut buffer = Buffer::new(target_fps);
                buffer.push_frametime(frametime);
                v.insert(buffer);
                self.topapp_checker.refresh();
            }
        }
    }
}
