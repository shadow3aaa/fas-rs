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

pub mod misc;
pub mod v0;
pub mod v1;
pub mod v2;
pub mod v3;

use std::sync::atomic::Ordering;

use crate::cpu_common::{IGNORE_MAP, OFFSET_MAP};

use super::core::ExtensionMap;
pub use v0::ApiV0;

pub trait Api: Send {
    fn handle_api(&self, ext: &ExtensionMap);

    fn into_box(self) -> Box<dyn Api>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
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

pub fn set_ignore_policy(policy: i32, val: bool) -> mlua::Result<()> {
    IGNORE_MAP
        .get()
        .unwrap()
        .get(&policy)
        .ok_or_else(|| mlua::Error::runtime("Policy Not Found!"))?
        .store(val, Ordering::Release);
    Ok(())
}
