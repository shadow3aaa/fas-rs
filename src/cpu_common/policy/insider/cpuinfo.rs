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

use std::fs;

use anyhow::Result;
use smallvec::SmallVec;

use super::{Freq, Insider};

#[derive(Debug, Clone, Copy)]
pub struct CpuTimeSlice {
    total: usize,
    idle: usize,
}

impl CpuTimeSlice {
    pub const fn new(total: usize, idle: usize) -> Self {
        Self { total, idle }
    }

    pub fn calculate_usage(self, last_slice: Self) -> f64 {
        let total_slice = self.total.saturating_sub(last_slice.total);
        let idle_slice = self.idle.saturating_sub(last_slice.idle);

        (total_slice - idle_slice) as f64 / total_slice as f64
    }
}

impl Insider {
    pub fn current_freq(&self) -> Result<Freq> {
        let scaling_cur_freq = self.path.join("scaling_cur_freq");
        let freq: Freq = fs::read_to_string(scaling_cur_freq)?.trim().parse()?;
        Ok(freq)
    }

    pub fn current_usage_max(&mut self) -> Result<f64> {
        let mut max_usage: f64 = 0.0;
        let stat = fs::read_to_string("/proc/stat")?;

        for cpu_info in stat
            .lines()
            .skip(1) // skip total cpu stat
            .filter(|info| info.starts_with("cpu"))
        {
            let mut splited_info = cpu_info.split_whitespace();

            // cpu0 40908 3245 54534 154350 341 16711 2034 0 0 0
            // cpuid, user, nice, system, idle, iowait, irq, softirq, stealstolen, guest, guest_nice (11)
            //
            // cpu_usage: (total - idle) / total

            let cpu_id = splited_info
                .next()
                .and_then(|id| id[3..].parse::<i32>().ok())
                .unwrap();
            if !self.cpus.contains(&cpu_id) {
                continue;
            }

            let times: SmallVec<[usize; 9]> =
                splited_info.map(|slice| slice.parse().unwrap()).collect();
            let idle = times[3];
            let total: usize = times.into_iter().sum();

            let cpu_slice = CpuTimeSlice::new(total, idle);
            if let Some(last_cpu_slice) = self.cpu_stat.insert(cpu_id, cpu_slice) {
                let usage = cpu_slice.calculate_usage(last_cpu_slice);
                max_usage = max_usage.max(usage);
            }
        }

        Ok(max_usage)
    }
}
