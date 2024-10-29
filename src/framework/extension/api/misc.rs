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
