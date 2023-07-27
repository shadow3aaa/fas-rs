/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
mod policy;

use std::{fs, path::Path};

use fas_rs_fw::prelude::*;

use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;
use log::debug;

use crate::config::CONFIG;
use policy::Policy;

pub struct CpuCommon {
    policies: Vec<Policy>,
}

impl CpuCommon {
    fn move_target_diff(&self, c: Cycles) {
        self.policies.iter().for_each(|p| p.move_target_diff(c));
    }

    fn set_target_diff(&self, c: Cycles) {
        self.policies.iter().for_each(|p| p.set_target_diff(c));
    }

    fn get_diff_move() -> Cycles {
        let mhz = CONFIG
            .get_conf("diff_move")
            .and_then_likely(|m| m.as_integer())
            .unwrap();

        Cycles::from_mhz(mhz)
    }

    pub(crate) fn always_on() -> bool {
        CONFIG
            .get_conf("always_on_gov")
            .and_then_likely(|b| b.as_bool())
            .unwrap()
    }
}

impl VirtualPerformanceController for CpuCommon {
    fn support() -> bool
    where
        Self: Sized,
    {
        Path::new("/sys/devices/system/cpu/cpufreq").exists()
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let cpufreq = fs::read_dir("/sys/devices/system/cpu/cpufreq")?;
        let mut policies: Vec<_> = cpufreq
            .into_iter()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect();

        let ignore = CONFIG
            .get_conf("ignore_little")
            .and_then_likely(|b| b.as_bool())
            .unwrap();

        policies.sort_by(|a, b| {
            let num_a: u8 = a
                .file_name()
                .and_then_likely(|f| f.to_str()?.split("policy").nth(1)?.parse().ok())
                .unwrap();
            let num_b: u8 = b
                .file_name()
                .and_then_likely(|f| f.to_str()?.split("policy").nth(1)?.parse().ok())
                .unwrap();
            num_b.cmp(&num_a)
        });

        if ignore {
            policies.truncate(2); // 保留后两个集群
        }

        let policies = policies
            .into_iter()
            .map(|path| Policy::new(&path))
            .collect();

        Ok(Self { policies })
    }

    fn limit(&self) {
        debug!("Cpu controller performance limit");
        let diff_move = Self::get_diff_move();

        self.move_target_diff(-diff_move);
    }

    fn release(&self) {
        debug!("Cpu controller performance release");
        let diff_move = Self::get_diff_move();

        self.move_target_diff(diff_move);
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        let target_diff = CONFIG
            .get_conf("default_target_diff_fas")
            .and_then_likely(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();

        self.set_target_diff(target_diff);

        self.policies.iter().for_each(Policy::resume);
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        if !Self::always_on() {
            self.policies.iter().for_each(Policy::pause);
            return Ok(());
        }

        let target_diff = CONFIG
            .get_conf("default_target_diff")
            .and_then_likely(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();

        self.set_target_diff(target_diff);

        Ok(())
    }
}
