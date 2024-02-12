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
mod default;

use serde_derive::{Deserialize, Serialize};
use toml::Table;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigData {
    pub config: Config,
    pub game_list: Table,
    pub powersave: ModeConfig,
    pub balance: ModeConfig,
    pub performance: ModeConfig,
    pub fast: ModeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    #[serde(default = "Config::default_value_keep_std")]
    pub keep_std: bool,
    #[serde(default = "Config::default_value_userspace_governor")]
    pub userspace_governor: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ModeConfig {
    pub fas_boost: bool,
    pub use_performance_governor: bool,
}
