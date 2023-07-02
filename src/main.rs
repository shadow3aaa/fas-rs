mod config;
mod controller;
mod sensor;

use std::{env, process, thread};

pub(crate) use fas_rs_fw::debug;
use fas_rs_fw::{prelude::*, support_controller, support_sensor, Scheduler};

use config::CONFIG;
use controller::cpu_common::CpuCommon;
use mimalloc::MiMalloc;
use sensor::mtk_fpsgo::MtkFpsGo;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ! {
    set_self_sched();

    // 搜索列表中第一个支持的控制器和传感器，并且构造
    // 构造错误会panic
    // 没有支持的就退出程序
    let controller = match support_controller!(CpuCommon) {
        Ok(o) => o,
        Err(_e) => {
            println!("Unsupported");
            debug! {
                println!("{}", _e);
            }
            process::exit(1);
        }
    };
    let sensor = match support_sensor!(MtkFpsGo) {
        Ok(o) => o,
        Err(_e) => {
            println!("Unsupported");
            debug! {
                println!("reasion: {}", _e);
            }
            process::exit(1);
        }
    };

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

        if temp != current {
            temp = current;

            if let Some((ref _game, fps)) = temp {
                scheduler.load(fps).unwrap();
                debug! {
                    println!("Loaded {} {}", _game, fps);
                }
            } else {
                scheduler.unload().unwrap();
                debug! {
                    println!("Unloaded");
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
