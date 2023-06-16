mod parse;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use fas_rs_fw::prelude::*;
use parse::*;

pub(crate) const FPSGO: &str = "/sys/kernel/fpsgo";
pub(crate) const BUFFER_CAP: usize = 512;

pub struct MtkFpsGo {
    // 缓冲区
    frametime_buffer: Arc<Mutex<Vec<FrameTime>>>,
    fps_buffer: Arc<Mutex<Vec<Fps>>>,
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

        // 缓冲区
        let frametime_buffer = Arc::new(Mutex::new(Vec::with_capacity(BUFFER_CAP)));
        let fps_buffer = Arc::new(Mutex::new(Vec::with_capacity(BUFFER_CAP)));

        let pause_frametime = pause.clone();
        let pause_fps = pause.clone();

        let frametime_clone = frametime_buffer.clone();
        let fps_clone = fps_buffer.clone();

        let thread_handle = (
            thread::spawn(move || frametime_thread(frametime_clone, pause_frametime)),
            thread::spawn(move || fps_thread(fps_clone, pause_fps)),
        );

        Ok(Self {
            frametime_buffer,
            fps_buffer,
            pause,
            thread_handle,
        })
    }

    fn frametimes(&self, count: u32) -> Vec<FrameTime> {
        *self.frametime_buffer.lock().unwrap()
    }

    fn fps(&self, time: Duration) -> Vec<Fps> {
        *self.fps_buffer.lock().unwrap()
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
