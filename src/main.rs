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

#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

mod cpu_common;
mod file_handler;
mod framework;
mod misc;

use std::{
    env, fs,
    io::{self, prelude::*},
    process,
};

use framework::prelude::*;

use anyhow::Result;
use flexi_logger::{DeferredNow, LogSpecification, Logger, Record};
use log::{error, warn};
use mimalloc::MiMalloc;

#[cfg(debug_assertions)]
use log::debug;

use cpu_common::Controller;
use misc::setprop;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const USER_CONFIG: &str = "/sdcard/Android/fas-rs/games.toml";

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    if args[1] == "merge" {
        let local = fs::read_to_string(USER_CONFIG)?;
        let std = fs::read_to_string(&args[2])?;

        let new = Config::merge(&local, &std).unwrap_or(std);
        println!("{new}");

        return Ok(());
    } else if args[1] == "run" {
        setprop("fas-rs-server-started", "true");
        run(&args[2]).unwrap_or_else(|e| {
            for cause in e.chain() {
                error!("{:#?}", cause);
            }
            error!("{:#?}", e.backtrace());
        });
    }

    Ok(())
}

fn run<S: AsRef<str>>(std_path: S) -> Result<()> {
    #[cfg(not(debug_assertions))]
    let logger_spec = LogSpecification::info();

    #[cfg(debug_assertions)]
    let logger_spec = LogSpecification::debug();

    Logger::with(logger_spec)
        .log_to_stdout()
        .format(log_format)
        .start()?;

    let std_path = std_path.as_ref();

    let self_pid = process::id();
    let _ = fs::write("/dev/cpuset/background/cgroup.procs", self_pid.to_string());

    let config = Config::new(USER_CONFIG, std_path)?;
    let cpu = Controller::new()?;

    #[cfg(debug_assertions)]
    debug!("{cpu:#?}");

    Scheduler::new()
        .config(config)
        .controller(cpu)
        .start_run()?;

    Ok(())
}

fn log_format(
    write: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record<'_>,
) -> Result<(), io::Error> {
    let time = now.format("%Y-%m-%d %H:%M:%S");
    write!(write, "[{time}] {}: {}", record.level(), record.args())
}
