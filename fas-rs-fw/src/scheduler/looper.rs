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
    collections::hash_map::{Entry, HashMap},
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

const JANK_REC: usize = 3;
const BIG_JANK_REC: usize = 5;

type Buffers = HashMap<Process, (Duration, Duration)>; // Process, (jank_scale, total_jank_time)
type Process = (String, i32); // process, pid

pub struct Looper<P: PerformanceController> {
    rx: Receiver<FasData>,
    config: Config,
    controller: P,
    topapp_checker: TimedWatcher,
    buffers: Buffers,
    started: bool,
    jank_counter: usize,
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
            jank_counter: JANK_REC,
        })
    }

    pub fn enter_loop(&mut self) -> Result<()> {
        loop {
            let data = self
                .rx
                .recv()
                .map_err(|_| Error::Other("Got an error when recving binder data"))?;

            if !self.check_topapp(data.pid)? {
                continue;
            }

            self.buffer_update(&data);
            self.buffer_policy()?;

            debug!("{:#?}", self.buffers);
        }
    }

    /* 检查是否为顶层应用，并且删除不是顶层应用的buffer **/
    fn check_topapp(&mut self, p: i32) -> Result<bool> {
        self.buffers
            .retain(|(_, p), _| self.topapp_checker.is_topapp(*p).unwrap_or(false));
        self.topapp_checker.is_topapp(p) // binder server已经忽略了非列表内应用，因此这里只用检查是否是顶层应用
    }

    fn buffer_update(&mut self, d: &FasData) {
        if d.frametime.is_zero() {
            return;
        } else if d.target_fps == 0 {
            panic!("Target fps must be bigger than zero");
        }

        let process = (d.pkg.clone(), d.pid);
        let scale_time = Duration::from_secs(1)
            .checked_div(d.target_fps)
            .unwrap_or_default();
        let jank_time = d.frametime.saturating_sub(scale_time);

        match self.buffers.entry(process) {
            Entry::Occupied(mut o) => {
                let value = o.get_mut();
                if value.0 == scale_time {
                    value.1 += jank_time;
                } else {
                    value.0 = scale_time;
                    value.1 = Duration::ZERO;
                }
            }
            Entry::Vacant(v) => {
                v.insert((scale_time, jank_time));
            }
        }
    }

    fn buffer_policy(&mut self) -> Result<()> {
        if self.buffers.is_empty() && self.started {
            self.controller.init_default(&self.config)?;
            self.started = false;
            return Ok(());
        } else if !self.started {
            self.controller.init_game(&self.config)?;
            self.started = true;
        }

        let level = self
            .buffers
            .values_mut()
            .filter_map(|(scale_time, jank_time)| {
                let result = if *jank_time > *scale_time {
                    self.jank_counter = BIG_JANK_REC;
                    Some(10)
                } else if *jank_time > *scale_time / 2 {
                    self.jank_counter = JANK_REC;
                    Some(5)
                } else if *jank_time > *scale_time / 4 {
                    self.jank_counter = JANK_REC;
                    Some(2)
                } else {
                    None
                };

                if result.is_some() {
                    *jank_time = Duration::ZERO;
                }

                result
            })
            .max()
            .unwrap_or_default();

        debug!("jank-level: {level}");

        if level == 0 && self.jank_counter > 0 {
            self.jank_counter -= 1;
            return Ok(());
        }

        self.controller.perf(level, &self.config);

        Ok(())
    }
}
