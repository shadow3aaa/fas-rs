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

mod weights;

use std::collections::HashMap;

use anyhow::Result;
use cpu_cycles_reader::{Cycles, CyclesInstant, CyclesReader};
#[cfg(debug_assertions)]
use log::debug;
pub use weights::Weights;

use super::cpu_info::Info;

#[derive(Debug)]
pub struct WeightedCalculator {
    map: HashMap<i32, CyclesInstant>,
    reader: CyclesReader,
    cache: Weights,
}

impl WeightedCalculator {
    pub fn new(policys: &[Info]) -> Result<Self> {
        Ok(Self {
            map: HashMap::new(),
            reader: CyclesReader::new(None)?,
            cache: Weights::new(policys),
        })
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn update(&mut self) -> &Weights {
        self.calculate_weights();
        &self.cache
    }

    fn calculate_weights(&mut self) {
        let num_cpus = num_cpus::get();

        let cycles_per_cpu: HashMap<_, _> = (0..num_cpus)
            .map(|cpu| {
                let now = self.reader.instant(cpu as i32).unwrap();
                let last = self.map.insert(cpu as i32, now);
                (cpu as i32, last.map_or(Cycles::ZERO, |last| now - last))
            })
            .collect();

        #[cfg(debug_assertions)]
        debug!("cycles per cpu: {cycles_per_cpu:#?}");

        let mut cycles_per_policy_max = HashMap::new();
        for cpus in self.cache.map.keys() {
            let cycles = cycles_per_cpu
                .iter()
                .filter(|(cpu, _)| cpus.contains(cpu))
                .map(|(_, cycles)| cycles)
                .max()
                .copied()
                .unwrap_or(Cycles::ZERO);
            cycles_per_policy_max.insert(cpus.clone(), cycles);
        }

        let cycles_sum: Cycles = cycles_per_policy_max.values().copied().sum();
        for (cpus, weight) in &mut self.cache.map {
            *weight =
                cycles_per_policy_max.get(cpus).unwrap().as_hz() as f64 / cycles_sum.as_hz() as f64;
        }
    }
}
