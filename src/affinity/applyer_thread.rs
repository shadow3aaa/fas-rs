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
