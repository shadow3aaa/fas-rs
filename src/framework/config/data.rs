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
use serde_derive::Deserialize;
use toml::Table;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigData {
    pub config: Config,
    pub game_list: Table,
    pub powersave: ModeConfig,
    pub balance: ModeConfig,
    pub performance: ModeConfig,
    pub fast: ModeConfig,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Config {
    pub keep_std: bool,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ModeConfig {
    pub fas_boost: bool,
    pub use_performance_governor: bool,
    pub scale: f64,
}
