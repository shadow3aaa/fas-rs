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

mod controller;
mod error;

use std::{env, fs, process};

use fas_rs_fw::prelude::*;

use anyhow::Result;
use clap::Parser;
use log::warn;
use pretty_env_logger::init_custom_env;

use controller::cpu_common::CpuCommon;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    local_profile: String,

    #[arg(short, long)]
    std_profile: String,
}

fn main() -> Result<()> {
    // 初始化Log
    init_custom_env("FAS_LOG");

    // 绑定到小核
    let self_pid = process::id();
    let _ = fs::write("/dev/cpuset/background/tasks", self_pid.to_string());

    let args = Args::parse();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "merge" => {
                let local_path = args.local_profile.clone();
                let std_path = args.std_profile.clone();

                let local = fs::read_to_string(&local_path)?;
                let std = fs::read_to_string(&std_path)?;

                let new = Config::merge(&local, &std)?;

                fs::write(local, new)?;
            }
            "run" => {
                let conf_path = args.local_profile.clone();
                let config = Config::new(conf_path)?;
                let cpu = CpuCommon::new(&config)?;

                Scheduler::new()
                    .config(config)
                    .controller(cpu)
                    .start_run()?;
            }
            _ => continue,
        }
    }

    Ok(())
}
