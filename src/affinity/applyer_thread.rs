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

use std::{collections::HashSet, sync::mpsc::Receiver};

use flower::flow_web::AnalyzeData;
use rustix::process::{sched_setaffinity, CpuSet, Pid};

#[derive(Debug)]
pub struct Data {
    pub datas: Vec<AnalyzeData>,
    pub threads: Vec<u32>,
}

pub fn affinity_applyer(rx: &Receiver<Data>, cpuset_big: CpuSet, cpuset_middle: CpuSet) {
    loop {
        if let Ok(data) = rx.recv() {
            let critical_threads: HashSet<_> = data.datas.iter().map(|data| data.tid).collect();
            for critical_thread in &critical_threads {
                let _ = sched_setaffinity(
                    Some(unsafe { Pid::from_raw_unchecked(*critical_thread as i32) }),
                    &cpuset_big,
                );
            }

            for tid in data.threads {
                if critical_threads.contains(&tid) {
                    continue;
                }

                let _ = sched_setaffinity(
                    Some(unsafe { Pid::from_raw_unchecked(tid as i32) }),
                    &cpuset_middle,
                );
            }
        }
    }
}
