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

use std::{fs, path::Path, sync::mpsc::Sender, thread, time::Duration};

use inotify::{Inotify, WatchMask};
use log::{debug, error};

use super::data::{ConfigData, SceneAppList};
use crate::framework::error::Result;

const SCENE_PROFILE: &str = "/data/data/com.omarea.vtools/shared_prefs/games.xml";

pub(super) fn wait_and_read(path: &Path, std_path: &Path, sx: &Sender<ConfigData>) -> Result<()> {
    let mut retry_count = 0;

    let std_config = fs::read_to_string(std_path)?;
    let std_config: ConfigData = toml::from_str(&std_config)?;

    loop {
        check_counter_final(&mut retry_count, sx, &std_config);

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

        let mut toml: ConfigData = match toml::from_str(&ori) {
            Ok(o) => {
                retry_count = 0;
                o
            }
            Err(e) => {
                assert!(
                    retry_count <= 3,
                    "Failed to parse config {path:?}, reason: {e}, go panic."
                );

                retry_count += 1;
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        if toml.config.scene_game_list {
            let _ = read_scene_games(&mut toml);
        }

        sx.send(toml).unwrap();

        wait_until_update(path)?;
    }
}

fn check_counter_final(retry_count: &mut u8, sx: &Sender<ConfigData>, std_config: &ConfigData) {
    if *retry_count > 10 {
        error!("Too many read / parse user config retries");
        error!("Use std profile instead until we could read and parse user config");

        sx.send(std_config.clone()).unwrap();
        *retry_count = 0;
    }
}

fn read_scene_games(toml: &mut ConfigData) -> Result<()> {
    if Path::new(SCENE_PROFILE).exists() {
        let scene_apps = fs::read_to_string(SCENE_PROFILE)?;
        let scene_apps: SceneAppList = quick_xml::de::from_str(&scene_apps)?;
        let game_list = scene_apps
            .apps
            .into_iter()
            .filter(|app| app.is_game)
            .map(|game| game.pkg)
            .collect();

        toml.scene_game_list = game_list;
    }

    Ok(())
}

fn wait_until_update<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let mut inotify = Inotify::init()?;

    if Path::new(SCENE_PROFILE).exists() {
        let _ = inotify
            .watches()
            .add(SCENE_PROFILE, WatchMask::CLOSE_WRITE | WatchMask::MODIFY);
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
