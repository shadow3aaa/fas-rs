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
use libc::pid_t;
use log::error;
use mlua::Function;

use super::core::ExtensionMap;

pub enum CallBacks {
    LoadFas(pid_t, String),
    UnloadFas(pid_t, String),
    StartFas,
    StopFas,
    InitCpuFreq,
    ResetCpuFreq,
}

impl CallBacks {
    pub fn do_callback(self, map: &ExtensionMap) {
        match self {
            Self::LoadFas(pid, pkg) => {
                for (extension, lua) in map {
                    if let Ok(func) = lua.globals().get::<_, Function>("load_fas") {
                        func.call((pid, pkg.clone())).unwrap_or_else(|e| error!("Got an error when executing extension '{extension:?}', reason: {e:#?}"));
                    }
                }
            }
            Self::UnloadFas(pid, pkg) => {
                for (extension, lua) in map {
                    if let Ok(func) = lua.globals().get::<_, Function>("unload_fas") {
                        func.call((pid, pkg.clone())).unwrap_or_else(|e| error!("Got an error when executing extension '{extension:?}', reason: {e:#?}"));
                    }
                }
            }
            Self::StartFas => {
                for (extension, lua) in map {
                    if let Ok(func) = lua.globals().get::<_, Function>("start_fas") {
                        func.call(()).unwrap_or_else(|e| error!("Got an error when executing extension '{extension:?}', reason: {e:#?}"));
                    }
                }
            }
            Self::StopFas => {
                for (extension, lua) in map {
                    if let Ok(func) = lua.globals().get::<_, Function>("stop_fas") {
                        func.call(()).unwrap_or_else(|e| error!("Got an error when executing extension '{extension:?}', reason: {e:#?}"));
                    }
                }
            }
            Self::InitCpuFreq => {
                for (extension, lua) in map {
                    if let Ok(func) = lua.globals().get::<_, Function>("init_cpu_freq") {
                        func.call(()).unwrap_or_else(|e| error!("Got an error when executing extension '{extension:?}', reason: {e:#?}"));
                    }
                }
            }
            Self::ResetCpuFreq => {
                for (extension, lua) in map {
                    if let Ok(func) = lua.globals().get::<_, Function>("reset_cpu_freq") {
                        func.call(()).unwrap_or_else(|e| error!("Got an error when executing extension '{extension:?}', reason: {e:#?}"));
                    }
                }
            }
        }
    }
}
