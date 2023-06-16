mod parse;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use fas_rs_fw::prelude::*;
use parse::*;

pub(crate) const FPSGO: &str = "/sys/kernel/fpsgo";
pub(crate) const BUFFER_CAP: usize = 512;

pub struct MtkFpsGo {
    // 缓冲区
    frametime_buffer: Arc<Mutex<Vec<FrameTime>>>,
    fps_buffer: Arc<Mutex<Vec<(Instant, Fps)>>>,
    // 控制启停
    thread_handle: [JoinHandle<()>; 2],
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

        let thread_handle = [
            thread::spawn(move || frametime_thread(frametime_clone, pause_frametime)),
            thread::spawn(move || fps_thread(fps_clone, pause_fps)),
        ];

        Ok(Self {
            frametime_buffer,
            fps_buffer,
            pause,
            thread_handle,
        })
    }

    fn frametimes(&self, count: usize) -> Vec<FrameTime> {
        let mut data = (*self.frametime_buffer.lock().unwrap()).clone();
        data.truncate(count);
        data
    }

    fn fps(&self, time: Duration) -> Vec<Fps> {
        let mut data = (*self.fps_buffer.lock().unwrap()).clone();

        let now = Instant::now();
        if let Some(pos) = data.iter().position(|(stamp, _)| now - *stamp >= time) {
            data.truncate(pos);
        }
        data.into_iter().map(|(_, fps)| fps).collect()
    }

    fn pause(&self) -> Result<(), Box<dyn Error>> {
        self.pause.store(true, Ordering::Release);
        Ok(())
    }

    fn resume(&self) -> Result<(), Box<dyn Error>> {
        enable_fpsgo()?;

        for handle in &self.thread_handle {
            handle.thread().unpark();
        }

        Ok(())
    }
}

#[inline]
pub(crate) fn enable_fpsgo() -> Result<(), std::io::Error> {
    fs::write(Path::new(FPSGO).join("common/fpsgo_enable"), "1")
}
