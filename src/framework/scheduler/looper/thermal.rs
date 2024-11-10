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

use crate::{framework::config::TemperatureThreshold, Config, Mode};

pub struct Thermal {
    target_fps_offset: f64,
    core_temperature: u64,
    nodes: Vec<PathBuf>,
}

impl Thermal {
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

        Ok(Self {
            target_fps_offset: 0.0,
            core_temperature: 0,
            nodes,
        })
    }

    pub fn target_fps_offset(&mut self, config: &mut Config, mode: Mode) -> f64 {
        let target_core_temperature = match config.mode_config(mode).core_temp_thresh {
            TemperatureThreshold::Disabled => u64::MAX,
            TemperatureThreshold::Temp(t) => t,
        };

        self.temperature_update();
        if self.core_temperature > target_core_temperature {
            self.target_fps_offset -= 0.1;
        } else {
            self.target_fps_offset += 0.1;
        }

        // self.target_fps_offset = self.target_fps_offset.clamp(-5.0, 0.0);
        self.target_fps_offset
    }

    fn temperature_update(&mut self) {
        self.core_temperature = self
            .nodes
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .map(|temp| temp.trim().parse::<u64>().unwrap_or_default())
            .max()
            .unwrap();
    }
}
