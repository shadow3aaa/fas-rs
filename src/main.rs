mod config;
mod controller;
mod sensor;

use std::{env, path::PathBuf, str::FromStr, thread};

use fas_rs_fw::{prelude::*, support_controller, support_sensor, Scheduler};

use config::Config;
use controller::cpu_common::CpuCommon;
use sensor::mtk_fpsgo::MtkFpsGo;

fn main() -> Result<(), Box<dyn Error>> {
    set_self_sched();

    // 搜索列表中第一个支持的控制器和传感器，并且构造
    // 构造错误会panic
    // 没有支持的就退出程序
    let controller = support_controller!(CpuCommon);
    let sensor = support_sensor!(MtkFpsGo);

    // 如果是测试支持模式这里就退出
    let mut args = env::args();
    if Some("test") == args.nth(1).as_deref() {
        println!("Supported");
        return Ok(());
    }

    let scheduler = Scheduler::new(sensor, controller)?;

    let config = Config::new(PathBuf::from_str("/sdcard/Android/fas-rs/games.txt")?);
    let mut temp = None;
    loop {
        let current = config.cur_game_fps();

        if temp != current {
            temp = current;

            if let Some((ref game, fps)) = temp {
                scheduler.load(fps)?;
                println!("Loaded {} {}", game, fps);
            } else {
                scheduler.unload()?;
                println!("Unloaded");
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn set_self_sched() {
    let self_pid = &std::process::id().to_string();
    let _ = std::fs::write("/dev/cpuset/background/tasks", self_pid);
}
