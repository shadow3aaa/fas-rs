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
mod merge;
mod read;
mod topapp;

use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
};

use likely_stable::LikelyOption;
use log::{info, trace};
use parking_lot::RwLock;
use toml::Value;

use crate::error::{Error, Result};

use read::wait_and_read;

type ConfData = RwLock<Value>;
pub struct Config {
    toml: Arc<ConfData>,
    exit: Arc<AtomicBool>,
    topapp: Option<Vec<String>>,
    topapp_rx: Receiver<Option<Vec<String>>>,
}

impl Drop for Config {
    fn drop(&mut self) {
        self.exit.store(true, Ordering::Release);
    }
}

impl Config {
    /// 读取配置
    ///
    /// # Errors
    ///
    /// 解析配置失败/路径不存在
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Self> {
        let path = p.as_ref();
        let ori = fs::read_to_string(path)?;

        let toml = toml::from_str(&ori)?;
        let toml = Arc::new(RwLock::new(toml));

        let exit = Arc::new(AtomicBool::new(false));

        {
            let path = path.to_owned();
            let toml = toml.clone();
            let exit = exit.clone();

            thread::Builder::new()
                .name("ConfigThread".into())
                .spawn(move || wait_and_read(&path, &toml, &exit))?;
        }

        let (sx, rx) = mpsc::channel();

        {
            let exit = exit.clone();

            thread::Builder::new()
                .name("TopappThread".into())
                .spawn(move || Self::topapp_updater(&sx, &exit))?;
        }

        info!("Config watcher started");

        Ok(Self {
            toml,
            exit,
            topapp: None,
            topapp_rx: rx,
        })
    }

    /// 从配置中读取现在的游戏和目标fps、帧窗口大小
    ///
    /// # Panics
    ///
    /// 读取/解析配置失败
    pub fn cur_game_fps(&mut self) -> Option<(String, u32)> {
        if let Some(t) = self.topapp_rx.try_iter().last() {
            trace!("topapp: {t:?}");
            self.topapp = t;
        }

        let pkgs = self.topapp.clone()?;

        let toml = self.toml.read();
        let list = toml
            .get("game_list")
            .and_then_likely(Value::as_table)
            .cloned()
            .unwrap();

        drop(toml); // early-drop

        let pkg = pkgs.into_iter().find(|key| list.contains_key(key))?;

        let target_fps = list
            .get(&pkg)?
            .as_integer()
            .ok_or(Error::Other("Failed to parse target_fps"))
            .unwrap();

        Some((pkg, target_fps.try_into().unwrap()))
    }

    /// 从配置中读取一个配置参数的值
    ///
    /// # Errors
    ///
    /// 读取目标配置失败
    #[allow(unused)]
    pub fn get_conf<S: AsRef<str>>(&self, l: S) -> Result<Value> {
        let label = l.as_ref();

        let toml = self.toml.read();
        toml.get("config")
            .and_then_likely(|t| t.get(label).cloned())
            .ok_or(Error::ConfigValueNotFound)
    }
}
