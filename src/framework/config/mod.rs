// Copyright 2024-2025, shadow3aaa
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

mod data;
mod inner;
mod merge;
mod read;

use std::{fs, path::Path, sync::mpsc, thread};

use inner::Inner;
use log::{error, info};
use toml::Value;

use crate::framework::{error::Result, node::Mode};
pub use data::{ConfigData, MarginFps, ModeConfig, TemperatureThreshold};
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
    pub fn new<P>(p: P, sp: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
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

    pub fn need_fas<S>(&mut self, pkg: S) -> bool
    where
        S: AsRef<str>,
    {
        let pkg = pkg.as_ref();

        self.inner.config().game_list.contains_key(pkg)
            || self.inner.config().scene_game_list.contains(pkg)
    }

    pub fn target_fps<S>(&mut self, pkg: S) -> Option<TargetFps>
    where
        S: AsRef<str>,
    {
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
    pub fn mode_config(&mut self, m: Mode) -> &ModeConfig {
        match m {
            Mode::Powersave => &self.inner.config().powersave,
            Mode::Balance => &self.inner.config().balance,
            Mode::Performance => &self.inner.config().performance,
            Mode::Fast => &self.inner.config().fast,
        }
    }

    pub fn logger_level(&mut self) -> log::LevelFilter {
        match self.inner.config().logger_level.as_str() {
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            _ => {
                log::error!("logger level is error");
                log::LevelFilter::Info
            }
        }
    }
}
