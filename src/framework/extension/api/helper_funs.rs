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

use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Context;
#[cfg(debug_assertions)]
use log::debug;
use log::warn;

use crate::cpu_common::{
    extra_policy::{AbsRangeBound, ExtraPolicy, RelRangeBound},
    EXTRA_POLICY_MAP, IGNORE_MAP,
};

static WARNING_FLAG: AtomicBool = AtomicBool::new(false);

pub fn remove_extra_policy(policy: i32) {
    *EXTRA_POLICY_MAP
        .get()
        .context("EXTRA_POLICY_MAP not initialized")
        .unwrap()
        .get(&policy)
        .context("CPU Policy not found")
        .unwrap()
        .lock() = ExtraPolicy::None;
}

pub fn set_extra_policy_abs(policy: i32, min: Option<isize>, max: Option<isize>) {
    let extra_policy = if min.is_none() && max.is_none() {
        ExtraPolicy::None
    } else {
        ExtraPolicy::AbsRangeBound(AbsRangeBound { min, max })
    };

    *EXTRA_POLICY_MAP
        .get()
        .context("EXTRA_POLICY_MAP not initialized")
        .unwrap()
        .get(&policy)
        .context("CPU Policy not found")
        .unwrap()
        .lock() = extra_policy;

    #[cfg(debug_assertions)]
    debug!("EXTRA_POLICY_MAP: {:?}", EXTRA_POLICY_MAP.get().unwrap());
}

pub fn set_extra_policy_rel(
    policy: i32,
    target_policy: i32,
    min: Option<isize>,
    max: Option<isize>,
) {
    let extra_policy = if min.is_none() && max.is_none() {
        ExtraPolicy::None
    } else {
        ExtraPolicy::RelRangeBound(RelRangeBound {
            min,
            max,
            rel_to: target_policy,
        })
    };

    *EXTRA_POLICY_MAP
        .get()
        .context("EXTRA_POLICY_MAP not initialized")
        .unwrap()
        .get(&policy)
        .context("CPU Policy not found")
        .unwrap()
        .lock() = extra_policy;

    #[cfg(debug_assertions)]
    debug!("EXTRA_POLICY_MAP: {:?}", EXTRA_POLICY_MAP.get().unwrap());
}

pub fn set_policy_freq_offset(_: i32, _: isize) {
    if !WARNING_FLAG.load(Ordering::Acquire) {
        warn!("The API set_policy_freq_offset was removed in v4.2.0. If you see this warning, it means an outdated plugin is trying to use it. The warning will only appear once.");
        WARNING_FLAG.store(true, Ordering::Release);
    }
}

pub fn set_ignore_policy(policy: i32, val: bool) {
    IGNORE_MAP
        .get()
        .unwrap()
        .get(&policy)
        .ok_or_else(|| mlua::Error::runtime("Policy Not Found!"))
        .unwrap()
        .store(val, Ordering::Release);
}
