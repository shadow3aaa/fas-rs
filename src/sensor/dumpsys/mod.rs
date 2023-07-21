mod dump;
mod fps;

use std::{
    cell::Cell,
    path::Path,
    sync::{
        atomic::AtomicU32,
        mpsc::{self, SyncSender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use fas_rs_fw::prelude::*;

use atomic::{Atomic, Ordering};

use super::IgnoreFrameTime;

#[derive(Clone, Copy)]
pub enum ThreadCommand {
    Time(Duration),
    Pause,
    Exit,
}

pub struct DumpSys {
    command: Arc<Atomic<ThreadCommand>>,
    count: Cell<u32>,
    avg_fps: Arc<AtomicU32>,
    handle: JoinHandle<()>,
    sync: SyncSender<()>,
    ignore: IgnoreFrameTime,
}

impl Drop for DumpSys {
    fn drop(&mut self) {
        self.command.store(ThreadCommand::Exit, Ordering::Release);
        self.handle.thread().unpark();
    }
}

impl VirtualFrameSensor for DumpSys {
    fn support() -> bool
    where
        Self: Sized,
    {
        Path::new("/system/bin/dumpsys").exists() && Path::new("/system/bin/service").exists()
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let command = Arc::new(Atomic::new(ThreadCommand::Pause));
        let avg_fps = Arc::new(AtomicU32::new(0));
        let (sx, rx) = mpsc::sync_channel(1);

        let handle = {
            let command = command.clone();
            let avg_fps = avg_fps.clone();
            thread::Builder::new()
                .name("FpsListener".into())
                .spawn(move || Self::thread(&command, &avg_fps, &rx))
        }
        .unwrap();

        Ok(Self {
            command,
            count: Cell::new(0),
            avg_fps,
            handle,
            sync: sx,
            ignore: IgnoreFrameTime::new(),
        })
    }

    fn frametimes(&self, target_fps: TargetFps) -> Vec<FrameTime> {
        self.dump_frametimes(target_fps)
    }

    fn fps(&self) -> Fps {
        let _ = self.sync.try_send(());
        self.avg_fps.load(Ordering::Acquire)
    }

    fn pause(&self) -> Result<(), Box<dyn Error>> {
        self.command.store(ThreadCommand::Pause, Ordering::Release);
        Ok(())
    }

    fn resume(&self, frame_windows: u32, fps_time: Duration) -> Result<(), Box<dyn Error>> {
        self.count.set(frame_windows);
        self.command
            .store(ThreadCommand::Time(fps_time), Ordering::Release);
        self.handle.thread().unpark();
        Ok(())
    }
}
