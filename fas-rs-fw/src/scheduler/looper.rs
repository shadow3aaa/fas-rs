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
    collections::{hash_map::HashMap, VecDeque},
    sync::mpsc::Receiver,
    time::Duration,
};

use log::debug;

use super::{topapp::TimedWatcher, FasData};
use crate::{
    config::Config,
    error::{Error, Result},
    PerformanceController,
};

const BUFFER_CAP: usize = 1024;

type FrameTimeBuffers = HashMap<Process, VecDeque<Duration>>;
type Process = (String, i32);

pub struct Looper<P: PerformanceController> {
    rx: Receiver<FasData>,
    config: Config,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: FrameTimeBuffers,
}

impl<P: PerformanceController> Looper<P> {
    pub fn new(rx: Receiver<FasData>, config: Config, controller: P) -> Result<Self> {
        Ok(Self {
            rx,
            config,
            controller,
            topapp_checker: TimedWatcher::new()?,
            buffers: HashMap::new(),
        })
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let data = self
                .rx
                .recv()
                .map_err(|_e| Error::Other("Got an error when recving binder data"))?;

            if !self.check_topapp(data.pid)? {
                continue;
            }

            self.buffer_push((data.pkg, data.pid), data.frametime);

            debug!("{:#?}", self.buffers);
        }
    }

    /* 检查是否为顶层应用，并且删除不是顶层应用的buffer **/
    fn check_topapp(&mut self, p: i32) -> Result<bool> {
        self.buffers
            .retain(|(_, p), _| self.topapp_checker.is_topapp(*p).unwrap_or(false));
        self.topapp_checker.is_topapp(p) // binder server已经忽略了非列表内应用，因此这里只用检查是否是顶层应用
    }

    fn buffer_push(&mut self, p: Process, d: Duration) {
        let buffer = self
            .buffers
            .entry(p)
            .or_insert_with(|| VecDeque::with_capacity(BUFFER_CAP));

        if buffer.len() >= BUFFER_CAP {
            buffer.pop_back();
        }

        buffer.push_front(d);
    }
}
