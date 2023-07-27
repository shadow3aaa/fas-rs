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
//! 提供一个帧感知调度基本逻辑和一些api

mod frame;

use std::{
    error::Error,
    sync::Arc,
    thread::{self, JoinHandle},
};

use atomic::{Atomic, Ordering};

use crate::{TargetFps, VirtualFrameSensor, VirtualPerformanceController};

/// [`self::Scheduler`]通过[`crate::VirtualFrameSensor`]和[`crate::VirtualPerformanceController`]来进行调度
pub struct Scheduler {
    command: Arc<Atomic<Command>>,
    handle: JoinHandle<()>,
}

#[derive(Clone, Copy)]
enum Command {
    Load((TargetFps, u32)),
    Unload,
    Exit,
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.command.store(Command::Exit, Ordering::Release);
        self.handle.thread().unpark();
    }
}

impl Scheduler {
    /// 构造一个[`self::Scheduler`]并且初始化
    ///
    /// # Errors
    ///
    /// 暂停控制器/传感器失败
    ///
    /// # Panics
    ///
    /// 创建线程失败
    pub fn new(
        sensor: Box<dyn VirtualFrameSensor>,
        controller: Box<dyn VirtualPerformanceController>,
    ) -> Result<Self, Box<dyn Error>> {
        sensor.pause()?;
        controller.plug_out()?;

        let command = Arc::new(Atomic::new(Command::Unload));

        let handle = {
            let command = command.clone();

            thread::Builder::new()
                .name("SchedulerThread".into())
                .spawn(move || Self::run(&*sensor, &*controller, &command))
                .unwrap()
        };

        Ok(Self { command, handle })
    }

    /// 卸载[`self::Scheduler`]
    ///
    /// 用于临时暂停
    #[inline]
    pub fn unload(&self) {
        self.command.store(Command::Unload, Ordering::Release);
    }

    /// 载入[`self::Scheduler`]
    ///
    /// 如果已经载入，再次调用会重载入(调用init)
    /// 每次载入/重载要指定新的[`crate::TargetFps`]
    #[inline]
    pub fn load(&self, target: TargetFps, windows: u32) {
        self.command
            .store(Command::Load((target, windows)), Ordering::Release);
        self.handle.thread().unpark();
    }
}

impl Scheduler {
    fn run(
        sensor: &dyn VirtualFrameSensor,
        controller: &dyn VirtualPerformanceController,
        command: &Arc<Atomic<Command>>,
    ) {
        let mut target_fps;
        let mut windows: u32;

        loop {
            let sleep_time = match command.load(Ordering::Acquire) {
                Command::Load(t) => {
                    (target_fps, windows) = t;
                    Self::init_load(sensor, controller, windows).unwrap()
                }
                Command::Unload => {
                    Self::process_unload(sensor, controller).unwrap();
                    thread::park();
                    continue;
                }
                Command::Exit => return,
            };

            Self::process_load(sensor, controller, target_fps);
            thread::sleep(sleep_time);
        }
    }
}
