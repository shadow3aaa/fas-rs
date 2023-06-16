mod parse;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use fas_rs_fw::prelude::*;
use parse::*;

pub(crate) const FPSGO: &str = "/sys/kernel/fpsgo";

pub struct MtkFpsGo {
    // 接收FrameTime
    frametime_receiver: Receiver<Vec<FrameTime>>,
    // 指定接收个数
    frametime_count_sender: SyncSender<u32>,
    // 接收Fps
    fps_receiver: Receiver<Vec<Fps>>,
    // 指定接收时间段
    fps_time_sender: SyncSender<Duration>,
    // 控制启停
    thread_handle: (JoinHandle<()>, JoinHandle<()>),
    pause: Arc<AtomicBool>,
}

impl VirtualFrameSensor for MtkFpsGo {
    fn support() -> bool
    where
        Self: Sized,
    {
        Path::new(FPSGO).join("fbt/fbt_info").exists()
            && Path::new(FPSGO).join("common/fpsgo_enable").exists() // 检查路径
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        // 控制启停的原子bool
        let pause = Arc::new(AtomicBool::new(false));

        // 管道
        let (frametime_sender, frametime_receiver) = mpsc::sync_channel(1);
        let (frametime_count_sender, frametime_count_receiver) = mpsc::sync_channel(1);

        let (fps_sender, fps_receiver) = mpsc::sync_channel(1);
        let (fps_time_sender, fps_time_receiver) = mpsc::sync_channel(1);

        let pause_frametime = pause.clone();
        let pause_fps = pause.clone();

        let thread_handle = (
            thread::spawn(move || {
                frametime_thread(frametime_sender, frametime_count_receiver, pause_frametime)
            }),
            thread::spawn(move || fps_thread(fps_sender, fps_time_receiver, pause_fps)),
        );

        Ok(Self {
            frametime_receiver,
            frametime_count_sender,
            fps_receiver,
            fps_time_sender,
            pause,
            thread_handle,
        })
    }

    fn frametimes(&self, count: u32) -> Vec<FrameTime> {
        self.frametime_count_sender.send(count).unwrap();
        self.frametime_receiver.recv().unwrap()
    }

    fn fps(&self, time: Duration) -> Vec<Fps> {
        self.fps_time_sender.send(time).unwrap();
        self.fps_receiver.recv().unwrap()
    }

    fn pause(&self) -> Result<(), Box<dyn Error>> {
        self.pause.store(true, Ordering::Release);
        Ok(())
    }

    fn resume(&self) -> Result<(), Box<dyn Error>> {
        fs::write(Path::new(FPSGO).join("common/fpsgo_enable"), "1")?;

        self.thread_handle.0.thread().unpark();
        self.thread_handle.1.thread().unpark();

        Ok(())
    }
}
