// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    collections::HashMap,
    fs,
    os::unix,
    time::{Duration, Instant},
};

use anyhow::Result;
use flower::flow_web::AnalyzeData;
use likely_stable::unlikely;

use crate::file_handler::FileHandler;

#[derive(Debug)]
pub struct Data {
    pub datas: Vec<AnalyzeData>,
    pub threads: Vec<u32>,
}

struct ForgetData<T> {
    data: T,
    instant: Instant,
    out: Duration,
}

impl<T> ForgetData<T> {
    pub fn new(data: T, out: Duration) -> Self {
        Self {
            data,
            instant: Instant::now(),
            out,
        }
    }

    pub fn get(&self) -> Option<&T> {
        if unlikely(self.outed()) {
            None
        } else {
            Some(&self.data)
        }
    }

    pub fn outed(&self) -> bool {
        self.instant.elapsed() > self.out
    }
}

fn init_cgroup_fs(cpuset_big: &[usize], cpuset_middle: &[usize]) -> Result<()> {
    let _ = fs::create_dir("/dev/cpuset/fas-rs");

    let cpus = fs::read_to_string("/sys/devices/system/cpu/possible")?;
    let _ = fs::write("/dev/cpuset/fas-rs/cpus", cpus);
    let _ = fs::write("/dev/cpuset/fas-rs/mems", "0");

    let _ = fs::create_dir("/dev/cpuset/fas-rs/critical");
    let _ = fs::set_permissions(
        "/dev/cpuset/fas-rs/critical",
        unix::fs::PermissionsExt::from_mode(0o755),
    );
    let cpus = format!(
        "{}-{}",
        cpuset_big.iter().min().unwrap(),
        cpuset_big.iter().max().unwrap()
    );
    let _ = fs::write("/dev/cpuset/fas-rs/critical/cpus", cpus);
    let _ = fs::write("/dev/cpuset/fas-rs/critical/mems", "0");

    let _ = fs::create_dir("/dev/cpuset/fas-rs/simple");
    let _ = fs::set_permissions(
        "/dev/cpuset/fas-rs/simple",
        unix::fs::PermissionsExt::from_mode(0o755),
    );
    let cpus = format!(
        "{}-{}",
        cpuset_middle.iter().min().unwrap(),
        cpuset_middle.iter().max().unwrap()
    );
    let _ = fs::write("/dev/cpuset/fas-rs/simple/cpus", cpus);
    let _ = fs::write("/dev/cpuset/fas-rs/simple/mems", "0");

    Ok(())
}

pub struct AffinityApplyer {
    file_handler: FileHandler,
    task_map: HashMap<u32, ForgetData<bool>>,
    gc_instant: Instant,
}

impl AffinityApplyer {
    pub fn new(cpuset_big: &[usize], cpuset_middle: &[usize]) -> Self {
        let _ = init_cgroup_fs(cpuset_big, cpuset_middle);
        Self {
            file_handler: FileHandler::new(),
            task_map: HashMap::new(),
            gc_instant: Instant::now(),
        }
    }

    pub fn apply(&mut self, data: Data) {
        if unlikely(self.gc_instant.elapsed() > Duration::from_secs(1)) {
            self.task_map.retain(|_, data| !data.outed());
            self.gc_instant = Instant::now();
        }

        let critical_thread = data.datas.iter().last().map(|data| data.tid).unwrap();
        if !self
            .task_map
            .get(&critical_thread)
            .and_then(|data| data.get())
            .copied()
            .unwrap_or(false)
        {
            self.task_map.insert(
                critical_thread,
                ForgetData::new(true, Duration::from_millis(100)),
            );
            let _ = self.file_handler.write(
                "/dev/cpuset/fas-rs/critical/tasks",
                critical_thread.to_string(),
            );
        }

        for tid in data.threads {
            if critical_thread == tid {
                continue;
            }

            if self.task_map
                .get(&tid)
                .and_then(|data| data.get())
                .copied()
                .unwrap_or(true)
            {
                self.task_map.insert(tid, ForgetData::new(false, Duration::from_millis(100)));
                let _ = self.file_handler.write("/dev/cpuset/fas-rs/simple/tasks", tid.to_string());
            }
        }
    }
}
