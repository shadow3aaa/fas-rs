/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
use std::{process::Command, time::Instant};

use fas_rs_fw::{config::CONFIG, prelude::*};
use likely_stable::LikelyOption;

use super::DumpSys;

impl DumpSys {
    pub fn get_cur_view() -> Option<String> {
        let dump = Command::new("dumpsys")
            .args(["SurfaceFlinger", "--list"])
            .output()
            .ok()?;

        let views = String::from_utf8_lossy(&dump.stdout).into_owned();

        views
            .lines()
            .find(|s| s.contains("SurfaceView[") && s.contains("BLAST"))
            .map_or_else(
                || {
                    views
                        .lines()
                        .find(|s| s.contains("SurfaceView -"))
                        .map(|view| view.trim().to_owned())
                },
                |view| Some(view.trim().to_owned()),
            )
    }

    pub fn dump_frametimes(&self, target_fps: TargetFps) -> Vec<FrameTime> {
        if self.timer.borrow().elapsed() >= Duration::from_secs(5) {
            self.view.replace(Self::get_cur_view());
            self.timer.replace(Instant::now());
        }

        let view = self.view.borrow();
        let Some(view) = view.as_ref() else {
            return Vec::new();
        };

        let take_count = self.count.get();

        let ori_data = Command::new("dumpsys")
            .args(["SurfaceFlinger", "--latency", view])
            .output()
            .unwrap();
        let ori_data = String::from_utf8_lossy(&ori_data.stdout).into_owned();

        let frametimes: Vec<_> = ori_data
            .lines()
            .skip(2)
            .filter_map(|l| l.split_whitespace().nth(1))
            .map(|v| v.parse::<u64>().unwrap())
            .filter(|v| v != &0 && v <= &10_000_000_000_000_000)
            .collect();

        let prefix = CONFIG
            .get_conf("dumpsys_prefix")
            .and_then_likely(|p| p.as_integer())
            .unwrap();

        frametimes
            .windows(2)
            .map(|ft| Duration::from_nanos(ft[1] - ft[0]))
            .rev()
            .take(take_count as usize)
            .map(|f| self.ignore.ign(f, target_fps))
            .map(|f| f + Duration::from_micros(prefix.try_into().unwrap())) // prefix
            .collect()
    }
}
