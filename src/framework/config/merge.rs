// Copyright 2023-2025, shadow3aaa
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

use likely_stable::LikelyOption;
use serde::{Deserialize, Serialize};
use toml::{Table, Value};

use super::Config;
use crate::framework::error::{Error, Result};

#[derive(Deserialize, Serialize)]
struct ConfigData {
    pub config: Table,
    pub game_list: Table,
    pub powersave: Table,
    pub balance: Table,
    pub performance: Table,
    pub fast: Table,
}

impl Config {
    pub fn merge<S: AsRef<str>>(l: S, s: S) -> Result<String> {
        let local_conf = l.as_ref();
        let std_conf = s.as_ref();

        let std_conf: ConfigData = toml::from_str(std_conf)?;
        let local_conf: ConfigData = toml::from_str(local_conf)?;

        if local_conf
            .config
            .get("keep_std")
            .and_then_likely(Value::as_bool)
            .ok_or(Error::ParseConfig)?
        {
            let new_conf = ConfigData {
                config: std_conf.config,
                game_list: local_conf.game_list,
                powersave: std_conf.powersave,
                balance: std_conf.balance,
                performance: std_conf.performance,
                fast: std_conf.fast,
            };
            return Ok(toml::to_string(&new_conf)?);
        }

        let config = Self::table_merge(std_conf.config, local_conf.config);
        let powersave = Self::table_merge(std_conf.powersave, local_conf.powersave);
        let balance = Self::table_merge(std_conf.balance, local_conf.balance);
        let performance = Self::table_merge(std_conf.performance, local_conf.performance);
        let fast = Self::table_merge(std_conf.fast, local_conf.fast);

        let new_conf = ConfigData {
            config,
            game_list: local_conf.game_list,
            powersave,
            balance,
            performance,
            fast,
        };

        Ok(toml::to_string(&new_conf)?)
    }

    fn table_merge(mut s: Table, l: Table) -> Table {
        let old: Table = l.into_iter().filter(|(k, _)| s.contains_key(k)).collect();
        s.extend(old);
        s
    }
}
