// Copyright 2023-2025, shadow3aaa
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

use std::process::{Command, ExitStatus};

use log::warn;

pub fn resetprop<S>(k: S, v: S)
where
    S: AsRef<str>,
{
    let key = k.as_ref();
    let value = v.as_ref();
    let output = match Command::new("resetprop").args([key, value]).spawn() {
        Ok(s) => match s.wait_with_output() {
            Ok(c) => c.status,
            Err(e) => {
                warn!("cannot wait resetprop output, error: {e}");
                return;
            }
        },
        Err(e) => {
            warn!("cannot run resetprop, error: {e}");
            ExitStatus::default()
        }
    };
    if !output.success() {
        let _ = Command::new("setprop").args([key, value]).spawn();
    }
}
