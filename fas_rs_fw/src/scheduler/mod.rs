mod frame;

use std::error::Error;
use std::thread;
use std::sync::mpsc::sync_channel;

use crate::{Fps, FrameTime};
use crate::{VirtualFrameSensor, VirtualPerformanceController};

pub struct Scheduler<'a> {
    sensor: &'a dyn VirtualFrameSensor,
    controller: &'a dyn VirtualPerformanceController,
}

type TargetFps = u32;
enum Command {
    Pause,
    Resume(TargetFps),
}

// 控制部分
impl<'a> Scheduler<'a> {
    pub fn new(
        sensor: &'a dyn VirtualFrameSensor,
        controller: &'a dyn VirtualPerformanceController,
    ) -> Self {
        thread::spawn(||)
        Self { sensor, controller }
    }

    pub fn pause(&self) -> Result<(), Box<dyn Error>> {
        self.sensor.pause()?;
        self.controller.plug_out()?;
        Ok(())
    }

    pub fn resume(&self) -> Result<(), Box<dyn Error>> {
        self.sensor.resume()?;
        self.controller.plug_in()?;
        Ok(())
    }
}

// 逻辑部分
impl<'a> Scheduler<'a> {}
