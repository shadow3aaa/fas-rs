//! 提供一个帧感知调度基本逻辑和一些api

mod frame;

use std::error::Error;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::Duration;

use crate::TargetFps;
use crate::{VirtualFrameSensor, VirtualPerformanceController};

/// [`self::Scheduler]通过[`crate::VirtualFrameSensor`]和[`crate::VirtualPerformanceController`]来进行调度
pub struct Scheduler {
    sender: SyncSender<Command>,
}

enum Command {
    Load(TargetFps),
    Unload,
    Stop,
}

impl Drop for Scheduler {
    // 这个drop实现是堵塞的…
    // 不过一般来说你也不会drop它，而且send而不是try_send可以保证发送
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Stop);
    }
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

    /// 卸载[`self::Scheduler`]
    /// 用于临时暂停
    pub fn unload(&self) -> Result<(), Box<dyn Error>> {
        self.sender.try_send(Command::Unload)?;
        Ok(())
    }

    /// 载入[`self::Scheduler`]
    /// 如果已经载入，再次调用会重载
    /// 每次载入/重载要指定新的[`crate::TargetFps`]
    pub fn load(&self, target: TargetFps) -> Result<(), Box<dyn Error>> {
        self.sender.try_send(Command::Load(target))?;
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
        let mut target_fps = TargetFps::default();

        loop {
            if let Ok(command) = receiver.try_recv() {
                match command {
                    Command::Stop => return,
                    Command::Unload => {
                        if loaded {
                            loaded = false;
                            // init unload
                            Self::process_unload(&*sensor, &*controller).unwrap();
                        }
                    }
                    Command::Load(fps) => {
                        loaded = true;
                        target_fps = fps;

                        // init load
                        sensor.resume().unwrap();
                        controller.plug_in().unwrap();
                    }
                }
            }

            if loaded {
                Self::process_load(&*sensor, &*controller, target_fps).unwrap();
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
