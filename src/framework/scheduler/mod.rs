// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

mod looper;
mod thermal;
mod topapp;

use std::time::Duration;

use super::{
    config::Config,
    error::{Error, Result},
    node::Node,
    Extension,
};
use crate::Controller;

use frame_analyzer::Analyzer;
use looper::Looper;

#[derive(Debug, Clone, Copy)]
pub struct FasData {
    pub pid: i32,
    pub frametime: Duration,
}

pub struct Scheduler {
    controller: Option<Controller>,
    config: Option<Config>,
}

impl Scheduler {
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
    pub fn controller(mut self, c: Controller) -> Self {
        self.controller = Some(c);
        self
    }

    pub fn start_run(self) -> Result<()> {
        let extension = Extension::init()?;
        let config = self.config.ok_or(Error::SchedulerMissing("Config"))?;

        let controller = self
            .controller
            .ok_or(Error::SchedulerMissing("Controller"))?;

        let node = Node::init()?;
        let analyzer = Analyzer::new()?;

        Looper::new(analyzer, config, node, extension, controller).enter_loop()
    }
}
