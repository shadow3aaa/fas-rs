// Copyright 2023-2025, shadow3 (@shadow3aaa)
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

use std::{fs, path::Path, sync::mpsc::Sender, time::Duration};

use inotify::{Inotify, WatchMask};
use log::{debug, error};

use super::data::{ConfigData, SceneAppList};
use crate::framework::error::Result;

const SCENE_PROFILE: &str = "/data/data/com.omarea.vtools/shared_prefs/games.xml";
const MAX_RETRY_COUNT: u8 = 10;

pub(super) fn wait_and_read(path: &Path, std_path: &Path, sx: &Sender<ConfigData>) -> Result<()> {
    let std_config = read_config(std_path)?;

    loop {
        match read_config_with_retry(path) {
            Ok(mut config) => {
                if config.config.scene_game_list {
                    if let Err(e) = read_scene_games(&mut config) {
                        error!("Failed to read scene games: {}", e);
                    }
                }
                sx.send(config).unwrap();
            }
            Err(e) => {
                error!("Too many retries reading config: {}", e);
                error!("Using standard profile until user config is available.");
                sx.send(std_config.clone()).unwrap();
            }
        }

        wait_until_update(path)?;
    }
}

fn read_config(path: &Path) -> Result<ConfigData> {
    let content = fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;
    Ok(config)
}

fn read_config_with_retry(path: &Path) -> Result<ConfigData> {
    let mut retry_count = 0;

    loop {
        match read_config(path) {
            Ok(config) => return Ok(config),
            Err(e) => {
                debug!("Failed to read config at {:?}: {}", path, e);
                retry_count += 1;
                if retry_count >= MAX_RETRY_COUNT {
                    return Err(e);
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn read_scene_games(config: &mut ConfigData) -> Result<()> {
    if Path::new(SCENE_PROFILE).exists() {
        let scene_apps = fs::read_to_string(SCENE_PROFILE)?;
        let scene_apps: SceneAppList = quick_xml::de::from_str(&scene_apps)?;
        let game_list = scene_apps
            .apps
            .into_iter()
            .filter(|app| app.is_game)
            .map(|game| game.pkg)
            .collect();

        config.scene_game_list = game_list;
    }

    Ok(())
}

fn wait_until_update(path: &Path) -> Result<()> {
    let mut inotify = Inotify::init()?;

    if Path::new(SCENE_PROFILE).exists() {
        inotify
            .watches()
            .add(SCENE_PROFILE, WatchMask::MODIFY | WatchMask::CLOSE_WRITE)?;
    }

    inotify
        .watches()
        .add(path, WatchMask::MODIFY | WatchMask::CLOSE_WRITE)?;

    let mut buffer = [0; 1024];
    inotify.read_events_blocking(&mut buffer)?;

    Ok(())
}
