#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
mod config;
mod controller;
mod sensor;

use std::{env, process, thread};

use fas_rs_fw::{prelude::*, support_controller, support_sensor, Scheduler};

use likely_stable::if_unlikely;
use log::{debug, info};
use pretty_env_logger::init_custom_env;

use config::CONFIG;
use controller::cpu_common::CpuCommon;
use sensor::mtk_fpsgo::MtkFpsGo;

fn main() -> ! {
    // 初始化Log
    init_custom_env("FAS_LOG");
    info!("Log initialized");

    // 绑定到小核
    set_self_sched();
    info!("Self sched setted");

    // 搜索列表中第一个支持的控制器和传感器，并且构造
    // 没有支持的就退出程序
    #[allow(unused_variables)]
    let controller = support_controller!(CpuCommon).unwrap();
    info!("Got supported controller");
    #[allow(unused_variables)]
    let sensor = support_sensor!(MtkFpsGo).unwrap();
    info!("Got supported sensor");

    // 如果是测试支持模式这里就退出
    let mut args = env::args();
    if Some("test") == args.nth(1).as_deref() {
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
