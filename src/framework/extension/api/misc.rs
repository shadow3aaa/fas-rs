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

use std::path::Path;

use likely_stable::LikelyResult;
use log::error;
use mlua::{Function, IntoLuaMulti, Lua};

pub fn get_api_version(lua: &Lua) -> u8 {
    lua.globals().get("API_VERSION").unwrap_or(0)
}

pub fn do_callback<P: AsRef<Path>, S: AsRef<str>, A: IntoLuaMulti>(
    extension: P,
    lua: &Lua,
    function: S,
    args: A,
) {
    let function = function.as_ref();
    let extension = extension.as_ref();

    if let Ok(func) = lua.globals().get::<Function>(function) {
        func.call(args).unwrap_or_else_likely(|e| {
            error!("Got an error when executing extension '{extension:?}', reason: {e:#?}");
        });
    }
}
