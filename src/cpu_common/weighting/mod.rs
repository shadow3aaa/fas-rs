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

mod task;
mod weights;

use std::{
    collections::HashMap,
    fs,
    time::{Duration, Instant},
};

use anyhow::Result;
use cpu_cycles_reader::Cycles;
use libc::pid_t;
#[cfg(debug_assertions)]
use log::debug;
use task::TaskMeta;
pub use weights::Weights;

use super::cpu_info::Info;

#[derive(Debug)]
pub struct WeightedCalculator {
    map: HashMap<i32, TaskMeta>,
    cpu_times_long: HashMap<i32, u64>,
    cpu_times_short: HashMap<i32, u64>,
    short_timer: Instant,
    long_timer: Instant,
    cache: Weights,
}

impl WeightedCalculator {
    pub fn new(policys: &Vec<Info>) -> Self {
        Self {
            map: HashMap::new(),
            cpu_times_long: HashMap::new(),
            cpu_times_short: HashMap::new(),
            short_timer: Instant::now(),
            long_timer: Instant::now(),
            cache: Weights::new(policys),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.cpu_times_short.clear();
        self.cpu_times_long.clear();
        self.short_timer = Instant::now();
        self.long_timer = Instant::now();
    }

    pub fn update(&mut self, process: pid_t) -> Result<&Weights> {
        /* if self.short_timer.elapsed() >= Duration::from_secs(1) {
            self.short_timer = Instant::now();

        } */

        self.update_top_tasks(process)?;
        self.calculate_weights()?;

        Ok(&self.cache)
    }

    fn calculate_weights(&mut self) -> Result<()> {
        let num_cpus = num_cpus::get() as i32;
        let mut cycles_per_cpu: HashMap<_, _> =
            (0..num_cpus).map(|cpu| (cpu, Cycles::ZERO)).collect();

        for meta in self.map.values_mut() {
            for cpu in 0..num_cpus {
                let now = meta.cycles_reader.instant(cpu)?;
                let last = meta.cycles_trace.get_mut(cpu as usize).unwrap();
                *cycles_per_cpu.get_mut(&cpu).unwrap() += now - *last;
                *last = now;
            }
        }

        #[cfg(debug_assertions)]
        debug!("cycles per cpu: {cycles_per_cpu:#?}");

        let cycles_per_policy_max: HashMap<_, _> = self
            .cache
            .map
            .keys()
            .map(|cpus| {
                (
                    cpus.clone(),
                    *cycles_per_cpu
                        .iter()
                        .filter(|(cpu, _)| cpus.contains(cpu))
                        .map(|(_, n)| n)
                        .max()
                        .unwrap(),
                )
            })
            .collect();
        let cycles_sum: Cycles = cycles_per_policy_max.values().copied().sum();
        for (cpus, weight) in &mut self.cache.map {
            *weight =
                cycles_per_policy_max.get(cpus).unwrap().as_hz() as f64 / cycles_sum.as_hz() as f64;
        }

        Ok(())
    }

    fn update_top_tasks(&mut self, process: pid_t) -> Result<()> {
        if self.long_timer.elapsed() <= Duration::from_secs(1) {
            return Ok(());
        }

        self.long_timer = Instant::now();

        let cpu_times: HashMap<_, _> = fs::read_dir(format!("/proc/{process}/task"))?
            .map(|e| e.unwrap().path())
            .filter_map(|p| {
                Some((
                    p.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse::<pid_t>()
                        .ok()?,
                    fs::read_to_string(p.join("schedstat")).ok()?,
                ))
            })
            .map(|(tid, stat)| {
                (
                    tid,
                    stat.split_whitespace()
                        .next()
                        .map(|t| t.parse::<u64>().unwrap())
                        .unwrap(),
                )
            })
            .collect();

        let mut cpu_slices: Vec<_> = cpu_times
            .iter()
            .map(|(tid, cputime)| {
                (
                    *tid,
                    self.cpu_times_long.get(tid).map_or(0, |t| cputime - t),
                )
            })
            .collect();
        cpu_slices.sort_by_key(|(_, slice)| *slice);
        cpu_slices.reverse();
        cpu_slices.truncate(5);

        self.cpu_times_long = cpu_times;

        let cpu_slices: HashMap<_, _> = cpu_slices.into_iter().collect();

        #[cfg(debug_assertions)]
        debug!("cpu_slices: {cpu_slices:?}");

        self.map.retain(|t, _| cpu_slices.contains_key(t));

        let num_cpus = num_cpus::get();
        for (task, _) in cpu_slices {
            self.map
                .entry(task)
                .or_insert(TaskMeta::new(task, num_cpus)?);
        }

        Ok(())
    }
}
