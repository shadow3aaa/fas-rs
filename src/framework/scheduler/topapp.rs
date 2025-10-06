// Copyright 2024-2025, shadow3aaa
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
        let Some(focused_app_line) = dump
            .lines()
            .find(|line| line.trim().starts_with("mFocusedApp="))
        else {
            return Vec::new();
        };
        let Some(package_name) = Self::extract_package_name(focused_app_line) else {
            return Vec::new();
        };

        // Try modern parser, if it fails, fall back to legacy parser.
        let pid = Self::parse_a16_format(dump, package_name)
            .or_else(|| Self::parse_a15_format(dump, package_name));

        pid.map_or_else(Vec::new, |p| vec![p])
    }

    fn extract_package_name(line: &str) -> Option<&str> {
        line.split_whitespace()
            .find(|p| p.contains('/'))?
            .split('/')
            .next()
    }

    // Modern Parser (Android 16+)
    // Parses the PID from the `WINDOW MANAGER WINDOWS` section.
    fn parse_a16_format(dump: &str, package_name: &str) -> Option<i32> {
        let mut in_target_window_section = false;
        for line in dump.lines() {
            if in_target_window_section {
                if line.contains("mSession=") {
                    let session_part = line.split("mSession=").nth(1)?;
                    let content_start = session_part.find('{')? + 1;
                    let content_end = session_part.find('}')?;
                    let content = &session_part[content_start..content_end];
                    let pid_part = content.split_whitespace().nth(1)?;
                    let pid_str = pid_part.split(':').next()?;
                    return pid_str.parse::<i32>().ok();
                }

                if line.contains("Window #") {
                    return None;
                }
            } else if line.contains("Window #") && line.contains(package_name) {
                in_target_window_section = true;
            }
        }
        None
    }

    // Legacy Parser (Android 15 and older)
    // Parses the PID from the `WINDOW MANAGER SESSIONS` section.
    fn parse_a15_format(dump: &str, package_name: &str) -> Option<i32> {
        let mut last_pid_found: Option<i32> = None;
        for line in dump.lines() {
            if line.starts_with("  Session Session{") {
                let content_start = line.find('{')? + 1;
                let content_end = line.find('}')?;
                let content = &line[content_start..content_end];
                let pid_part = content.split_whitespace().nth(1)?;
                let pid_str = pid_part.split(':').next()?;
                last_pid_found = pid_str.parse::<i32>().ok();
            }

            let trimmed_line = line.trim();
            if trimmed_line.starts_with("mPackageName=")
                && let Some(pkg) = trimmed_line.split('=').nth(1)
                && pkg == package_name
            {
                return last_pid_found;
            }
        }
        None
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
                        log::error!("Failed to dump windows: {e}, retrying");
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
