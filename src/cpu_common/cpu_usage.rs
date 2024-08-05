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

use std::{collections::HashMap, fs};

#[derive(Debug)]
pub struct UsageReader {
    usage_map: HashMap<i32, f64>,
    cputimes: HashMap<i32, CpuTime>,
}

impl UsageReader {
    pub fn new() -> Self {
        Self {
            usage_map: HashMap::new(),
            cputimes: HashMap::new(),
        }
    }

    pub fn update(&mut self) -> &HashMap<i32, f64> {
        let stat = fs::read_to_string("/proc/stat").unwrap();

        for (cpu, cputime_raw) in stat
            .lines()
            .filter(|l| l.starts_with("cpu"))
            .skip(1)
            .enumerate()
        {
            let cpu = cpu as i32;
            let cputime = CpuTime::new(cputime_raw);

            if let Some(last_cputime) = self.cputimes.get(&cpu) {
                let idle_slice = cputime.idle - last_cputime.idle;
                let total_slice = cputime.total - last_cputime.total;
                let usage = (total_slice - idle_slice) as f64 / total_slice as f64;

                self.usage_map.insert(cpu, usage);
            }

            self.cputimes.insert(cpu, cputime);
        }

        &self.usage_map
    }
}

#[derive(Debug)]
struct CpuTime {
    pub total: isize,
    pub idle: isize,
}

impl CpuTime {
    pub fn new(line: &str) -> Self {
        let total = line
            .split_whitespace()
            .skip(1)
            .map(|t| t.parse::<isize>().unwrap())
            .sum();
        let idle = line
            .split_whitespace()
            .skip(1)
            .nth(3)
            .map(|t| t.parse::<isize>().unwrap())
            .unwrap();

        Self { total, idle }
    }
}
