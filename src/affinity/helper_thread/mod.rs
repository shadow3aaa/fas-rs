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

mod top_threads;

use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    sync::mpsc::Receiver,
    thread,
    time::{Duration, Instant},
};

use flower::{list_threads, Flower};
use likely_stable::unlikely;
use log::info;

use super::applyer::{AffinityApplyer, Data};
use top_threads::TopThreads;

pub enum Command {
    Attach(i32),
    Detach,
    StartAnalyze,
}

struct Context {
    pub flower: Flower,
    pub pid: u32,
    pub instant: Instant,
    pub top_threads: TopThreads,
    pub threads: Vec<u32>,
}

pub fn affinity_helper(receiver: &Receiver<Command>) {
    let mut context = None;

    let cpus = Path::new("/")
        .join("sys")
        .join("devices")
        .join("system")
        .join("cpu");
    let num_cpus = num_cpus::get();
    let mut capacity_map = BTreeMap::new();

    for cpu in 0..num_cpus {
        // example: /sys/devices/system/cpu0
        let path = format!("{}/cpu{cpu}/cpu_capacity", cpus.to_str().unwrap());
        let capacity: usize = fs::read_to_string(path).unwrap().trim().parse().unwrap();
        capacity_map
            .entry(capacity)
            .or_insert_with(Vec::new)
            .push(cpu);
    }

    let mut cpu_sets = capacity_map.into_values().rev().take(2);
    let cpuset_big = cpu_sets.next().unwrap();
    let cpuset_middle = cpu_sets.next().unwrap();

    info!("cpuset big: {cpuset_big:#?}");
    info!("cpuset middle: {cpuset_middle:#?}");

    let mut applyer = AffinityApplyer::new(&cpuset_big, &cpuset_middle);

    loop {
        if let Ok(event) = receiver.try_recv() {
            match event {
                Command::Attach(target_pid) => {
                    let threads = list_threads(target_pid as u32).unwrap();
                    context = Some(Context {
                        flower: Flower::new(target_pid as u32).unwrap(),
                        pid: target_pid as u32,
                        instant: Instant::now(),
                        top_threads: TopThreads::new(&threads),
                        threads,
                    });
                }
                Command::Detach => {
                    let _ = applyer.detach();
                    context = None;
                }
                Command::StartAnalyze => {
                    if let Some(context) = &mut context {
                        context.flower.clear();
                    }
                }
            }
        }

        if let Some(context) = &mut context {
            if unlikely(context.instant.elapsed() > Duration::from_secs(1)) {
                context.instant = Instant::now();
                context.threads = list_threads(context.pid).unwrap();
                let mut top_threads = context.top_threads.result();
                top_threads.truncate(5);
                context.flower.set_top_threads(Some(top_threads));
                context.top_threads = TopThreads::new(&context.threads);
            }

            if !context.flower.update(Some(Duration::from_secs(1))) {
                continue;
            }

            if let Some(datas) = context.flower.analyze() {
                let data = Data {
                    datas,
                    threads: context.threads.clone(),
                };
                applyer.apply(data);
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
