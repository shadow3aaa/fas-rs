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

use std::sync::atomic::Ordering;

use libc::pid_t;

use crate::cpu_common::OFFSET_MAP;

use super::{
    super::core::ExtensionMap,
    misc::{do_callback, get_api_version},
    Api,
};

#[derive(Debug, Clone)]
pub enum ApiV1 {
    LoadFas(pid_t, String),
    UnloadFas(pid_t, String),
    StartFas,
    StopFas,
    InitCpuFreq,
    ResetCpuFreq,
}

impl Api for ApiV1 {
    fn handle_api(&self, ext: &ExtensionMap) {
        for (extension, lua) in ext.iter().filter(|(_, lua)| get_api_version(lua) == 1) {
            match self.clone() {
                Self::LoadFas(pid, pkg) => {
                    do_callback(extension, lua, "load_fas", (pid, pkg));
                }
                Self::UnloadFas(pid, pkg) => {
                    do_callback(extension, lua, "unload_fas", (pid, pkg));
                }
                Self::StartFas => {
                    do_callback(extension, lua, "start_fas", ());
                }
                Self::StopFas => {
                    do_callback(extension, lua, "stop_fas", ());
                }
                Self::InitCpuFreq => {
                    do_callback(extension, lua, "init_cpu_freq", ());
                }
                Self::ResetCpuFreq => {
                    do_callback(extension, lua, "reset_cpu_freq", ());
                }
            }
        }
    }
}

pub fn set_policy_freq_offset(policy: i32, offset: isize) -> mlua::Result<()> {
    OFFSET_MAP
        .get()
        .unwrap()
        .get(&policy)
        .ok_or_else(|| mlua::Error::runtime("Policy Not Found!"))?
        .store(offset, Ordering::Release);
    Ok(())
}
