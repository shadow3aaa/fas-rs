// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

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
