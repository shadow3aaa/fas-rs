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
mod cycles;
mod schedule;

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use anyhow::Result;
use atomic::Atomic;
use cpu_cycles_reader::Cycles;
use fas_rs_fw::prelude::*;

use cycles::DiffReader;
use schedule::Schedule;

pub struct Policy {
    target_diff: Arc<Atomic<Cycles>>,
    max_diff: Arc<Atomic<Cycles>>,
    exit: Arc<AtomicBool>,
}

impl Drop for Policy {
    fn drop(&mut self) {
        self.exit.store(true, Ordering::Release);
    }
}

impl Policy {
    pub fn new(policy_path: &Path, config: &Config) -> Result<Self> {
        let mut reader = DiffReader::new(policy_path, config)?;
        let mut schedule = Schedule::new(policy_path, config)?;
        let target_diff = schedule.target_diff.clone();
        let max_diff = schedule.max_diff.clone();

        let exit = Arc::new(AtomicBool::new(false));

        {
            let exit = exit.clone();
            thread::Builder::new()
                .name("CpuPolicyThread".into())
                .spawn(move || {
                    schedule.init();
                    loop {
                        if exit.load(Ordering::Acquire) {
                            return;
                        }

                        let cur_freq = schedule.cur_freq.load(Ordering::Acquire);
                        let diff = reader.read_diff(cur_freq);
                        schedule.run(diff);
                    }
                })
                .unwrap()
        };

        Ok(Self {
            target_diff,
            max_diff,
            exit,
        })
    }

    pub fn move_target_diff(&self, c: Cycles) {
        let target_diff = self.target_diff.load(Ordering::Acquire) + c;
        let target_diff = target_diff.max(Cycles::new(0));
        self.target_diff.store(target_diff, Ordering::Release);
    }

    pub fn set_target_diff(&self, c: Cycles) {
        let c = c.clamp(Cycles::new(0), self.max_diff.load(Ordering::Acquire));
        self.target_diff.store(c, Ordering::Release);
    }
}
