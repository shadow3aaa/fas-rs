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
use std::{fs, path::Path, sync::Arc, thread, time::Duration};

use inotify::{Inotify, WatchMask};
use log::{debug, error};
use parking_lot::RwLock;
use toml::Value;

use super::data::{ConfigData, SceneAppList};
use crate::framework::error::Result;

const SCENE_PROFILE: &str = "/data/data/com.omarea.vtools/shared_prefs/games.xml";

pub(super) fn wait_and_read(
    path: &Path,
    std_path: &Path,
    toml: &Arc<RwLock<ConfigData>>,
) -> Result<()> {
    let mut retry_count = 0;

    let std_config = fs::read_to_string(std_path)?;
    let std_config: ConfigData = toml::from_str(&std_config)?;

    loop {
        check_counter(&mut retry_count, toml, &std_config);

        let ori = match fs::read_to_string(path) {
            Ok(s) => {
                retry_count = 0;
                s
            }
            Err(e) => {
                debug!("Failed to read config {path:?}, reason: {e}");
                retry_count += 1;
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        *toml.write() = match toml::from_str(&ori) {
            Ok(o) => {
                retry_count = 0;
                o
            }
            Err(e) => {
                error!("Failed to parse config {path:?}, reason: {e}");
                error!("Trying to roll back to the last configuration that could be resolved...");
                let latest = toml::to_string(&*toml.read())?;
                if fs::write(path, latest).is_ok() {
                    error!("Rollback successful");
                }

                retry_count += 1;
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let scene_profile = Path::new(SCENE_PROFILE);

        let _ = read_scene_games(scene_profile, toml);
        wait_until_update(path, scene_profile)?;
    }
}

fn check_counter(retry_count: &mut u8, toml: &Arc<RwLock<ConfigData>>, std_config: &ConfigData) {
    if *retry_count > 10 {
        error!("Too many read / parse user config retries");
        error!("Use std profile instead until we could read and parse user config");

        *toml.write() = std_config.clone();
        *retry_count = 0;
    }
}

fn read_scene_games(scene_profile: &Path, toml: &Arc<RwLock<ConfigData>>) -> Result<()> {
    if scene_profile.exists() {
        let scene_apps = fs::read_to_string(scene_profile)?;
        let scene_apps: SceneAppList = quick_xml::de::from_str(&scene_apps)?;

        for pkg in scene_apps
            .apps
            .into_iter()
            .filter(|app| app.is_game)
            .map(|game| game.pkg)
        {
            toml.write()
                .game_list
                .entry(pkg)
                .or_insert(Value::String("auto".to_string()));
        }
    }

    Ok(())
}

fn wait_until_update(path: &Path, scene_profile: &Path) -> Result<()> {
    let mut inotify = Inotify::init()?;

    if scene_profile.exists() {
        let _ = inotify
            .watches()
            .add(scene_profile, WatchMask::CLOSE_WRITE | WatchMask::MODIFY);
    }

    if inotify
        .watches()
        .add(path, WatchMask::CLOSE_WRITE | WatchMask::MODIFY)
        .is_ok()
    {
        let _ = inotify.read_events_blocking(&mut []);
    }

    Ok(())
}
