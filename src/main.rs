mod config;
mod controller;
mod sensor;

use std::{path::PathBuf, str::FromStr, thread};

use fas_rs_fw::prelude::*;
use fas_rs_fw::Scheduler;

use config::Config;
use controller::cpu_common::CpuCommon;
use sensor::mtk_fpsgo::MtkFpsGo;

fn main() -> Result<(), Box<dyn Error>> {
    set_self_sched();

    let scheduler = Scheduler::new(Box::new(MtkFpsGo::new()?), Box::new(CpuCommon::new()?))?;

    let config = Config::new(PathBuf::from_str("/sdcard/games.txt")?);
    let mut temp = None;
    loop {
        let current = config.cur_game_fps();

        if temp != current {
            temp = current;

            if let Some((_, fps)) = temp {
                scheduler.load(fps)?;
            } else {
                scheduler.unload()?;
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn set_self_sched() {
    let self_pid = &std::process::id().to_string();
    let _ = std::fs::write("/dev/cpuset/background/tasks", self_pid);
}
