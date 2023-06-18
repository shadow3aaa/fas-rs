mod controller;
mod sensor;

use std::error::Error;
use std::thread;

use fas_rs_fw::prelude::*;
use fas_rs_fw::Scheduler;

use controller::cpu_common::CpuCommon;
use sensor::mtk_fpsgo::MtkFpsGo;

fn main() -> Result<(), Box<dyn Error>> {
    let scheduler = Scheduler::new(Box::new(MtkFpsGo::new()?), Box::new(CpuCommon::new()?))?;
    scheduler.load(60)?;

    thread::park();

    Ok(())
}
