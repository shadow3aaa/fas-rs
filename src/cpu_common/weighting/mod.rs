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
    collections::{hash_map, HashMap},
    fs,
    path::Path,
    time::{Duration, Instant},
};

use anyhow::Result;
use cpu_instructions_reader::InstructionNumber;
use libc::pid_t;
#[cfg(debug_assertions)]
use log::debug;
use task::TaskMeta;
pub use weights::Weights;

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
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            cpu_times_long: HashMap::new(),
            cpu_times_short: HashMap::new(),
            short_timer: Instant::now(),
            long_timer: Instant::now(),
            cache: Weights::new(),
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
        if self.short_timer.elapsed() <= Duration::from_secs(1) {
            return Ok(&self.cache);
        }

        self.short_timer = Instant::now();

        self.update_top_tasks(process)?;
        self.calculate_weights()?;

        Ok(&self.cache)
    }

    fn calculate_weights(&mut self) -> Result<()> {
        for meta in self.map.values() {
            let num_cpus = num_cpus::get();
            let mut instructions_instants = Vec::new();

            for cpu in 0..num_cpus {
                instructions_instants.push(meta.instructions_reader.instant(cpu as i32)?);
            }

            let instructions: Vec<_> = instructions_instants
                .iter()
                .zip(meta.instructions_trace.iter())
                .map(|(now, last)| *now - *last)
                .collect();
            let instructions_sum: InstructionNumber = instructions.iter().copied().sum();

            for (cpu, instructions) in instructions.iter().enumerate() {
                let cpu_weight = instructions.as_raw() as f64 / instructions_sum.as_raw() as f64;
                let final_weight = cpu_weight * meta.weight;
                match self.cache.map.entry(cpu as i32) {
                    hash_map::Entry::Occupied(mut o) => {
                        *o.get_mut() += final_weight;
                    }
                    hash_map::Entry::Vacant(v) => {
                        v.insert(final_weight);
                    }
                }
            }
        }

        Ok(())
    }

    fn update_cpu_times(&mut self, process: pid_t) {
        self.map.retain(|task, _| {
            Path::new(&format!("/proc/{process}/task/{task}/schedstat")).exists()
        });

        let new_cpu_times: HashMap<_, _> = self
            .map
            .keys()
            .filter_map(|task| {
                Some((
                    task,
                    fs::read_to_string(format!("/proc/{process}/task/{task}/schedstat")).ok()?,
                ))
            })
            .map(|(task, stat)| {
                (
                    *task,
                    stat.split_whitespace()
                        .next()
                        .map(|t| t.parse::<u64>().unwrap())
                        .unwrap(),
                )
            })
            .collect();

        let cpu_slices: HashMap<_, _> = new_cpu_times
            .iter()
            .map(|(tid, cputime)| {
                (
                    *tid,
                    self.cpu_times_short
                        .get(tid)
                        .map_or(0, |last| cputime - last),
                )
            })
            .collect();

        #[cfg(debug_assertions)]
        debug!("cpu_slices: {cpu_slices:?}");

        self.cpu_times_short = new_cpu_times;

        let total_time: u64 = cpu_slices.values().sum();
        for (task, time) in cpu_slices {
            let weight = time as f64 / total_time as f64;
            self.map.get_mut(&task).unwrap().weight = weight;
        }
    }

    fn update_top_tasks(&mut self, process: pid_t) -> Result<()> {
        if self.long_timer.elapsed() <= Duration::from_secs(5) {
            self.update_cpu_times(process);
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
        let total_time: u64 = cpu_slices.values().sum();
        for (task, time) in cpu_slices {
            let weight = time as f64 / total_time as f64;
            self.map
                .entry(task)
                .or_insert(TaskMeta::new(task, num_cpus)?)
                .weight = weight;
        }

        Ok(())
    }
}
