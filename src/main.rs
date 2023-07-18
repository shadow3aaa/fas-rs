#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
mod config;
mod controller;
mod sensor;

use std::{env, process, thread};

pub use fas_rs_fw::debug;

use fas_rs_fw::{prelude::*, support_controller, support_sensor, Scheduler};

use likely_stable::if_unlikely;

use config::CONFIG;
use controller::cpu_common::CpuCommon;
use sensor::mtk_fpsgo::MtkFpsGo;

fn main() -> ! {
    // 绑定到小核
    set_self_sched();

    // 搜索列表中第一个支持的控制器和传感器，并且构造
    // 没有支持的就退出程序
    #[allow(unused_variables)]
    let controller = support_controller!(CpuCommon).unwrap();
    #[allow(unused_variables)]
    let sensor = support_sensor!(MtkFpsGo).unwrap();

    // 如果是测试支持模式这里就退出
    let mut args = env::args();
    if Some("test") == args.nth(1).as_deref() {
        println!("Supported");
        process::exit(0);
    }

    let scheduler = Scheduler::new(sensor, controller).unwrap();

    let mut temp = None;
    loop {
        let current = CONFIG.cur_game_fps();

        #[allow(unused_variables)]
        if temp != current {
            temp = current;
            if_unlikely! {
                let Some((ref game, fps)) = &temp => {
                    scheduler.load(*fps).unwrap();
                    debug! {
                        println!("Loaded {} {}", game, fps);
                    }
                } else {
                    scheduler.unload().unwrap();
                    debug! {
                        println!("Unloaded");
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn set_self_sched() {
    let self_pid = std::process::id();
    let _ = std::fs::write("/dev/cpuset/background/tasks", self_pid.to_string());
    debug! {
        println!("Self sched seted");
    }
}
