mod frame;

use std::error::Error;
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::{Fps, FrameTime};
use crate::{VirtualFrameSensor, VirtualPerformanceController};

pub struct Scheduler {
    sensor: &'a dyn VirtualFrameSensor,
    controller: &'a dyn VirtualPerformanceController,
    sender: Sender<Command>
}

type TargetFps = u32;
enum Command {
    Pause,
    Resume(TargetFps),
    Kill,
}

// 控制部分
impl Scheduler {
    pub fn new(
        sensor: Box<dyn VirtualFrameSensor>,
        controller: Box<dyn VirtualPerformanceController>,
    ) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = mpsc::sync_channel(1);
    
        thread::spawn(||)?;
        Self { sender: tx }
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
impl Scheduler {
    fn run(sensor: Box<dyn VirtualFrameSensor>,
        controller: Box<dyn VirtualPerformanceController>,
        receiver: Receiver<Command>
    ) {
        
    }
}
