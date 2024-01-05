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
mod binder;
mod looper;
mod topapp;

use std::time::Duration;

use crate::{
    config::{Config, TargetFps},
    error::{Error, Result},
    node::Node,
    PerformanceController,
};

use self::binder::FasServer;
use looper::Looper;

#[derive(Debug, Clone)]
pub enum BinderMessage {
    Data(FasData),
    RemoveBuffer((i64, i32)),
}

#[derive(Debug, Clone)]
pub struct FasData {
    pub buffer: i64,
    pub target_fps: TargetFps,
    pub pkg: String,
    pub pid: i32,
    pub frametime: Duration,
    pub cpu: i32,
}

pub struct Scheduler<P: PerformanceController> {
    controller: Option<P>,
    config: Option<Config>,
}

impl<P: PerformanceController> Scheduler<P> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            controller: None,
            config: None,
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn config(mut self, c: Config) -> Self {
        self.config = Some(c);
        self
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn controller(mut self, c: P) -> Self {
        self.controller = Some(c);
        self
    }

    pub fn start_run(self) -> Result<()> {
        let node = Node::init()?;
        let config = self.config.ok_or(Error::SchedulerMissing("Config"))?;

        let controller = self
            .controller
            .ok_or(Error::SchedulerMissing("Controller"))?;

        let rx = FasServer::run_server(config.clone())?;

        Looper::new(rx, config, node, controller).enter_loop()
    }
}
