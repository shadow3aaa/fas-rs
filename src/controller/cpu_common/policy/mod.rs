/* Copyright 2023 shadow3aaa@gitbub.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License. */
mod cycles;
mod schedule;

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use atomic::Atomic;
use cpu_cycles_reader::Cycles;

use super::CpuCommon;
use cycles::DiffReader;
use schedule::Schedule;

pub struct Policy {
    target_diff: Arc<Atomic<Cycles>>,
    pub cur_cycles: Arc<Atomic<Cycles>>,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Drop for Policy {
    fn drop(&mut self) {
        self.exit.store(true, Ordering::Release);
        self.resume();
    }
}

impl Policy {
    pub fn new(policy_path: &Path) -> Self {
        let mut reader = DiffReader::new(policy_path);
        let mut schedule = Schedule::new(policy_path);
        let target_diff = schedule.target_diff.clone();
        let cur_cycles = schedule.cur_cycles.clone();

        let pause = Arc::new(AtomicBool::new(!CpuCommon::always_on()));
        let exit = Arc::new(AtomicBool::new(false));

        let handle = {
            let pause = pause.clone();
            let exit = exit.clone();
            thread::Builder::new()
                .name("CpuPolicyThread".into())
                .spawn(move || {
                    schedule.init();
                    loop {
                        if pause.load(Ordering::Acquire) {
                            schedule.deinit();
                            thread::park();
                            schedule.init();
                        } else if exit.load(Ordering::Acquire) {
                            schedule.deinit();
                            return;
                        }

                        let cur_freq = schedule.cur_cycles.load(Ordering::Acquire);
                        let diff = reader.read_diff(cur_freq);
                        schedule.run(diff);
                    }
                })
                .unwrap()
        };

        Self {
            target_diff,
            cur_cycles,
            pause,
            exit,
            handle,
        }
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::Release);
        self.handle.thread().unpark();
    }

    pub fn pause(&self) {
        self.pause.store(true, Ordering::Release);
    }

    pub fn move_target_diff(&self, c: Cycles) {
        let target_diff = self.target_diff.load(Ordering::Acquire) + c;
        let target_diff =
            target_diff.clamp(Cycles::new(0), self.cur_cycles.load(Ordering::Acquire));
        self.target_diff.store(target_diff, Ordering::Release);
    }

    pub fn set_target_diff(&self, c: Cycles) {
        self.target_diff.store(c, Ordering::Release);
    }
}
