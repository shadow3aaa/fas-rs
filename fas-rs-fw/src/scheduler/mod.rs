//! 提供一个帧感知调度基本逻辑和一些api

mod frame;

use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use likely_stable::likely;

use crate::{this_unwrap::ThisResult, TargetFps, VirtualFrameSensor, VirtualPerformanceController};

/// [`self::Scheduler`]通过[`crate::VirtualFrameSensor`]和[`crate::VirtualPerformanceController`]来进行调度
pub struct Scheduler {
    sender: Sender<Command>,
    stop: Arc<AtomicBool>,
}

enum Command {
    Load(TargetFps),
    Unload,
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

impl Scheduler {
    /// 构造一个[`self::Scheduler`]并且初始化
    ///
    /// # Errors
    /// 
    /// 暂停控制器/传感器失败
    pub fn new(
        sensor: Box<dyn VirtualFrameSensor>,
        controller: Box<dyn VirtualPerformanceController>,
    ) -> Result<Self, Box<dyn Error>> {
        sensor.pause()?;
        controller.plug_out()?;

        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();

        let _ = thread::Builder::new()
            .name("SchedulerThread".into())
            .spawn(move || Self::run(&*sensor, &*controller, &rx, &stop_clone))
            .this_unwrap();

        Ok(Self { sender: tx, stop })
    }

    /// 卸载[`self::Scheduler`]
    /// 用于临时暂停
    ///
    /// # Errors
    ///
    /// 发送消息失败(接收端退出)
    pub fn unload(&self) -> Result<(), Box<dyn Error>> {
        self.sender.send(Command::Unload)?;
        Ok(())
    }

    /// 载入[`self::Scheduler`]
    /// 如果已经载入，再次调用会重载入(调用init)
    /// 每次载入/重载要指定新的[`crate::TargetFps`]
    ///
    /// # Errors
    ///
    /// 发送消息失败(接收端退出)
    pub fn load(&self, target: TargetFps) -> Result<(), Box<dyn Error>> {
        self.sender.send(Command::Load(target))?;
        Ok(())
    }
}

impl Scheduler {
    fn run(
        sensor: &dyn VirtualFrameSensor,
        controller: &dyn VirtualPerformanceController,
        receiver: &Receiver<Command>,
        stop: &Arc<AtomicBool>,
    ) {
        let mut loaded = false;
        let mut target_fps = TargetFps::default();

        if stop.load(Ordering::Acquire) {
            return;
        }

        loop {
            if let Ok(command) = receiver.try_recv() {
                match command {
                    Command::Unload => {
                        if loaded {
                            loaded = false;
                            // init unload
                            Self::process_unload(sensor, controller).unwrap();
                            // 清空管道
                            let _ = receiver.try_iter().count();
                        }
                    }
                    Command::Load(fps) => {
                        loaded = true;
                        target_fps = fps;

                        Self::init_load(sensor, controller, target_fps).unwrap();
                    }
                }
            }

            if likely(loaded) {
                Self::process_load(sensor, controller, target_fps).unwrap();
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
