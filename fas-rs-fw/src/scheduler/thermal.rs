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
use std::{fs, num::IntErrorKind, path::PathBuf, time::Duration};

use crate::{
    error::{Error, Result},
    Config, Node,
};

const POWERSAVE_DUR_MAX: Duration = Duration::from_nanos(1_000_000);
const BALANCE_DUR_MAX: Duration = Duration::from_nanos(500_000);
const PERFORMANCE_DUR_MAX: Duration = Duration::from_nanos(250_000);
const FAST_DUR_MAX: Duration = Duration::from_nanos(100_000);

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

    #[allow(clippy::cast_precision_loss)]
    pub fn thermal(&self) -> Result<Duration> {
        let temp = self.temp()?;
        let mode = Node::read_node("mode")?;

        Ok(match mode.trim() {
            "powersave" => {
                let rhs = Self::rhs(
                    temp as f32,
                    self.powersave as f32 + 500.0,
                    self.powersave as f32,
                );
                POWERSAVE_DUR_MAX.mul_f32(rhs)
            }
            "balance" => {
                let rhs = Self::rhs(
                    temp as f32,
                    self.balance as f32 + 500.0,
                    self.balance as f32,
                );
                BALANCE_DUR_MAX.mul_f32(rhs)
            }
            "performance" => {
                let rhs = Self::rhs(
                    temp as f32,
                    self.performance as f32 + 500.0,
                    self.performance as f32,
                );
                PERFORMANCE_DUR_MAX.mul_f32(rhs)
            }
            "fast" => {
                let rhs = Self::rhs(temp as f32, self.fast as f32 + 500.0, self.fast as f32);
                FAST_DUR_MAX.mul_f32(rhs)
            }
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
                _ => return Err(Error::Other("Failed to parse temp")),
            },
        };

        Ok(temp)
    }

    fn rhs(cur: f32, max: f32, min: f32) -> f32 {
        if cur > max {
            return 1.0;
        } else if cur < min {
            return 0.0;
        }

        let per = (cur - min) / (max - min);
        1.0 - (per * -2.8).exp()
    }
}
