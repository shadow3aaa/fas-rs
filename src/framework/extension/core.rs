// Copyright 2023-2024, shadow3 (@shadow3aaa)
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

use std::{collections::HashMap, fs, path::PathBuf, sync::mpsc::Receiver, time::Duration};

use inotify::{Inotify, WatchMask};
use log::{debug, error, info};
use mlua::Lua;

use super::{
    api::{helper_funs, Api},
    EXTENSIONS_PATH,
};
use crate::framework::error::Result;

pub type ExtensionMap = HashMap<PathBuf, Lua>;

pub fn thread(rx: &Receiver<Box<dyn Api>>) {
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

        if let Ok(trigger) = rx.recv_timeout(Duration::from_secs(1)) {
            trigger.handle_api(&extensions);
        }
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

        lua.globals().set(
            "log_info",
            lua.create_function(|_, message: String| {
                info!("extension: {message}");
                Ok(())
            })?,
        )?;

        lua.globals().set(
            "log_debug",
            lua.create_function(|_, message: String| {
                debug!("extension: {message}");
                Ok(())
            })?,
        )?;

        lua.globals().set(
            "log_error",
            lua.create_function(|_, message: String| {
                error!("extension: {message}");
                Ok(())
            })?,
        )?;

        // Add in api v1
        // Removed in v4.2.0(apiv4)
        lua.globals().set(
            "set_policy_freq_offset",
            lua.create_function(|_, (policy, offset)| {
                helper_funs::set_policy_freq_offset(policy, offset);
                Ok(())
            })?,
        )?;

        // Add in api v3
        lua.globals().set(
            "set_ignore_policy",
            lua.create_function(|_, (policy, val)| {
                helper_funs::set_ignore_policy(policy, val);
                Ok(())
            })?,
        )?;

        // Add in api v4
        lua.globals().set(
            "set_extra_policy_abs",
            lua.create_function(|_, (policy, min, max)| {
                helper_funs::set_extra_policy_abs(policy, min, max);
                Ok(())
            })?,
        )?;

        // Add in api v4
        lua.globals().set(
            "set_extra_policy_rel",
            lua.create_function(|_, (policy, target_policy, min, max)| {
                helper_funs::set_extra_policy_rel(policy, target_policy, min, max);
                Ok(())
            })?,
        )?;

        // Add in api v4
        lua.globals().set(
            "remove_extra_policy",
            lua.create_function(|_, policy| {
                helper_funs::remove_extra_policy(policy);
                Ok(())
            })?,
        )?;

        match lua.load(&file).exec() {
            Ok(()) => {
                info!("Extension loaded successfully: {path:?}");
                map.insert(path, lua);
            }
            Err(e) => {
                error!("Extension loading failed, reason: {e:#?}");
            }
        }
    }

    Ok(map)
}
