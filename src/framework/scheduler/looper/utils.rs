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

use log::{error, info};

use super::{super::FasData, policy::JankEvent, Buffer, Looper, State};
use crate::framework::{config::TargetFps, CallBacks};

impl Looper {
    pub fn update_limit_delay(&mut self, new: Duration) {
        self.last_limit = Instant::now();
        self.limit_delay = self.limit_delay.max(new);
    }

    pub fn set_limit_delay(&mut self, new: Duration) {
        self.last_limit = Instant::now();
        self.limit_delay = new;
    }

    pub fn retain_topapp(&mut self) {
        self.buffers.retain(|(_, p), _| {
            if self.topapp_checker.is_topapp(*p) {
                true
            } else {
                self.extension.call_extentions(CallBacks::UnloadFas(*p));
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
        if self.state != State::NotWorking {
            self.extension.call_extentions(CallBacks::StopFas);
            self.controller.init_default(&self.config, &self.extension);
            self.state = State::NotWorking;
            self.jank_state = JankEvent::None;
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
        } else if d.target_fps == TargetFps::Value(0) {
            error!(
                "Target fps must be bigger than zero, reject to load fas on [{}]",
                d.pkg
            );
            return;
        }

        let producer = (d.buffer, d.pid);
        let frametime = d.frametime;
        let target_fps = d.target_fps.clone();

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
                self.extension.call_extentions(CallBacks::LoadFas(d.pid));

                info!("New fas buffer on game: [{}] pid: [{}]", d.pkg, d.pid);

                let mut buffer = Buffer::new(target_fps);
                buffer.push_frametime(frametime);
                v.insert(buffer);
                self.topapp_checker.refresh();
            }
        }
    }
}
