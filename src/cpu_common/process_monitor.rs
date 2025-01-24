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
    cmp,
    collections::{hash_map::Entry, HashMap},
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender, SyncSender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use libc::{sysconf, _SC_CLK_TCK};

#[derive(Debug, Clone, Copy)]
struct UsageTracker {
    pid: i32,
    tid: i32,
    last_cputime: u64,
    read_timer: Instant,
}

impl UsageTracker {
    fn new(pid: i32, tid: i32) -> Result<Self> {
        Ok(Self {
            pid,
            tid,
            last_cputime: get_thread_cpu_time(pid, tid)?,
            read_timer: Instant::now(),
        })
    }

    fn try_calculate(&mut self) -> Result<f64> {
        let tick_per_sec = unsafe { sysconf(_SC_CLK_TCK) };
        let new_cputime = get_thread_cpu_time(self.pid, self.tid)?;
        let elapsed_ticks = self.read_timer.elapsed().as_secs_f64() * tick_per_sec as f64;
        self.read_timer = Instant::now();
        let cputime_slice = new_cputime - self.last_cputime;
        self.last_cputime = new_cputime;
        Ok(cputime_slice as f64 / elapsed_ticks)
    }
}

#[derive(Debug)]
pub struct ProcessMonitor {
    stop: Arc<AtomicBool>,
    sender: SyncSender<Option<i32>>,
    util_max: Receiver<f64>,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel(0);
        let stop = Arc::new(AtomicBool::new(false));
        let (util_max_sender, util_max) = mpsc::channel();

        {
            let stop = stop.clone();

            thread::Builder::new()
                .name("ProcessMonitor".to_string())
                .spawn(move || {
                    monitor_thread(&stop, &receiver, &util_max_sender);
                })
                .unwrap();
        }

        Self {
            stop,
            sender,
            util_max,
        }
    }

    pub fn set_pid(&self, pid: Option<i32>) {
        self.sender.send(pid).unwrap();
    }

    fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }

    pub fn update_util_max(&self) -> Option<f64> {
        self.util_max.try_iter().last()
    }
}

impl Drop for ProcessMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

fn monitor_thread(
    stop: &Arc<AtomicBool>,
    receiver: &Receiver<Option<i32>>,
    util_max: &Sender<f64>,
) {
    let mut current_pid = None;
    let mut last_full_update = Instant::now();
    let mut all_trackers = HashMap::new();
    let mut top_trackers = HashMap::new();

    while !stop.load(Ordering::Acquire) {
        if let Ok(pid) = receiver.try_recv() {
            current_pid = pid;
            all_trackers.clear();
            top_trackers.clear();
        }

        if let Some(pid) = current_pid {
            if last_full_update.elapsed() >= Duration::from_secs(1) {
                if let Ok(threads) = get_thread_ids(pid) {
                    all_trackers = threads
                        .iter()
                        .copied()
                        .filter_map(|tid| {
                            Some((
                                tid,
                                match all_trackers.entry(tid) {
                                    Entry::Occupied(o) => o.remove(),
                                    Entry::Vacant(_) => UsageTracker::new(pid, tid).ok()?,
                                },
                            ))
                        })
                        .collect();
                    let mut top_threads: Vec<_> = all_trackers
                        .iter()
                        .filter_map(|(tid, tracker)| {
                            Some((*tid, tracker.clone().try_calculate().ok()?))
                        })
                        .collect();
                    top_threads
                        .sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(cmp::Ordering::Equal));
                    top_threads.truncate(5);
                    top_trackers = top_threads
                        .into_iter()
                        .filter_map(|(tid, _)| match top_trackers.entry(tid) {
                            Entry::Occupied(o) => Some((tid, o.remove())),
                            Entry::Vacant(_) => Some((tid, UsageTracker::new(pid, tid).ok()?)),
                        })
                        .collect();
                    last_full_update = Instant::now();
                }
            }

            let mut max_usage: f64 = 0.0;
            for tracker in top_trackers.values_mut() {
                if let Ok(usage) = tracker.try_calculate() {
                    max_usage = max_usage.max(usage);
                }
            }

            util_max.send(max_usage).unwrap();
        }

        thread::sleep(Duration::from_millis(300));
    }
}

fn get_thread_ids(pid: i32) -> Result<Vec<i32>> {
    let proc_path = format!("/proc/{pid}/task");
    Ok(fs::read_dir(proc_path)?
        .filter_map(|entry| {
            entry
                .ok()
                .and_then(|e| e.file_name().to_string_lossy().parse::<i32>().ok())
        })
        .collect())
}

fn get_thread_cpu_time(pid: i32, tid: i32) -> Result<u64> {
    let stat_path = format!("/proc/{pid}/task/{tid}/stat");
    let stat_content = fs::read_to_string(stat_path)?;
    let parts: Vec<&str> = stat_content.split_whitespace().collect();
    let utime = parts[13].parse::<u64>().unwrap_or(0);
    let stime = parts[14].parse::<u64>().unwrap_or(0);
    Ok(utime + stime)
}
