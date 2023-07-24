/* Copyright 2023 shadow3aaa@gitbub.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License. */
use std::{
    process::Command,
    sync::{atomic::AtomicU32, mpsc::Receiver, Arc},
    thread,
    time::Instant,
};

use atomic::{Atomic, Ordering};

use super::{DumpSys, ThreadCommand};

impl DumpSys {
    pub fn thread(
        command: &Arc<Atomic<ThreadCommand>>,
        avg_fps: &Arc<AtomicU32>,
        sync: &Receiver<()>,
    ) {
        loop {
            let time = match command.load(Ordering::Acquire) {
                ThreadCommand::Time(d) => d,
                ThreadCommand::Pause => {
                    let _ = sync.try_recv();
                    thread::park();
                    continue;
                }
                ThreadCommand::Exit => return,
            };

            let dump_and_stamp = || {
                let dump = Command::new("service")
                    .args(["call", "SurfaceFlinger", "1013"])
                    .output()
                    .ok()?;

                let dump = String::from_utf8_lossy(&dump.stdout).into_owned();
                let dump = dump
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.split('(').nth(1))
                    .unwrap();

                let dump = i32::from_str_radix(dump, 16).unwrap();
                Some((Instant::now(), dump))
            };

            let Some((time_a, stamp_a)) = dump_and_stamp() else {
                continue;
            };
            thread::sleep(time);
            let Some((time_b, stamp_b)) = dump_and_stamp() else {
                continue;
            };

            let time = time_b - time_a;
            let count = stamp_b - stamp_a;

            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_sign_loss)]
            let fps = (f64::from(count) / time.as_secs_f64()) as u32;

            avg_fps.store(fps, Ordering::Release);

            let _ = sync.recv_timeout(time); // 与scheduler同步
        }
    }
}
