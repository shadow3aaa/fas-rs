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

pub mod helper_funs;
pub mod misc;
pub mod v0;
pub mod v1;
pub mod v2;
pub mod v3;

use super::{core::ExtensionMap, Extension};
pub use v0::ApiV0;
use v1::ApiV1;
use v2::ApiV2;
use v3::ApiV3;

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
}

pub fn trigger_reset_cpu_freq(extension: &Extension) {
    extension.trigger_extentions(ApiV0::ResetCpuFreq);
    extension.trigger_extentions(ApiV1::ResetCpuFreq);
    extension.trigger_extentions(ApiV2::ResetCpuFreq);
    extension.trigger_extentions(ApiV3::ResetCpuFreq);
}

pub fn trigger_load_fas(extension: &Extension, pid: i32, pkg: String) {
    extension.trigger_extentions(ApiV0::LoadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV1::LoadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV2::LoadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV3::LoadFas(pid, pkg));
}

pub fn trigger_unload_fas(extension: &Extension, pid: i32, pkg: String) {
    extension.trigger_extentions(ApiV0::UnloadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV1::UnloadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV2::UnloadFas(pid, pkg.clone()));
    extension.trigger_extentions(ApiV3::UnloadFas(pid, pkg));
}

pub fn trigger_start_fas(extension: &Extension) {
    extension.trigger_extentions(ApiV0::StartFas);
    extension.trigger_extentions(ApiV1::StartFas);
    extension.trigger_extentions(ApiV2::StartFas);
    extension.trigger_extentions(ApiV3::StartFas);
}

pub fn trigger_stop_fas(extension: &Extension) {
    extension.trigger_extentions(ApiV0::StopFas);
    extension.trigger_extentions(ApiV1::StopFas);
    extension.trigger_extentions(ApiV2::StopFas);
    extension.trigger_extentions(ApiV3::StopFas);
}

pub fn trigger_target_fps_change(extension: &Extension, target_fps: u32, pkg: String) {
    extension.trigger_extentions(ApiV2::TargetFpsChange(target_fps, pkg.clone()));
    extension.trigger_extentions(ApiV3::TargetFpsChange(target_fps, pkg));
}
