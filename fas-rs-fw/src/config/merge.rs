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
use std::convert::AsRef;

use likely_stable::LikelyOption;
use serde_derive::{Deserialize, Serialize};
use toml::Table;

use super::Config;
use crate::error::{Error, Result};

/// fas配置
#[derive(Deserialize, Serialize)]
struct ConfigData {
    pub config: Table,
    pub game_list: Table,
}

impl Config {
    /// 合并标准配置和本地配置
    ///
    /// # Errors
    ///
    /// 解析错误
    pub fn merge<S: AsRef<str>>(l: S, s: S) -> Result<String> {
        let local_conf = l.as_ref();
        let std_conf = s.as_ref();

        let std_conf: ConfigData = toml::from_str(std_conf)?;
        let local_conf: ConfigData = toml::from_str(local_conf)?;

        if local_conf
            .config
            .get("keep_std")
            .and_then_likely(toml::Value::as_bool)
            .ok_or(Error::ParseConfig)?
        {
            let new_conf = ConfigData {
                config: std_conf.config,
                game_list: local_conf.game_list,
            };
            return Ok(toml::to_string(&new_conf)?);
        }

        let old_config = local_conf.config;
        let mut std_config = std_conf.config;

        let old_config: Table = old_config
            .into_iter()
            .filter(|(k, _)| std_config.contains_key(k))
            .collect();

        std_config.extend(old_config);

        let new_conf = ConfigData {
            config: std_config,
            game_list: local_conf.game_list,
        };

        Ok(toml::to_string_pretty(&new_conf)?)
    }
}
