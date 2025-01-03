// Copyright 2023-2024, shadow3 (@shadow3aaa)
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

use std::{
    cmp,
    collections::HashMap,
    fs,
    path::Path,
    sync::{atomic::AtomicBool, OnceLock},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
#[cfg(debug_assertions)]
use log::debug;
use log::warn;
use parking_lot::Mutex;

use crate::{
    api::{trigger_init_cpu_freq, trigger_reset_cpu_freq},
    file_handler::FileHandler,
    Extension,
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
        debug!("cpu infos: {:?}", cpu_infos);

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
        })
    }

    fn load_cpu_infos() -> Result<Vec<Info>> {
        let mut cpu_infos = Vec::new();

        for entry in fs::read_dir("/sys/devices/system/cpu/cpufreq")? {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(e) => {
                    warn!("Failed to read entry: {:?}", e);
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
                    warn!("Failed to read cpu info from: {:?}, reason: {:?}", path, e);
                    warn!("Retrying...");
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    pub fn init_game(&mut self, extension: &Extension) {
        trigger_init_cpu_freq(extension);
        self.set_all_cpu_freq(self.max_freq);
    }

    pub fn init_default(&mut self, extension: &Extension) {
        trigger_reset_cpu_freq(extension);
        self.reset_all_cpu_freq();
    }

    pub fn fas_update_freq(&mut self, control: isize) {
        #[cfg(debug_assertions)]
        debug!("change freq: {}", control);

        let fas_freqs = self.compute_target_frequencies(control);
        let sorted_policies = self.sort_policies_topologically();
        let fas_freqs = Self::apply_absolute_constraints(fas_freqs, &sorted_policies);
        let fas_freqs = Self::apply_relative_constraints(fas_freqs, &sorted_policies);

        for cpu in &mut self.cpu_infos {
            if let Some(freq) = fas_freqs.get(&cpu.policy).copied() {
                let _ = cpu.write_freq(freq, &mut self.file_handler);
            }
        }
    }

    fn compute_target_frequencies(&self, control: isize) -> HashMap<i32, isize> {
        let cur_freq_max = self
            .cpu_infos
            .iter()
            .map(|cpu| cpu.cur_freq)
            .max()
            .unwrap_or_default();
        self.cpu_infos
            .iter()
            .map(|cpu| {
                let cpu_usage = cpu
                    .cpu_usage()
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal))
                    .unwrap_or_default();
                let usage_tracking_sugg_freq =
                    (cpu.cur_freq as f32 * cpu_usage / 100.0 / 0.5) as isize; // target_usage: 50%
                (
                    cpu.policy,
                    cur_freq_max
                        .saturating_add(control)
                        .min(usage_tracking_sugg_freq)
                        .clamp(0, self.max_freq),
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
            .filter(|(_, &deg)| deg == 0)
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
                        debug!("policy{} rel_to {}", policy, rel_to_freq);

                        freq.clamp(
                            rel_to_freq + rel_bound.min.unwrap_or(isize::MIN),
                            rel_to_freq + rel_bound.max.unwrap_or(isize::MAX),
                        )
                    }
                    _ => freq,
                };

                #[cfg(debug_assertions)]
                debug!(
                    "policy{} freq after relative bound: {}",
                    policy, adjusted_freq
                );

                fas_freqs.insert(*policy, adjusted_freq);
            }
        }

        fas_freqs
    }

    fn set_all_cpu_freq(&mut self, freq: isize) {
        for cpu in &mut self.cpu_infos {
            let _ = cpu.write_freq(freq, &mut self.file_handler);
        }
    }

    fn reset_all_cpu_freq(&mut self) {
        for cpu in &self.cpu_infos {
            let _ = cpu.reset_freq(&mut self.file_handler);
        }
    }

    pub fn refresh_cpu_usage(&mut self) {
        for cpu in &mut self.cpu_infos {
            cpu.refresh_cpu_usage();
        }
    }

    pub fn usage_max(&mut self) -> f32 {
        self.cpu_infos
            .iter_mut()
            .map(|cpu| {
                cpu.cpu_usage()
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal))
                    .unwrap_or_default()
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal))
            .unwrap_or_default()
    }
}
