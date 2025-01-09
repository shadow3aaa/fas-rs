// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

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
        let windows_dumper = loop {
            match Dumpsys::new("window") {
                Some(d) => break d,
                None => std::thread::sleep(Duration::from_secs(1)),
            }
        };

        Self {
            windows_dumper,
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
            let dump = loop {
                match self.windows_dumper.dump(&["visible-apps"]) {
                    Ok(dump) => break dump,
                    Err(e) => {
                        log::error!("Failed to dump windows: {}, retrying", e);
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            };
            self.cache = WindowsInfo::new(&dump);

            self.last_refresh = Instant::now();
        }

        &self.cache
    }
}
