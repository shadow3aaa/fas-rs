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
use std::error::Error;

use likely_stable::LikelyOption;
use serde_derive::{Deserialize, Serialize};
use toml::Table;

/// fas配置
#[derive(Deserialize, Serialize)]
struct Config {
    pub config: Table,
    pub game_list: Table,
}

/// 合并标准配置和本地配置
///
/// # Errors
///
/// 无法读取配置
pub fn merge(local_conf: &str, std_conf: &str) -> Result<String, Box<dyn Error>> {
    let std_conf: Config = toml::from_str(std_conf)?;
    let local_conf: Config = toml::from_str(local_conf)?;

    if local_conf
        .config
        .get("keep_std")
        .and_then_likely(toml::Value::as_bool)
        .unwrap_or(false)
    {
        let new_conf = Config {
            config: std_conf.config,
            game_list: local_conf.game_list,
        };
        return Ok(toml::to_string(&new_conf)?);
    }

    let old_config = local_conf.config;
    let std_config = std_conf.config;

    let mut new_config: Table = old_config
        .clone()
        .into_iter()
        .filter(|(k, _)| std_config.contains_key(k))
        .collect();

    new_config.extend(
        std_config
            .into_iter()
            .filter(|(k, _)| !old_config.contains_key(k)),
    );

    let new_conf = Config {
        config: new_config,
        game_list: local_conf.game_list,
    };

    Ok(toml::to_string(&new_conf)?)
}
