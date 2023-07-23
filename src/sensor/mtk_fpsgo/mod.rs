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
mod parse;

use std::{
    fs::{self, set_permissions},
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
};

use fas_rs_fw::prelude::*;

use atomic::Atomic;

use super::IgnoreFrameTime;
use parse::{fps_thread, frametime_thread};

pub const FPSGO: &str = "/sys/kernel/fpsgo";

pub struct MtkFpsGo {
    // 数据量
    target_frametime_count: Arc<AtomicU32>,
    fps_time: Arc<Atomic<Duration>>,
    // 数据
    frametime_receiver: Receiver<Vec<FrameTime>>,
    avg_fps: Arc<AtomicU32>,
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

        // 数据
        let (frametime_sender, frametime_receiver) = mpsc::sync_channel(1);
        let avg_fps = Arc::new(AtomicU32::new(0));

        // 数据量
        let target_frametime_count = Arc::new(AtomicU32::new(0));
        let fps_time = Arc::new(Atomic::new(Duration::default()));

        let thread_handle = {
            let count = target_frametime_count.clone();
            let time = fps_time.clone();
            let avg_fps = avg_fps.clone();

            let pause_frametime = pause.clone();
            let pause_fps = pause.clone();

            [
                thread::Builder::new()
                    .name("FrameTimeListenerThread".into())
                    .spawn(move || {
                        frametime_thread(&frametime_sender, &count, &pause_frametime);
                    })
                    .unwrap(),
                thread::Builder::new()
                    .name("FpsListenerThread".into())
                    .spawn(move || fps_thread(&avg_fps, &time, &pause_fps))
                    .unwrap(),
            ]
        };

        Ok(Self {
            frametime_receiver,
            avg_fps,
            target_frametime_count,
            fps_time,
            ignore: IgnoreFrameTime::new(),
            pause,
            thread_handle,
        })
    }

    fn frametimes(&self, target_fps: TargetFps) -> Vec<FrameTime> {
        let Some(data) = self.frametime_receiver.try_iter().last() else {
            return Vec::default();
        };

        data.into_iter()
            .map(|frametime| self.ignore.ign(frametime, target_fps))
            .collect()
    }

    fn fps(&self) -> Fps {
        self.avg_fps.load(Ordering::Acquire)
    }

    fn pause(&self) -> Result<(), Box<dyn Error>> {
        disable_fpsgo()?;

        self.pause.store(true, Ordering::Release);
        Ok(())
    }

    fn resume(&self, frametime_count: u32, fps_time: Duration) -> Result<(), Box<dyn Error>> {
        enable_fpsgo()?;

        self.pause.store(false, Ordering::Release);
        self.target_frametime_count
            .store(frametime_count, Ordering::Release);
        self.fps_time.store(fps_time, Ordering::Release);

        self.thread_handle
            .iter()
            .for_each(|handle| handle.thread().unpark());

        Ok(())
    }
}

pub fn enable_fpsgo() -> Result<(), std::io::Error> {
    let path = Path::new(FPSGO).join("common/fpsgo_enable");
    set_permissions(&path, PermissionsExt::from_mode(0o644))?;
    fs::write(&path, "1")
}

fn disable_fpsgo() -> Result<(), std::io::Error> {
    let path = Path::new(FPSGO).join("common/fpsgo_enable");
    let _ = set_permissions(&path, PermissionsExt::from_mode(0o644));
    fs::write(&path, "0")
}
