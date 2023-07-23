/* Copyright 2023 shadow3aaa@gitbub.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0

* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License. */
mod dump;
mod fps;

use std::{
    cell::{Cell, RefCell},
    path::Path,
    sync::{
        atomic::AtomicU32,
        mpsc::{self, SyncSender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
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
    view: RefCell<Option<String>>,
    timer: RefCell<Instant>,
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
            timer: RefCell::new(Instant::now()),
            view: RefCell::new(None),
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
        self.view.replace(None);
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
