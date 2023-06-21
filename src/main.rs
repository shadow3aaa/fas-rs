mod config;
mod controller;
mod sensor;

use std::{path::PathBuf, str::FromStr, thread, env};

use fas_rs_fw::{prelude::*, support_controller, support_sensor, Scheduler};

use config::Config;
use controller::cpu_common::CpuCommon;
use sensor::mtk_fpsgo::MtkFpsGo;

fn main() -> Result<(), Box<dyn Error>> {
    set_self_sched();

    let controller = support_controller!(CpuCommon);
    let sensor = support_sensor!(MtkFpsGo);
    let scheduler = Scheduler::new(sensor, controller)?;

    let config = Config::new(PathBuf::from_str("/sdcard/fas-rs/games.txt")?);
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
