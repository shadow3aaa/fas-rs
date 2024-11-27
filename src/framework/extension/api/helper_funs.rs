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

use std::sync::atomic::{AtomicBool, Ordering};

use log::error;

use crate::cpu_common::IGNORE_MAP;

static OFFSET_DEPRECATED_MSG_PRINTED: AtomicBool = AtomicBool::new(false);

pub fn set_policy_freq_offset(_: i32, _: isize) {
    if !OFFSET_DEPRECATED_MSG_PRINTED.load(Ordering::Acquire) {
        OFFSET_DEPRECATED_MSG_PRINTED.store(true, Ordering::Release);
        error!("'set_policy_freq_offset' is deprecated and does nothing since v4.0.0.");
    }
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
