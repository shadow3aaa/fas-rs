// Copyright 2023-2024, shadow3 (@shadow3aaa)
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

use std::sync::atomic::Ordering;

use crate::cpu_common::{IGNORE_MAP, OFFSET_MAP};

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
