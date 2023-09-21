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
mod policy;
mod utils;

use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use log::debug;
use sliding_features::{Echo, View, ALMA};

use super::{topapp::TimedWatcher, FasData};
use crate::{
    config::Config,
    error::{Error, Result},
    PerformanceController,
};

const BUFFER_LEN: usize = 144;

pub type Buffers = HashMap<Process, Buffer>; // Process, (jank_scale, total_jank_time_ns)
pub type Process = (String, i32); // process, pid

#[derive(Debug)]
pub struct Buffer {
    pub scale: Duration,
    pub target_fps: u32,
    pub frametimes: VecDeque<Duration>,
    pub smoother: ALMA<Echo>,
}

impl Buffer {
    pub fn push_frametime(&mut self, d: Duration) {
        if self.frametimes.len() >= BUFFER_LEN {
            self.frametimes.pop_back();
        }

        self.smoother.update(d.as_nanos() as f64);
        let frametime = self.smoother.last();
        let frametime = Duration::from_nanos(frametime as u64);

        debug!("frametime: {frametime:?}");

        self.frametimes.push_front(frametime);
    }
}

pub struct Looper<P: PerformanceController> {
    rx: Receiver<FasData>,
    config: Config,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    started: bool,
}

impl<P: PerformanceController> Looper<P> {
    pub fn new(rx: Receiver<FasData>, config: Config, controller: P) -> Result<Self> {
        Ok(Self {
            rx,
            config,
            controller,
            topapp_checker: TimedWatcher::new()?,
            buffers: Buffers::new(),
            started: false,
        })
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let mut timeout = self
                .buffers
                .values()
                .map(|b| Duration::from_secs(1) / b.target_fps)
                .max()
                .unwrap_or_default();
            timeout *= 10; // 获取buffer中最大的 (标准帧时间 * 10) 作为接收超时时间

            let data = if timeout.is_zero() {
                Some(
                    self.rx
                        .recv()
                        .map_err(|_| Error::Other("Binder Disconnected"))?,
                )
            } else {
                match self.rx.recv_timeout(timeout) {
                    Ok(d) => Some(d),
                    Err(e) => {
                        if e == RecvTimeoutError::Disconnected {
                            return Err(Error::Other("Binder Disconnected"));
                        }

                        if self.started {
                            self.buffers
                                .values_mut()
                                .for_each(|b| b.push_frametime(timeout));
                        }

                        None
                    }
                }
            };

            if let Some(data) = data {
                self.buffer_update(&data);
            }

            self.retain_topapp();
            self.buffer_policy()?;

            debug!("{:#?}", self.buffers);
        }
    }
}
