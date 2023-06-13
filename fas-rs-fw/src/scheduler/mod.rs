//! [`self::Scheduler]通过[`crate::VirtualFrameSensor`]和[`crate::VirtualPerformanceController`]来进行调度
//! 提供一个帧感知调度基本逻辑和一些api

mod frame;

use std::error::Error;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use crate::{Fps, FrameTime, TargetFps};
use crate::{VirtualFrameSensor, VirtualPerformanceController};

pub struct Scheduler {
    sender: SyncSender<Command>,
}

enum Command {
    Unload,
    Load(TargetFps),
    Stop,
}

impl Scheduler {
    /// 构造一个[`self::Scheduler`]
    /// 并且初始化
    pub fn new(
        sensor: Box<dyn VirtualFrameSensor>,
        controller: Box<dyn VirtualPerformanceController>,
    ) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = mpsc::sync_channel(1);
        thread::spawn(move || Self::run(sensor, controller, rx));

        Ok(Self { sender: tx })
    }

    /// 卸载
    pub fn unload(&self) -> Result<(), Box<dyn Error>> {
        self.sender.try_send(Command::Unload)?;
        Ok(())
    }

    /// 载入
    /// 如果已经载入，再次调用会重载
    /// 每次载入/重载要指定新的[`crate::TargetFps`]
    pub fn load(&self, target: TargetFps) -> Result<(), Box<dyn Error>> {
        self.sender.try_send(Command::Load(target))?;
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn Error>> {
        self.sender.try_send(Command::Stop)?;
        Ok(())
    }
}

impl Scheduler {
    fn run(
        sensor: Box<dyn VirtualFrameSensor>,
        controller: Box<dyn VirtualPerformanceController>,
        receiver: Receiver<Command>,
    ) {
        let mut loaded = false;
        loop {
            match receiver.recv().unwrap() {
                Command::Stop => return,
                Command::Unload => {
                    if loaded {
                        loaded = false;
                        Self::process_unload();
                    }
                }
                Command::Load(fps) => {
                    loaded = true;
                    Self::process_load(&sensor, &controller, fps);
                }
            }
        }
    }
}
