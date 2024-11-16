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
    collections::HashMap,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicIsize},
        OnceLock,
    },
    thread,
    time::Duration,
};

use anyhow::Result;
use cpu_info::Info;
#[cfg(debug_assertions)]
use log::debug;
use log::{error, warn};

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
    policy_freq: isize,
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
            policy_freq: max_freq,
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
        self.policy_freq = self.max_freq;
        trigger_init_cpu_freq(extension);
        self.set_all_cpu_freq(self.max_freq);
    }

    pub fn init_default(&mut self, extension: &Extension) {
        self.policy_freq = self.max_freq;
        trigger_reset_cpu_freq(extension);
        self.reset_all_cpu_freq();
    }

    pub fn fas_update_freq(&mut self, control: isize) {
        self.policy_freq = self
            .policy_freq
            .saturating_add(control)
            .clamp(0, self.max_freq);

        #[cfg(debug_assertions)]
        {
            debug!("change freq: {}", control);
            debug!("policy freq: {}", self.policy_freq);
        }

        self.set_all_cpu_freq(self.policy_freq);
    }

    fn set_all_cpu_freq(&mut self, freq: isize) {
        for cpu in &self.cpu_infos {
            if let Err(e) = cpu.write_freq(freq, &mut self.file_handler) {
                error!("{:?}", e);
            }
        }
    }

    fn reset_all_cpu_freq(&mut self) {
        for cpu in &self.cpu_infos {
            if let Err(e) = cpu.reset_freq(&mut self.file_handler) {
                error!("{:?}", e);
            }
        }
    }
}
