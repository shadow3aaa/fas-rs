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
//! # Quick Start
//!
//! ```ignore
//! use fas_rs_fw::config::CONFIG;
//!
//! if let Some((game, fps, windows)) = CONFIG.cur_game_fps() {
//!     // do something
//! }
//! let foo = CONFIG.get_conf("foo")
//!     .and_then(|f| Some(f.as_str()?.to_string()))
//!     .unwrap();
//! ```
mod merge;
mod read;
mod single;

pub use merge::merge;
pub use single::CONFIG;

use std::{
    fs,
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use likely_stable::LikelyOption;
use log::info;
use parking_lot::RwLock;
use toml::Value;

use read::wait_and_read;

type ConfData = RwLock<Value>;
pub struct Config {
    toml: Arc<ConfData>,
    exit: Arc<AtomicBool>,
}

impl Drop for Config {
    fn drop(&mut self) {
        self.exit.store(true, Ordering::Release);
    }
}

impl Config {
    #[must_use]
    pub(crate) fn new(path: &Path) -> Self {
        let ori = fs::read_to_string(path).unwrap();
        let toml = toml::from_str(&ori).unwrap();
        let toml = Arc::new(RwLock::new(toml));
        let exit = Arc::new(AtomicBool::new(false));

        {
            let path = path.to_owned();
            let toml = toml.clone();
            let exit = exit.clone();

            thread::Builder::new()
                .name("ConfigThread".into())
                .spawn(move || wait_and_read(&path, &toml, &exit))
                .unwrap();
        }
        info!("Config watcher started");

        Self { toml, exit }
    }

    /// 从配置中读取现在的游戏和目标fps、帧窗口大小
    ///
    /// # Panics
    ///
    /// 读取/解析配置失败
    pub fn cur_game_fps(&self) -> Option<(String, u32, u32)> {
        let toml = self.toml.read();
        #[allow(unused)]
        let list = toml
            .get("game_list")
            .and_then_likely(Value::as_table)
            .cloned()
            .unwrap();

        drop(toml); // early-drop

        let pkgs = Self::get_top_pkgname()?;
        let pkg = pkgs.into_iter().find(|key| list.contains_key(key))?;

        let (game, fps_windows) = (&pkg, list.get(&pkg)?.as_array().unwrap());

        let fps_windows: Vec<_> = fps_windows
            .iter()
            .map(|v| u32::try_from(v.as_integer().unwrap()).unwrap())
            .collect();

        let fps_windows: [u32; 2] = fps_windows.as_slice().try_into().unwrap();

        Some((game.clone(), fps_windows[0], fps_windows[1]))
    }

    /// 从配置中读取一个配置参数的值
    ///
    /// # Panics
    ///
    /// 读取配置不存在config table
    #[allow(unused)]
    #[must_use]
    pub fn get_conf(&self, label: &'static str) -> Option<Value> {
        let toml = self.toml.read();
        toml.get("config").unwrap().get(label).cloned()
    }

    fn get_top_pkgname() -> Option<Vec<String>> {
        let dump = Command::new("dumpsys")
            .args(["window", "visible-apps"])
            .output()
            .ok()?;
        let dump = String::from_utf8_lossy(&dump.stdout).into_owned();

        let result = parse_top_app(&dump);

        if result.is_empty() {
            return None;
        }

        Some(result)
    }
}

fn parse_top_app(dump: &str) -> Vec<String> {
    let mut result = Vec::new();
    for l in dump.lines() {
        if l.contains("package=") {
            if let Some(p) = l
                .split_whitespace()
                .nth(2)
                .and_then_likely(|p| p.split('=').nth(1))
            {
                result.push(p.to_string());
            }
        } else if l.contains("canReceiveKeys()=false") {
            result.pop();
        }
    }

    result
}

#[test]
fn test_topapp_parse() {
    let test1 = r"#* ActivityRecord{2cd21f8 u0 com.lbe.security.miui/com.android.packageinstaller.permission.ui.GrantPermissionsActivity} t105}
    mOwnerUid=10248 showForAllUsers=false package=bin.mt.plus appop=NONE
    mHasSurface=true isReadyForDisplay()=true canReceiveKeys()=true mWindowRemovalAllowed=false
    #";

    let test2 = "mOwnerUid=10248 showForAllUsers=false package=bin.mt.plus appop=NONE";

    assert!(parse_top_app(test1).contains(&"bin.mt.plus".to_string()));
    assert!(parse_top_app(test2).contains(&"bin.mt.plus".to_string()));
}
