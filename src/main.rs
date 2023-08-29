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
mod controller;
mod error;

use std::{fs, process, thread};

use fas_rs_fw::prelude::*;

use anyhow::Result;
use clap::Parser;
use log::warn;
use pretty_env_logger::init_custom_env;

use controller::cpu_common::CpuCommon;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/sdcard/Android/fas-rs/games.toml")]
    local_profile: String,
    #[arg(short, long, default_value = "games.toml")]
    std_profile: String,
    #[arg(short, long)]
    run: bool,
    #[arg(short, long)]
    merge: bool,
}

fn main() -> Result<()> {
    // 初始化Log
    init_custom_env("FAS_LOG");

    // 绑定到小核
    let self_pid = process::id();
    let _ = fs::write("/dev/cpuset/background/tasks", self_pid.to_string());

    let args = Args::parse();

    if args.merge {
        let local_path = args.local_profile.clone();
        let std_path = args.std_profile.clone();

        let local = fs::read_to_string(&local_path)?;
        let std = fs::read_to_string(std_path)?;

        let new = Config::merge(&local, &std)?;

        fs::write(local_path, new)?;
    }

    if args.run {
        Node::init()?;
        let conf_path = args.local_profile;
        let config = Config::new(conf_path)?;
        let cpu = CpuCommon::new(&config)?;

        thread::Builder::new()
            .name("Cleaner".into())
            .spawn(|| clean::cleaner)?;

        Scheduler::new()
            .config(config)
            .controller(cpu)
            .jank_level_max(3)
            .start_run()?;
    }
    Ok(())
}
