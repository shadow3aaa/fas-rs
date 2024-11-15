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

#[derive(Default)]
struct WindowsInfo {
    pub visible_freeform_window: bool,
    pub pids: Vec<i32>,
}

impl WindowsInfo {
    pub fn new(dump: &str) -> Self {
        let pids = Self::parse_top_app(dump);
        let visible_freeform_window = dump.contains("freeform")
            || dump.contains("FlexibleTaskCaptionView")
            || dump.contains("FlexibleTaskIndicatorView");

        Self {
            visible_freeform_window,
            pids,
        }
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

pub struct TopAppsWatcher {
    windows_dumper: Dumpsys,
    cache: WindowsInfo,
    last_refresh: Instant,
}

impl TopAppsWatcher {
    pub fn new() -> Self {
        Self {
            windows_dumper: Dumpsys::new("window").unwrap(),
            cache: WindowsInfo::default(),
            last_refresh: Instant::now(),
        }
    }

    pub fn topapp_pids(&mut self) -> &Vec<i32> {
        &self.cache().pids
    }

    pub fn visible_freeform_window(&mut self) -> bool {
        self.cache().visible_freeform_window
    }

    fn cache(&mut self) -> &WindowsInfo {
        if self.last_refresh.elapsed() > REFRESH_TIME {
            let dump = self.windows_dumper.dump(&["visible-apps"]).unwrap();
            self.cache = WindowsInfo::new(&dump);

            self.last_refresh = Instant::now();
        }

        &self.cache
    }
}
