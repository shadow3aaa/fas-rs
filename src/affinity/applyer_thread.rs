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
    collections::{HashMap, HashSet},
    fs,
    os::unix,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use anyhow::Result;
use flower::flow_web::AnalyzeData;

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
        if self.outed() {
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

pub fn affinity_applyer(rx: &Receiver<Data>, cpuset_big: &[usize], cpuset_middle: &[usize]) {
    let _ = init_cgroup_fs(cpuset_big, cpuset_middle);

    let mut file_handler = FileHandler::new();
    let mut task_map: HashMap<u32, ForgetData<bool>> = HashMap::new();
    let mut gc_instant = Instant::now();

    loop {
        if let Ok(data) = rx.recv() {
            if gc_instant.elapsed() > Duration::from_secs(1) {
                task_map.retain(|_, data| !data.outed());
                gc_instant = Instant::now();
            }

            let critical_threads: HashSet<_> = data.datas.iter().map(|data| data.tid).collect();
            for critical_thread in &critical_threads {
                if task_map
                    .get(critical_thread)
                    .and_then(|data| data.get())
                    .copied()
                    .unwrap_or(false)
                {
                    continue;
                }

                task_map.insert(
                    *critical_thread,
                    ForgetData::new(true, Duration::from_millis(100)),
                );
                file_handler
                    .write(
                        "/dev/cpuset/fas-rs/critical/tasks",
                        critical_thread.to_string(),
                    )
                    .unwrap();
            }

            for tid in data.threads {
                if critical_threads.contains(&tid) {
                    continue;
                }

                if !task_map
                    .get(&tid)
                    .and_then(|data| data.get())
                    .copied()
                    .unwrap_or(true)
                {
                    continue;
                }

                task_map.insert(tid, ForgetData::new(false, Duration::from_millis(100)));
                file_handler
                    .write("/dev/cpuset/fas-rs/simple/tasks", tid.to_string())
                    .unwrap();
            }
        }
    }
}
