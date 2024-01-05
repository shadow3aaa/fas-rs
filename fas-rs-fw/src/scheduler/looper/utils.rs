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
use std::collections::hash_map::Entry;

use log::info;

use super::{super::FasData, Buffer, Looper};
use crate::{config::TargetFps, error::Result, node::Mode, PerformanceController};

impl<P: PerformanceController> Looper<P> {
    pub fn retain_topapp(&mut self, mode: Mode) -> Result<()> {
        self.buffers
            .retain(|(_, p), _| self.topapp_checker.is_topapp(*p));

        if self.buffers.is_empty() {
            self.disable_fas(mode)?;
        } else {
            self.enable_fas(mode)?;
        }

        Ok(())
    }

    pub fn disable_fas(&mut self, mode: Mode) -> Result<()> {
        if self.started {
            self.controller.init_default(mode, &self.config)?;
            self.started = false;
        }

        Ok(())
    }

    pub fn enable_fas(&mut self, mode: Mode) -> Result<()> {
        if !self.started {
            self.controller.init_game(mode, &self.config)?;
            self.started = true;
        }

        Ok(())
    }

    pub fn buffer_update(&mut self, d: &FasData) {
        if !self.topapp_checker.is_topapp(d.pid) || d.frametime.is_zero() {
            return;
        } else if d.target_fps == TargetFps::Value(0) {
            panic!("Target fps must be bigger than zero");
        }

        let producer = (d.buffer, d.pid);
        let frametime = d.frametime;
        let target_fps = d.target_fps.clone();

        for (process, buffer) in self.buffers.iter_mut() {
            if *process != producer {
                buffer.frame_prepare(); // 其它buffer计算额外超时时间
            }
        }

        match self.buffers.entry(producer) {
            Entry::Occupied(mut o) => {
                o.get_mut().push_frametime(frametime);
            }
            Entry::Vacant(v) => {
                info!("New fas buffer on game: [{}] pid: [{}]", d.pkg, d.pid);

                let mut buffer = Buffer::new(target_fps);
                buffer.push_frametime(frametime);
                v.insert(buffer);
            }
        }
    }
}
