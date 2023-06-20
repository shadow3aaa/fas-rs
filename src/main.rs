mod controller;
mod sensor;

use std::error::Error;
use std::thread;

use fas_rs_fw::prelude::*;
use fas_rs_fw::Scheduler;

use controller::cpu_common::CpuCommon;
use sensor::mtk_fpsgo::MtkFpsGo;

fn main() -> Result<(), Box<dyn Error>> {
    set_self_sched();

    let scheduler = Scheduler::new(Box::new(MtkFpsGo::new()?), Box::new(CpuCommon::new()?))?;
    scheduler.load(120)?;

    thread::park();

    Ok(())
}

fn set_self_sched() {
    let self_pid = &std::process::id().to_string();
    let _ = std::fs::write("/dev/cpuset/background/tasks", self_pid);
}
