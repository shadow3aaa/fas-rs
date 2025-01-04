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

pub mod helper_funs;
pub mod misc;
pub mod v0;
pub mod v1;
pub mod v2;
pub mod v3;
pub mod v4;

use super::{core::ExtensionMap, Extension};
pub use v0::ApiV0;
use v1::ApiV1;
use v2::ApiV2;
use v3::ApiV3;
use v4::ApiV4;

pub trait Api: Send {
    fn handle_api(&self, ext: &ExtensionMap);

    fn into_box(self) -> Box<dyn Api>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub fn trigger_init_cpu_freq(extension: &Extension) {
    extension.trigger_extentions(ApiV0::InitCpuFreq);
    extension.trigger_extentions(ApiV1::InitCpuFreq);
    extension.trigger_extentions(ApiV2::InitCpuFreq);
    extension.trigger_extentions(ApiV3::InitCpuFreq);
    extension.trigger_extentions(ApiV4::InitCpuFreq);
}

pub fn trigger_reset_cpu_freq(extension: &Extension) {
    extension.trigger_extentions(ApiV0::ResetCpuFreq);
    extension.trigger_extentions(ApiV1::ResetCpuFreq);
    extension.trigger_extentions(ApiV2::ResetCpuFreq);
    extension.trigger_extentions(ApiV3::ResetCpuFreq);
    extension.trigger_extentions(ApiV4::ResetCpuFreq);
}

pub fn trigger_load_fas(extension: &Extension, pid: i32, pkg: String) {
    extension.trigger_extentions(ApiV0::LoadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV1::LoadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV2::LoadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV3::LoadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV4::LoadFas(pid, pkg));
}

pub fn trigger_unload_fas(extension: &Extension, pid: i32, pkg: String) {
    extension.trigger_extentions(ApiV0::UnloadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV1::UnloadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV2::UnloadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV3::UnloadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV4::UnloadFas(pid, pkg));
}

pub fn trigger_start_fas(extension: &Extension) {
    extension.trigger_extentions(ApiV0::StartFas);
    extension.trigger_extentions(ApiV1::StartFas);
    extension.trigger_extentions(ApiV2::StartFas);
    extension.trigger_extentions(ApiV3::StartFas);
    extension.trigger_extentions(ApiV4::StartFas);
}

pub fn trigger_stop_fas(extension: &Extension) {
    extension.trigger_extentions(ApiV0::StopFas);
    extension.trigger_extentions(ApiV1::StopFas);
    extension.trigger_extentions(ApiV2::StopFas);
    extension.trigger_extentions(ApiV3::StopFas);
    extension.trigger_extentions(ApiV4::StopFas);
}

pub fn trigger_target_fps_change(extension: &Extension, target_fps: u32, pkg: String) {
    extension.trigger_extentions(ApiV2::TargetFpsChange(target_fps, pkg.clone()));
    extension.trigger_extentions(ApiV3::TargetFpsChange(target_fps, pkg.clone()));
    extension.trigger_extentions(ApiV4::TargetFpsChange(target_fps, pkg));
}
