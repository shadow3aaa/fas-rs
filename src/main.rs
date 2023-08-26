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
mod cleaner;
mod controller;

use std::{env, fs, path::Path, process};

use fas_rs_fw::{
    config,
    macros::{get_scheduler, run_modules, support},
    module::prelude::*,
    prelude::*,
    Scheduler,
};

use log::{error, info, warn};
use pretty_env_logger::init_custom_env;

use controller::cpu_common::CpuCommon;
use sensor::{dumpsys::DumpSys, mtk_fpsgo::MtkFpsGo};

fn main() -> ! {
    // 初始化Log
    init_custom_env("FAS_LOG");
    info!("Log initialized");

    // 绑定到小核
    set_self_sched();
    info!("Self sched setted");

    let mut args = env::args();
    match args.nth(1).as_deref() {
        Some("merge") => {
            info!("Merging config");

            let (Some(local_path), Some(std_path)) = (args.next(), args.next()) else {
                error!("Missing configuration path parameter");
                error!("Example: fas-rs merge local_config_path std_config_path");

                process::exit(1);
            };

            merge_config(Path::new(&local_path), Path::new(&std_path)); // exited 0 here
        }
        Some("test") => {
            if support!(CpuCommon; MtkFpsGo, DumpSys) {
                info!("On test mod, supported");
                process::exit(0);
            } else {
                error!("Not supported");
                process::exit(1);
            }
        }
        _ => (),
    }

    info!("Starting scheduler");
    let scheduler = get_scheduler!(MtkFpsGo, DumpSys; CpuCommon);

    run_modules!(
        scheduler;
        cleaner::Cleaner
    )
}

fn set_self_sched() {
    let self_pid = process::id();
    let _ = fs::write("/dev/cpuset/background/tasks", self_pid.to_string());
}

fn merge_config(local: &Path, std: &Path) -> ! {
    let conf_local = fs::read_to_string(local).unwrap();
    let conf_std = fs::read_to_string(std).unwrap();

    let new_conf = config::merge(&conf_local, &conf_std).unwrap();

    fs::write(local, new_conf).unwrap();

    process::exit(0);
}
