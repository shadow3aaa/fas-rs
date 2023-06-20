mod parse;

use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use fas_rs_fw::prelude::*;

use super::IgnoreFrameTime;
use parse::*;

pub(crate) const FPSGO: &str = "/sys/kernel/fpsgo";
pub(crate) const BUFFER_CAP: usize = 512;

pub struct MtkFpsGo {
    // 缓冲区
    frametime_buffer: Arc<Mutex<Vec<FrameTime>>>,
    fps_buffer: Arc<Mutex<Vec<(Instant, Fps)>>>,
    // 异常FrameTime忽略器
    ignore: IgnoreFrameTime,
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
            ignore: IgnoreFrameTime::new(),
            pause,
            thread_handle,
        })
    }

    fn frametimes(&self, count: usize, target_fps: TargetFps) -> Vec<FrameTime> {
        let mut data = (*self.frametime_buffer.lock().unwrap()).clone();

        data.truncate(count);
        data.into_iter()
            .map(|frametime| self.ignore.ign(frametime, target_fps))
            .collect()
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

pub(crate) fn enable_fpsgo() -> Result<(), std::io::Error> {
    use std::{fs::set_permissions, os::unix::fs::PermissionsExt};

    let path = Path::new(FPSGO).join("common/fpsgo_enable");
    set_permissions(&path, PermissionsExt::from_mode(0o644))?;
    fs::write(&path, "1")?;
    set_permissions(&path, PermissionsExt::from_mode(0o444))
}
