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

mod cpu_info;

use std::{
    cmp,
    collections::HashMap,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicIsize, Ordering},
        OnceLock,
    },
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use cpu_info::Info;
#[cfg(debug_assertions)]
use log::debug;
use log::warn;

use crate::{
    api::{trigger_init_cpu_freq, trigger_reset_cpu_freq},
    file_handler::FileHandler,
    Extension,
};

pub static OFFSET_MAP: OnceLock<HashMap<i32, AtomicIsize>> = OnceLock::new();
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

        OFFSET_MAP.get_or_init(|| {
            cpu_infos
                .iter()
                .map(|cpu| (cpu.policy, AtomicIsize::new(0)))
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

        let fas_freqs: HashMap<_, _> = self
            .cpu_infos
            .iter_mut()
            .map(|cpu| {
                let cpu_usage = cpu
                    .cpu_usage()
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal))
                    .unwrap_or_default();
                let usage_tracking_sugg_freq =
                    (cpu.cur_freq as f32 * cpu_usage / 100.0 / 0.5) as isize; // target_usage: 50%
                (
                    cpu.policy,
                    cpu.cur_freq
                        .saturating_add(control)
                        .min(usage_tracking_sugg_freq)
                        .clamp(0, self.max_freq),
                )
            })
            .collect();

        let fas_freq_max = fas_freqs.values().max().copied().unwrap();

        #[cfg(debug_assertions)]
        debug!(
            "policy{} freq: {}",
            self.cpu_infos.last().unwrap().policy,
            fas_freq_max
        );

        let _ = self
            .cpu_infos
            .last_mut()
            .unwrap()
            .write_freq(fas_freq_max, &mut self.file_handler);

        // skip P cores
        for cpu in self.cpu_infos.iter_mut().rev().skip(1) {
            let freq = fas_freqs.get(&cpu.policy).copied().unwrap();
            let freq = freq.max(fas_freq_max * 80 / 100);

            let offset = OFFSET_MAP
                .get()
                .context("OFFSET_MAP not initialized")
                .unwrap()
                .get(&cpu.policy)
                .context("Policy offset not found")
                .unwrap()
                .load(Ordering::Acquire);
            let freq = freq.min(fas_freq_max.saturating_add(offset));

            #[cfg(debug_assertions)]
            debug!("policy{} freq: {}", cpu.policy, freq);

            let _ = cpu.write_freq(freq, &mut self.file_handler);
        }
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
