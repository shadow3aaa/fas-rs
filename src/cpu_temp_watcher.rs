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

use std::{fs, path::PathBuf};

use anyhow::Result;

pub struct CpuTempWatcher {
    nodes: Vec<PathBuf>,
}

impl CpuTempWatcher {
    pub fn new() -> Result<Self> {
        let mut nodes = Vec::new();
        for device in fs::read_dir("/sys/devices/virtual/thermal")? {
            let device = device?;
            let device_type = device.path().join("type");
            let Ok(device_type) = fs::read_to_string(device_type) else {
                continue;
            };
            if device_type.contains("cpu-")
                || device_type.contains("soc_max")
                || device_type.contains("mtktscpu")
            {
                nodes.push(device.path().join("temp"));
            }
        }

        Ok(Self { nodes })
    }

    pub fn temp(&mut self) -> u64 {
        self.nodes
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .map(|temp| temp.trim().parse::<u64>().unwrap_or_default())
            .max()
            .unwrap()
    }
}
