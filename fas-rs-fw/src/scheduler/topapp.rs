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
use std::{
    process::Command,
    time::{Duration, Instant},
};

use likely_stable::LikelyOption;

const REFRESH_TIME: Duration = Duration::from_secs(2);

pub struct TimedWatcher {
    cache: Vec<i32>,
    last_refresh: Instant,
}

impl TimedWatcher {
    pub fn new() -> Self {
        let cache = Self::get_top_pids().unwrap_or_default();
        Self {
            cache,
            last_refresh: Instant::now(),
        }
    }

    pub fn is_topapp(&mut self, pid: i32) -> bool {
        if self.last_refresh.elapsed() > REFRESH_TIME {
            self.cache = Self::get_top_pids().unwrap_or_default();
            self.last_refresh = Instant::now();
        }

        self.cache.contains(&pid)
    }

    fn get_top_pids() -> Option<Vec<i32>> {
        let dump = Command::new("dumpsys")
            .args(["window", "visible-apps"])
            .output()
            .ok()?;
        let dump = String::from_utf8_lossy(&dump.stdout).into_owned();

        Some(Self::parse_top_app(&dump))
    }

    fn parse_top_app(dump: &str) -> Vec<i32> {
        let mut result: Vec<i32> = Vec::new();
        for l in dump.lines() {
            if l.contains("Session{") {
                if let Some(p) = l
                    .split_whitespace()
                    .nth(3)
                    .and_then_likely(|p| p.split(':').next())
                {
                    if let Ok(pid) = p.trim().parse() {
                        result.push(pid);
                    }
                }
            }
        }

        result
    }
}
