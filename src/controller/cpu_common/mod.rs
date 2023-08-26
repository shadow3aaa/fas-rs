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
//! 通用cpu控制器
mod policy;

use std::{ffi::OsStr, fs};

use fas_rs_fw::prelude::*;

use anyhow::Result;
use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;
use log::debug;

use policy::Policy;

use crate::error::Error;

pub struct CpuCommon {
    policies: Vec<Policy>,
}

impl CpuCommon {
    pub fn new(config: &Config) -> Result<Self>
    where
        Self: Sized,
    {
        let cpufreq = fs::read_dir("/sys/devices/system/cpu/cpufreq")?;
        let mut policies: Vec<_> = cpufreq
            .into_iter()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.is_dir()
                    && p.file_name()
                        .and_then(OsStr::to_str)
                        .unwrap()
                        .contains("policy")
            })
            .collect();

        let ignore = config
            .get_conf("ignore_little")?
            .as_bool()
            .ok_or(Error::ParseConfig)?;

        policies.sort_by(|a, b| {
            let num_a: u8 = a
                .file_name()
                .and_then_likely(|f| parse_policy(f.to_str()?))
                .unwrap_or_default();
            let num_b: u8 = b
                .file_name()
                .and_then_likely(|f| parse_policy(f.to_str()?))
                .unwrap_or_default();
            num_b.cmp(&num_a)
        });

        if ignore {
            policies.truncate(2); // 保留后两个集群
        }

        Node::create_node("max_freq_per", "100").unwrap();

        let policies = policies
            .into_iter()
            .map(|path| Policy::new(&path, config).unwrap())
            .collect();

        Ok(Self { policies })
    }

    fn move_target_diff(&self, c: Cycles) {
        self.policies.iter().for_each(|p| p.move_target_diff(c));
    }

    fn set_target_diff(&self, c: Cycles) {
        self.policies.iter().for_each(|p| p.set_target_diff(c));
    }

    fn get_diff_move(config: &Config) -> Result<Cycles> {
        let mhz = config
            .get_conf("diff_move")?
            .as_integer()
            .ok_or(Error::ParseConfig)?;

        Ok(Cycles::from_mhz(mhz))
    }
}

impl PerformanceController for CpuCommon {
    fn perf(&self, l: u32, config: &Config) {
        let diff_move = Self::get_diff_move(config).unwrap();
        let diff_move = if l > 0 {
            diff_move * i64::from(l)
        } else {
            -diff_move
        };

        self.move_target_diff(diff_move);

        debug!("Move margin: {diff_move}");
    }

    fn init_game(&self, config: &Config) -> Result<(), fas_rs_fw::Error> {
        let target_diff = config
            .get_conf("default_target_diff_fas")?
            .as_integer()
            .ok_or(fas_rs_fw::Error::Other("Failed to parse config"))?;
        let target_diff = Cycles::from_mhz(target_diff);

        self.set_target_diff(target_diff);

        Ok(())
    }

    fn init_default(&self, config: &Config) -> Result<(), fas_rs_fw::Error> {
        let target_diff = config
            .get_conf("default_target_diff")?
            .as_integer()
            .ok_or(fas_rs_fw::Error::Other("Failed to parse config"))?;
        let target_diff = Cycles::from_mhz(target_diff);

        self.set_target_diff(target_diff);

        Ok(())
    }
}

pub fn parse_policy(p: &str) -> Option<u8> {
    p.replace("policy", "").trim().parse().ok()
}
