/* Copyright 2023 shadow3aaa@gitbub.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License. */
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
mod config;
mod controller;
mod sensor;

use std::{env, fs, process, thread};

use fas_rs_fw::{prelude::*, support_controller, support_sensor, Scheduler};

use likely_stable::if_unlikely;
use log::{debug, error, info, warn};
use pretty_env_logger::init_custom_env;

use config::CONFIG;
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
    if args.nth(1).as_deref() == Some("merge") {
        info!("Merging config");

        let (Some(conf_local_path), Some(conf_std_path)) = (args.next(), args.next()) else {
        error!("Missing configuration path parameter");
        error!("Example: fas-rs merge local_config_path std_config_path");

        process::exit(1);
    };

        let conf_local = fs::read_to_string(&conf_local_path).unwrap();
        let conf_std = fs::read_to_string(conf_std_path).unwrap();

        let new_conf = config::merge(&conf_local, &conf_std).unwrap();

        fs::write(&conf_local_path, new_conf).unwrap();
        process::exit(0);
    }

    // 搜索列表中第一个支持的控制器和传感器
    let controller = support_controller!(CpuCommon).unwrap();
    info!("Got supported controller");

    let sensor = support_sensor!(MtkFpsGo, DumpSys).unwrap();
    info!("Got supported sensor");

    // Test mode
    if env::args().nth(1).as_deref() == Some("test") {
        info!("On test mod, supported");
        process::exit(0);
    }

    let scheduler = Scheduler::new(sensor, controller).unwrap();
    info!("Scheduler started");

    let mut temp = None;
    loop {
        let current = CONFIG.cur_game_fps();

        #[allow(unused_variables)]
        if temp != current {
            temp = current;
            if_unlikely! {
                let Some((ref game, fps_frame_windows)) = &temp => {
                    let fps = fps_frame_windows[0];
                    let frame_windows = fps_frame_windows[1];
                    scheduler.load(fps, frame_windows);
                    debug!("Loaded {} {}", game, fps);
                } else {
                    scheduler.unload();
                    debug!("Unloaded");
                }
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn set_self_sched() {
    let self_pid = std::process::id();
    let _ = std::fs::write("/dev/cpuset/background/tasks", self_pid.to_string());
}
