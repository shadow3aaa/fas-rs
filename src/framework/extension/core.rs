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
use std::{collections::HashMap, fs, path::PathBuf, sync::mpsc::Receiver, time::Duration};

use inotify::{Inotify, WatchMask};
use log::{debug, error, info};
use rlua::Lua;

use super::{callbacks::CallBacks, EXTENSIONS_PATH};
use crate::framework::error::Result;

pub type ExtensionMap = HashMap<PathBuf, Lua>;

pub fn thread(rx: &Receiver<CallBacks>) {
    let mut extensions = load_extensions().unwrap_or_default();
    let mut inotify = Inotify::init().unwrap();

    inotify
        .watches()
        .add(
            EXTENSIONS_PATH,
            WatchMask::CLOSE_WRITE | WatchMask::CREATE | WatchMask::DELETE,
        )
        .unwrap();

    loop {
        if need_update(&mut inotify) {
            extensions = load_extensions().unwrap_or_default();
        }

        let Ok(callback) = rx.recv_timeout(Duration::from_secs(1)) else {
            continue;
        };

        callback.do_callback(&extensions);
    }
}

fn need_update(inotify: &mut Inotify) -> bool {
    inotify.read_events(&mut [0; 1024]).is_ok()
}

fn load_extensions() -> Result<ExtensionMap> {
    let mut map: ExtensionMap = HashMap::new();

    for file in fs::read_dir(EXTENSIONS_PATH)?
        .map(std::result::Result::unwrap)
        .filter(|f| f.file_type().unwrap().is_file() && f.path().extension().unwrap() == "lua")
    {
        let lua = Lua::new();
        let path = file.path();
        let file = fs::read_to_string(&path)?;

        let result = lua.context(|context| {
            context.globals().set(
                "log_info",
                context.create_function(|_, message: String| {
                    info!("extension: {message}");
                    Ok(())
                })?,
            )?;
            context.globals().set(
                "log_debug",
                context.create_function(|_, message: String| {
                    debug!("extension: {message}");
                    Ok(())
                })?,
            )?;
            context.globals().set(
                "log_error",
                context.create_function(|_, message: String| {
                    error!("extension: {message}");
                    Ok(())
                })?,
            )?;

            context.load(&file).exec()
        });

        match result {
            Ok(()) => {
                info!("Extension loaded successfully: {path:?}");
                map.insert(path, lua);
            }
            Err(e) => error!("Extension loading failed, reason: {e:#?}"),
        }
    }

    Ok(map)
}
