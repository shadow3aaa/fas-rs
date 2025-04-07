// Copyright 2023-2025, dependabot[bot], shadow3, shadow3aaa
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

mod cpu_info;
pub mod extra_policy;
mod process_monitor;

use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{OnceLock, atomic::AtomicBool},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
#[cfg(debug_assertions)]
use log::debug;
use log::warn;
use nix::{
    sched::{CpuSet, sched_getaffinity},
    unistd::Pid,
};
use parking_lot::Mutex;
use process_monitor::ProcessMonitor;

use crate::{
    Extension,
    api::{trigger_init_cpu_freq, trigger_reset_cpu_freq},
    file_handler::FileHandler,
};
use cpu_info::Info;
use extra_policy::ExtraPolicy;

pub static EXTRA_POLICY_MAP: OnceLock<HashMap<i32, Mutex<ExtraPolicy>>> = OnceLock::new();
pub static IGNORE_MAP: OnceLock<HashMap<i32, AtomicBool>> = OnceLock::new();

#[derive(Debug)]
pub struct Controller {
    max_freq: isize,
    cpu_infos: Vec<Info>,
    file_handler: FileHandler,
    process_monitor: ProcessMonitor,
    util_max: Option<f64>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        let mut cpu_infos = Self::load_cpu_infos()?;
        cpu_infos.sort_by_key(|cpu| cpu.policy);

        EXTRA_POLICY_MAP.get_or_init(|| {
            cpu_infos
                .iter()
                .map(|cpu| (cpu.policy, Mutex::new(ExtraPolicy::None)))
                .collect()
        });
        IGNORE_MAP.get_or_init(|| {
            cpu_infos
                .iter()
                .map(|cpu| (cpu.policy, AtomicBool::new(false)))
                .collect()
        });

        #[cfg(debug_assertions)]
        debug!("cpu infos: {cpu_infos:?}");

        let max_freq = cpu_infos
            .iter()
            .flat_map(|info| info.freqs.iter())
            .max()
            .copied()
            .unwrap_or(0);

        Ok(Self {
            max_freq,
            cpu_infos,
            file_handler: FileHandler::new(),
            process_monitor: ProcessMonitor::new(),
            util_max: None,
        })
    }

    fn load_cpu_infos() -> Result<Vec<Info>> {
        let mut cpu_infos = Vec::new();

        for entry in fs::read_dir("/sys/devices/system/cpu/cpufreq")? {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(e) => {
                    warn!("Failed to read entry: {e:?}");
                    continue;
                }
            };

            if !path.is_dir() {
                continue;
            }

            let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if !filename.starts_with("policy") {
                continue;
            }

            cpu_infos.push(Self::retry_load_info(&path));
        }

        Ok(cpu_infos)
    }

    fn retry_load_info(path: &Path) -> Info {
        loop {
            match Info::new(path) {
                Ok(info) => return info,
                Err(e) => {
                    warn!("Failed to read cpu info from: {path:?}, reason: {e:?}");
                    warn!("Retrying...");
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    pub fn init_game(&mut self, pid: i32, extension: &Extension) {
        trigger_init_cpu_freq(extension);
        self.reset_all_cpu_freq();
        self.process_monitor.set_pid(Some(pid));
        self.util_max = None;
    }

    pub fn init_default(&mut self, extension: &Extension) {
        trigger_reset_cpu_freq(extension);
        self.reset_all_cpu_freq();
        self.process_monitor.set_pid(None);
        self.util_max = None;
    }

    pub fn fas_update_freq(&mut self, control: isize, is_janked: bool) {
        #[cfg(debug_assertions)]
        debug!("change freq: {control}");

        let fas_freqs = self.compute_target_frequencies(control, is_janked);
        let sorted_policies = self.sort_policies_topologically();
        let fas_freqs = Self::apply_absolute_constraints(fas_freqs, &sorted_policies);
        let fas_freqs = Self::apply_relative_constraints(fas_freqs, &sorted_policies);
        let top_used_cores = self.top_used_cores().unwrap_or_else(|| {
            let mut all_cores = CpuSet::new();
            for core in 0..num_cpus::get() {
                all_cores.set(core).unwrap();
            }
            all_cores
        });

        if no_extra_policy() {
            let fas_freq_max = fas_freqs.values().max().copied().unwrap();
            for cpu in &mut self.cpu_infos {
                if let Some(freq) = fas_freqs.get(&cpu.policy).copied() {
                    let freq = freq.clamp(
                        fas_freq_max.saturating_sub(100_000),
                        fas_freq_max.saturating_add(100_000),
                    );
                    let _ = cpu.write_freq(top_used_cores, freq, &mut self.file_handler);
                }
            }
        } else {
            for cpu in &mut self.cpu_infos {
                if let Some(freq) = fas_freqs.get(&cpu.policy).copied() {
                    let _ = cpu.write_freq(top_used_cores, freq, &mut self.file_handler);
                }
            }
        }
    }

    fn update_util_max(&mut self) {
        if let Some(util_max) = self.process_monitor.update() {
            self.util_max = Some(util_max);
        }
    }

    fn compute_target_frequencies(
        &mut self,
        control: isize,
        is_janked: bool,
    ) -> HashMap<i32, isize> {
        let cur_fas_freq_max = self
            .cpu_infos
            .iter()
            .map(|cpu| cpu.cur_fas_freq)
            .max()
            .unwrap_or_default();
        let cur_freq_max = self
            .cpu_infos
            .iter()
            .map(cpu_info::Info::read_freq)
            .max()
            .unwrap_or_default();

        if is_janked {
            self.util_max = None;
        } else {
            self.update_util_max();
        }

        self.cpu_infos
            .iter()
            .map(|cpu| {
                (
                    cpu.policy,
                    if is_janked || self.util_max.is_none() {
                        cur_fas_freq_max
                            .saturating_add(control)
                            .clamp(0, self.max_freq)
                    } else {
                        let util_tracking_sugg_freq =
                            (cur_freq_max as f64 * self.util_max.unwrap() / 0.5) as isize; // min_util: 50%
                        #[cfg(debug_assertions)]
                        debug!(
                            "util: {}, cur_freq_max: {}, util_tracking_sugg_freq: {}",
                            self.util_max.unwrap(),
                            cur_freq_max,
                            util_tracking_sugg_freq
                        );
                        cur_fas_freq_max
                            .saturating_add(control)
                            .min(util_tracking_sugg_freq)
                            .clamp(0, self.max_freq)
                    },
                )
            })
            .collect()
    }

    fn sort_policies_topologically(&self) -> Vec<i32> {
        let mut graph: HashMap<_, Vec<_>> = HashMap::new();
        let mut indegree: HashMap<_, _> = HashMap::new();

        for cpu in &self.cpu_infos {
            let policy = cpu.policy;

            if let ExtraPolicy::RelRangeBound(ref rel_bound) = *EXTRA_POLICY_MAP
                .get()
                .context("EXTRA_POLICY_MAP not initialized")
                .unwrap()
                .get(&policy)
                .context("CPU Policy not found")
                .unwrap()
                .lock()
            {
                graph.entry(rel_bound.rel_to).or_default().push(policy);
                *indegree.entry(policy).or_insert(0) += 1;
            }

            indegree.entry(policy).or_insert(0);
        }

        let mut queue: Vec<_> = indegree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&policy, _)| policy)
            .collect();
        let mut sorted_policies = Vec::new();

        while let Some(policy) = queue.pop() {
            sorted_policies.push(policy);
            if let Some(dependents) = graph.get(&policy) {
                for &dependent in dependents {
                    if let Some(deg) = indegree.get_mut(&dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(dependent);
                        }
                    }
                }
            }
        }

        assert!(
            (sorted_policies.len() >= indegree.len()),
            "Circular dependency detected in CPU policies"
        );

        sorted_policies
    }

    fn top_used_cores(&self) -> Option<CpuSet> {
        let top_threads_cpu_sets: Vec<_> = self
            .process_monitor
            .top_threads()
            .filter_map(|tid| sched_getaffinity(Pid::from_raw(tid)).ok())
            .collect();

        let mut counts = HashMap::new();
        for cpu_set in top_threads_cpu_sets.iter().copied() {
            *counts.entry(cpu_set).or_insert(0) += 1;
        }

        if counts.len() <= 1 {
            let mut top_used_cores = CpuSet::new();
            for cpuset in top_threads_cpu_sets {
                for core in 0..num_cpus::get() {
                    if cpuset.is_set(core).unwrap() {
                        top_used_cores.set(core).unwrap();
                    }
                }
            }

            return Some(top_used_cores);
        }

        let (mode, _) = counts.into_iter().max_by_key(|&(_num, count)| count)?;

        let mut top_used_cores = CpuSet::new();
        for cpuset in top_threads_cpu_sets
            .into_iter()
            .filter(|cpu_set| *cpu_set != mode)
        {
            for core in 0..num_cpus::get() {
                if cpuset.is_set(core).unwrap() {
                    top_used_cores.set(core).unwrap();
                }
            }
        }

        Some(top_used_cores)
    }

    fn apply_absolute_constraints(
        mut fas_freqs: HashMap<i32, isize>,
        sorted_policies: &[i32],
    ) -> HashMap<i32, isize> {
        for policy in sorted_policies {
            if let Some(freq) = fas_freqs.get(policy).copied() {
                if let ExtraPolicy::AbsRangeBound(ref abs_bound) = *EXTRA_POLICY_MAP
                    .get()
                    .context("EXTRA_POLICY_MAP not initialized")
                    .unwrap()
                    .get(policy)
                    .context("CPU Policy not found")
                    .unwrap()
                    .lock()
                {
                    let clamped_freq = freq.clamp(
                        abs_bound.min.unwrap_or(0),
                        abs_bound.max.unwrap_or(isize::MAX),
                    );
                    fas_freqs.insert(*policy, clamped_freq);
                }
            }
        }

        fas_freqs
    }

    fn apply_relative_constraints(
        mut fas_freqs: HashMap<i32, isize>,
        sorted_policies: &[i32],
    ) -> HashMap<i32, isize> {
        for policy in sorted_policies {
            if let Some(freq) = fas_freqs.get(policy).copied() {
                let adjusted_freq = match *EXTRA_POLICY_MAP
                    .get()
                    .context("EXTRA_POLICY_MAP not initialized")
                    .unwrap()
                    .get(policy)
                    .context("CPU Policy not found")
                    .unwrap()
                    .lock()
                {
                    ExtraPolicy::RelRangeBound(ref rel_bound) => {
                        let rel_to_freq = fas_freqs.get(&rel_bound.rel_to).copied().unwrap_or(0);

                        #[cfg(debug_assertions)]
                        debug!("policy{policy} rel_to {rel_to_freq}");

                        freq.clamp(
                            rel_to_freq + rel_bound.min.unwrap_or(isize::MIN),
                            rel_to_freq + rel_bound.max.unwrap_or(isize::MAX),
                        )
                    }
                    _ => freq,
                };

                #[cfg(debug_assertions)]
                debug!(
                    "policy{policy} freq after relative bound: {adjusted_freq}"
                );

                fas_freqs.insert(*policy, adjusted_freq);
            }
        }

        fas_freqs
    }

    fn reset_all_cpu_freq(&mut self) {
        for cpu in &mut self.cpu_infos {
            let _ = cpu.reset(&mut self.file_handler);
        }
    }

    pub fn util_max(&self) -> f64 {
        self.util_max.unwrap_or_default()
    }
}

fn no_extra_policy() -> bool {
    EXTRA_POLICY_MAP
        .get()
        .context("EXTRA_POLICY_MAP not initialized")
        .unwrap()
        .values()
        .all(|policy| *policy.lock() == ExtraPolicy::None)
}
