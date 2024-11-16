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
