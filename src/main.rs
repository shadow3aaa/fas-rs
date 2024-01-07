/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

#[cfg(not(target_os = "android"))]
#[cfg(not(target_arch = "aarch64"))]
compile_error!("Only for aarch64 android");

mod clean;
mod cpu_common;
mod error;
mod misc;

use std::{env, fs, process, thread};

use fas_rs_fw::prelude::*;

use anyhow::Result;
use flexi_logger::{LogSpecification, Logger};
use log::{error, info, warn};

#[cfg(debug_assertions)]
use log::debug;

use cpu_common::CpuCommon;

const USER_CONFIG: &str = "/data/media/0/Android/fas-rs/games.toml";

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    if args[1] == "merge" {
        let local = fs::read_to_string(USER_CONFIG)?;
        let std = fs::read_to_string(&args[2])?;

        let new = Config::merge(&local, &std).unwrap_or(std);
        println!("{new}");

        return Ok(());
    } else if args[1] == "run" {
        run(&args[2]).unwrap_or_else(|e| error!("{e:?}"));
        panic!("An unrecoverable error occurred!");
    }

    Ok(())
}

fn run<S: AsRef<str>>(std_path: S) -> Result<()> {
    #[cfg(not(debug_assertions))]
    let logger_spec = LogSpecification::info();

    #[cfg(debug_assertions)]
    let logger_spec = LogSpecification::debug();

    Logger::with(logger_spec).log_to_stdout().start()?;

    let std_path = std_path.as_ref();

    let self_pid = process::id();
    let _ = fs::write("/dev/cpuset/background/cgroup.procs", self_pid.to_string());

    let config = Config::new(USER_CONFIG, std_path)?;
    let cpu = CpuCommon::new()?;

    #[cfg(debug_assertions)]
    debug!("{cpu:#?}");

    thread::Builder::new()
        .name("CleanerThead".into())
        .spawn(clean::cleaner)?;
    info!("Cleaner thread started");

    Scheduler::new()
        .config(config)
        .controller(cpu)
        .start_run()?;

    Ok(())
}
