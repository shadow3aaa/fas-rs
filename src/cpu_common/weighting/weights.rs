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

use std::collections::HashMap;

pub struct Weights {
    pub map: HashMap<i32, f64>,
}

impl Weights {
    pub fn weight(&self, cpus: &Vec<i32>) -> Option<f64> {
        if self.map.is_empty() {
            return None;
        }

        let mut weight = 1.0;
        for cpu in cpus {
            let partial_weight = *self.map.get(cpu)?;
            if partial_weight.is_normal() {
                weight += partial_weight;
            }
        }

        let weight = weight.min(1.5);

        Some(weight)
    }
}
