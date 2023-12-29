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

use std::{fs, process, thread};

use fas_rs_fw::prelude::*;

use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};

#[cfg(debug_assertions)]
use log::debug;

use cpu_common::CpuCommon;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/data/media/0/Android/fas-rs/games.toml")]
    local_profile: String,
    #[arg(short, long, default_value = "/data/adb/modules/fas_rs/games.toml")]
    std_profile: String,
    #[arg(short, long)]
    run: bool,
    #[arg(short, long)]
    merge: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let local_path = args.local_profile;
    let std_path = args.std_profile;

    if args.merge {
        let local = fs::read_to_string(&local_path)?;
        let std = fs::read_to_string(&std_path)?;

        let new = Config::merge(&local, &std).unwrap_or(std);
        print!("{new}");

        return Ok(());
    }

    if args.run {
        run(std_path, local_path).unwrap_or_else(|e| error!("{e:?}"));
        panic!("An unrecoverable error occurred!");
    }

    Ok(())
}

fn run<S: AsRef<str>>(std_path: S, local_path: S) -> Result<()> {
    let std_path = std_path.as_ref();
    let local_path = local_path.as_ref();

    pretty_env_logger::init_custom_env("FAS_LOG");

    let self_pid = process::id();
    let _ = fs::write("/dev/cpuset/background/cgroup.procs", self_pid.to_string());

    let config = Config::new(&local_path, &std_path)?;
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
