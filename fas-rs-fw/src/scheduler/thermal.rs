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
use std::{fs, num::IntErrorKind, path::PathBuf};

use crate::{
    error::{Error, Result},
    Config, Node,
};

pub type Temp = i32;

pub struct Thermal {
    temp_path: PathBuf,
    powersave: Temp,
    balance: Temp,
    performance: Temp,
    fast: Temp,
}

impl Thermal {
    pub fn new(config: &Config) -> Result<Self> {
        let temp_path = fs::read_dir("/sys/devices/virtual/thermal")?
            .filter_map(std::result::Result::ok)
            .find_map(|t| {
                let thermal_type = t.path().join("type");
                let thermal_type = fs::read_to_string(thermal_type).ok()?;
                if thermal_type.trim() == "battery" {
                    let path = t.path().join("temp");
                    if path.exists() {
                        Some(path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or(Error::Other("No battery thermal device found"))?;

        Ok(Self {
            temp_path,
            powersave: config
                .get_conf("powersave_thermal")?
                .as_integer()
                .ok_or(Error::ParseConfig)?
                .try_into()
                .map_err(|_| Error::ParseConfig)?,
            balance: config
                .get_conf("balance_thermal")?
                .as_integer()
                .ok_or(Error::ParseConfig)?
                .try_into()
                .map_err(|_| Error::ParseConfig)?,
            performance: config
                .get_conf("performance_thermal")?
                .as_integer()
                .ok_or(Error::ParseConfig)?
                .try_into()
                .map_err(|_| Error::ParseConfig)?,
            fast: config
                .get_conf("fast_thermal")?
                .as_integer()
                .ok_or(Error::ParseConfig)?
                .try_into()
                .map_err(|_| Error::ParseConfig)?,
        })
    }

    pub fn need_thermal(&self) -> Result<bool> {
        let temp = self.temp()?;
        let mode = Node::read_node("mode")?;

        Ok(match mode.trim() {
            "powersave" => temp >= self.powersave,
            "balance" => temp >= self.balance,
            "performance" => temp >= self.performance,
            "fast" => temp >= self.fast,
            _ => return Err(Error::ParseNode),
        })
    }

    fn temp(&self) -> Result<Temp> {
        let temp = fs::read_to_string(&self.temp_path)?;
        let temp: Temp = match temp.trim().parse() {
            Ok(o) => o,
            Err(e) => match e.kind() {
                IntErrorKind::PosOverflow => Temp::MAX,
                IntErrorKind::NegOverflow => Temp::MIN,
                _ => {
                    log::error!("Failed to parse temp: {temp}");
                    return Err(Error::Other("Failed to parse temp"));
                }
            },
        };

        Ok(temp)
    }
}
