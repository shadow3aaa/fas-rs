// Copyright 2025-2025, shadow3aaa
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

use libc::pid_t;

use super::{
    super::core::ExtensionMap,
    misc::{do_callback, get_api_version},
    Api,
};

#[derive(Debug, Clone)]
pub enum ApiV3 {
    LoadFas(pid_t, String),
    UnloadFas(pid_t, String),
    StartFas,
    StopFas,
    InitCpuFreq,
    ResetCpuFreq,
    TargetFpsChange(u32, String),
}

impl Api for ApiV3 {
    fn handle_api(&self, ext: &ExtensionMap) {
        for (extension, lua) in ext.iter().filter(|(_, lua)| get_api_version(lua) == 3) {
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
                Self::TargetFpsChange(target_fps, pkg) => {
                    do_callback(extension, lua, "target_fps_change", (target_fps, pkg));
                }
            }
        }
    }
}
