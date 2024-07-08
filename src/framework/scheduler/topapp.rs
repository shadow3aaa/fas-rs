// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::{Duration, Instant};

use dumpsys_rs::Dumpsys;

const REFRESH_TIME: Duration = Duration::from_secs(1);

struct Insider {
    windows_dumper: Dumpsys,
    cache: Vec<i32>,
    last_refresh: Instant,
}

impl Insider {
    pub fn new() -> Self {
        Self {
            windows_dumper: Dumpsys::new("window").unwrap(),
            cache: Vec::new(),
            last_refresh: Instant::now(),
        }
    }

    pub fn pids(&mut self) -> &Vec<i32> {
        if self.last_refresh.elapsed() > REFRESH_TIME {
            self.cache = self.get_top_pids().unwrap_or_default();
            self.last_refresh = Instant::now();
        }

        &self.cache
    }

    fn get_top_pids(&self) -> Option<Vec<i32>> {
        let dump = self.windows_dumper.dump(&["visible-apps"]).ok()?;
        Some(Self::parse_top_app(&dump))
    }

    fn parse_top_app(dump: &str) -> Vec<i32> {
        dump.lines()
            .filter(|l| l.contains("Session{"))
            .filter_map(|l| l.split_whitespace().nth(3))
            .filter_map(|s| s.split(':').next())
            .map(|p| p.trim().parse().unwrap())
            .collect()
    }
}

pub struct TimedWatcher {
    insider: Insider,
}

impl TimedWatcher {
    pub fn new() -> Self {
        Self {
            insider: Insider::new(),
        }
    }

    pub fn is_topapp(&mut self, pid: i32) -> bool {
        self.insider.pids().contains(&pid)
    }

    #[allow(dead_code)]
    pub fn top_apps(&mut self) -> impl Iterator<Item = i32> + '_ {
        self.insider.pids().iter().copied()
    }
}
