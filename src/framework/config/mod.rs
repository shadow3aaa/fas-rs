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

mod data;
mod merge;
mod read;
mod inner;

use std::{fs, path::Path, sync::mpsc, thread};

use inner::Inner;
use log::{error, info};
use toml::Value;

use crate::framework::{error::Result, node::Mode};
use data::{Config as ConfigConfig, ConfigData, ModeConfig};
use read::wait_and_read;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetFps {
    Value(u32),
    Array(Vec<u32>),
}

#[derive(Debug)]
pub struct Config {
    inner: Inner,
}

impl Config {
    pub fn new<P: AsRef<Path>>(p: P, sp: P) -> Result<Self> {
        let path = p.as_ref();
        let std_path = sp.as_ref();
        let toml_raw = fs::read_to_string(path)?;
        let toml: ConfigData = toml::from_str(&toml_raw)?;

        let (sx, rx) = mpsc::channel();
        let inner = Inner::new(toml, rx);

        {
            let path = path.to_owned();
            let std_path = std_path.to_owned();

            thread::Builder::new()
                .name("ConfigThread".into())
                .spawn(move || {
                    wait_and_read(&path, &std_path, &sx).unwrap_or_else(|e| error!("{e:#?}"));
                    panic!("An unrecoverable error occurred!");
                })?;
        }

        info!("Config watcher started");

        Ok(Self { inner })
    }

    pub fn need_fas<S: AsRef<str>>(&mut self, pkg: S) -> bool {
        let pkg = pkg.as_ref();

        self.inner.config().game_list.contains_key(pkg) || self.inner.config().scene_game_list.contains(pkg)
    }

    pub fn target_fps<S: AsRef<str>>(&mut self, pkg: S) -> Option<TargetFps> {
        let pkg = pkg.as_ref();
        let pkg = pkg.split(':').next()?;

        self.inner.config().game_list.get(pkg).cloned().map_or_else(
            || {
                if self.inner.config().scene_game_list.contains(pkg) {
                    Some(TargetFps::Array(vec![30, 45, 60, 90, 120, 144]))
                } else {
                    None
                }
            },
            |value| match value {
                Value::Array(arr) => {
                    let mut arr: Vec<_> = arr
                        .iter()
                        .filter_map(toml::Value::as_integer)
                        .map(|i| i as u32)
                        .collect();
                    arr.sort_unstable();
                    Some(TargetFps::Array(arr))
                }
                Value::Integer(i) => Some(TargetFps::Value(i as u32)),
                Value::String(s) => {
                    if s == "auto" {
                        Some(TargetFps::Array(vec![30, 45, 60, 90, 120, 144]))
                    } else {
                        error!("Find target game {pkg} in config, but meet illegal data type");
                        error!("Sugg: try \'{pkg} = \"auto\"\'");
                        None
                    }
                }
                _ => {
                    error!("Find target game {pkg} in config, but meet illegal data type");
                    error!("Sugg: try \'{pkg} = \"auto\"\'");
                    None
                }
            },
        )
    }

    #[must_use]
    pub fn mode_config(&mut self, m: Mode) -> ModeConfig {
        match m {
            Mode::Powersave =>self.inner.config().powersave,
            Mode::Balance => self.inner.config().balance,
            Mode::Performance => self.inner.config().performance,
            Mode::Fast => self.inner.config().fast,
        }
    }

    #[must_use]
    pub fn config(&mut self) -> ConfigConfig {
        self.inner.config().config
    }
}
