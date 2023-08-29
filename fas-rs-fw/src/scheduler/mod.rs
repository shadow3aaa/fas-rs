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
mod main_loop;

use log::info;
use surfaceflinger_hook_api::Connection;

use crate::{
    config::Config,
    error::{Error, Result},
    PerformanceController,
};

/// 调度器
pub struct Scheduler<P: PerformanceController> {
    controller: Option<P>,
    config: Option<Config>,
    jank_level_max: Option<u32>,
}

impl<P: PerformanceController> Scheduler<P> {
    /// 构造调度器并且初始化
    #[must_use]
    pub const fn new() -> Self {
        Self {
            controller: None,
            config: None,
            jank_level_max: None,
        }
    }

    /// 配置
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn config(mut self, c: Config) -> Self {
        self.config = Some(c);
        self
    }

    /// 控制器
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn controller(mut self, c: P) -> Self {
        self.controller = Some(c);
        self
    }

    /// jank-level上限，超过的视为上限
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn jank_level_max(mut self, l: u32) -> Self {
        self.jank_level_max = Some(l);
        self
    }

    /// 运行
    ///
    /// # Errors
    ///
    /// 缺少必要参数构建失败
    pub fn start_run(self) -> Result<()> {
        let mut config = self.config.ok_or(Error::SchedulerMissing("Config"))?;
        let controller = self
            .controller
            .ok_or(Error::SchedulerMissing("Controller"))?;
        info!("Connecting to hook");
        let mut connection = Connection::init_and_wait()?;
        info!("Connected to hook");

        Self::main_loop(
            &mut config,
            &controller,
            &mut connection,
            self.jank_level_max,
        )
    }
}
