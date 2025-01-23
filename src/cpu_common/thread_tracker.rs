// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicI32, Ordering},
        mpsc::Receiver,
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use perf_event::{events::Hardware, Counter};

struct ThreadTracker {
    target_threads: Vec<u32>,
    obvserve_threads: Vec<u32>,
    sched_stats: HashMap<u32, usize>,
    counter_map: HashMap<CounterKey, CounterValue>,
}

#[derive(Hash, Eq, PartialEq)]
struct CounterKey {
    cpu: i32,
    thread: u32,
}

struct CounterValue {
    counter: Counter,
    cycles_last: u64,
    timer: Instant,
}

impl CounterValue {
    fn new(counter: Counter) -> Self {
        Self {
            counter,
            cycles_last: 0,
            timer: Instant::now(),
        }
    }

    fn read(&mut self) -> u64 {
        let cycles = self.counter.read().unwrap();
        let elapsed = self.timer.elapsed();
        self.timer = Instant::now();
        let result = (cycles - self.cycles_last) as f64 * Duration::from_secs(1).as_nanos() as f64
            / elapsed.as_nanos() as f64;
        self.cycles_last = cycles;
        result as u64
    }
}

impl ThreadTracker {
    pub fn new() -> Self {
        Self {
            target_threads: Vec::new(),
            obvserve_threads: Vec::new(),
            counter_map: HashMap::new(),
            sched_stats: HashMap::new(),
        }
    }

    pub fn fetech_threads(&mut self, pid: i32) {
        self.target_threads = fs::read_dir(format!("/proc/{pid}/task"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .map(|name| name.to_str().unwrap().parse::<u32>().unwrap())
            .collect();
    }

    pub fn fetch_sched_stats(&mut self) {
        let new_sched_stats: HashMap<_, _> = self
            .target_threads
            .iter()
            .copied()
            .filter_map(|tid| {
                Some((
                    tid,
                    fs::read_to_string(Path::new("/proc").join(tid.to_string()).join("schedstat"))
                        .ok()?,
                ))
            })
            .map(|(tid, cpu_time)| {
                (
                    tid,
                    cpu_time
                        .split_whitespace()
                        .next()
                        .map(|s| s.parse::<usize>().unwrap())
                        .unwrap(),
                )
            })
            .collect();

        let mut threads_with_cpu_time: Vec<_> = new_sched_stats
            .iter()
            .filter_map(|(tid, cpu_time)| {
                Some((*tid, cpu_time.saturating_sub(*self.sched_stats.get(tid)?)))
            })
            .collect();
        self.sched_stats = new_sched_stats;

        threads_with_cpu_time.sort_by_key(|(_, cpu_time)| *cpu_time);
        threads_with_cpu_time.reverse();
        threads_with_cpu_time.truncate(5);
        self.obvserve_threads = threads_with_cpu_time
            .into_iter()
            .map(|(tid, _)| tid)
            .collect();
    }

    pub fn build_counters(&mut self) {
        self.counter_map.clear();

        for thread in self.obvserve_threads.iter().copied() {
            for cpu in 0..num_cpus::get() {
                if let Ok(mut counter) = perf_event::Builder::new(Hardware::CPU_CYCLES)
                    .one_cpu(cpu)
                    .observe_pid(thread as i32)
                    .build()
                {
                    counter.enable().unwrap();
                    self.counter_map.insert(
                        CounterKey {
                            cpu: cpu as i32,
                            thread,
                        },
                        CounterValue::new(counter),
                    );
                }
            }
        }
    }

    pub fn calculate_cycles_per_sec(&mut self, cpus: &[i32]) -> i32 {
        let mut cycles = 0;

        for thread in self.target_threads.iter().copied() {
            let mut sum = 0;

            for cpu in cpus {
                let key = CounterKey { cpu: *cpu, thread };
                if let Some(counter) = self.counter_map.get_mut(&key) {
                    let cycles_per_sec = counter.read();
                    sum += cycles_per_sec as i32;
                }
            }

            cycles = cycles.max(sum);
        }

        cycles
    }
}

pub fn thread_tracker(
    thread_map: &Arc<HashMap<Vec<i32>, AtomicI32>>,
    target_pid_receiver: &Receiver<Option<i32>>,
) {
    let mut thread_tracker = ThreadTracker::new();
    let mut target_process = None;
    let mut fetch_timer = Instant::now();

    loop {
        if let Ok(pid) = target_pid_receiver.try_recv() {
            target_process = pid;
        }

        match target_process {
            Some(pid) => {
                if fetch_timer.elapsed() >= Duration::from_secs(1) {
                    fetch_timer = Instant::now();
                    thread_tracker.fetech_threads(pid);
                    thread_tracker.fetch_sched_stats();
                    thread_tracker.build_counters();
                }

                thread::sleep(Duration::from_millis(24));

                for (cpus, cycles) in thread_map.iter() {
                    let new_cycles = thread_tracker.calculate_cycles_per_sec(cpus);
                    cycles.store(new_cycles, Ordering::Release);
                }
            }
            None => thread::sleep(Duration::from_millis(8)),
        }
    }
}
