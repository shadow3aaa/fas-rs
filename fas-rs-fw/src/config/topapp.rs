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
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
    time::Duration,
};

use likely_stable::LikelyOption;

use super::Config;
use crate::error::Error;

impl Config {
    pub(super) fn topapp_updater(sx: &Sender<Option<Vec<String>>>, exit: &Arc<AtomicBool>) {
        let mut temp = None;

        loop {
            if exit.load(Ordering::Acquire) {
                return;
            }

            let cur = Self::get_top_pkgname();

            if temp != cur {
                temp = cur.clone();
                sx.send(cur)
                    .map_err(|_| Error::Other("Failed to send topapp"))
                    .unwrap();
            }
            thread::sleep(Duration::from_secs(2));
        }
    }

    fn get_top_pkgname() -> Option<Vec<String>> {
        let dump = Command::new("dumpsys")
            .args(["window", "visible-apps"])
            .output()
            .ok()?;
        let dump = String::from_utf8_lossy(&dump.stdout).into_owned();

        let result = Self::parse_top_app(&dump);

        if result.is_empty() {
            return None;
        }

        Some(result)
    }

    fn parse_top_app(dump: &str) -> Vec<String> {
        let mut result = Vec::new();
        for l in dump.lines() {
            if l.contains("package=") {
                if let Some(p) = l
                    .split_whitespace()
                    .nth(2)
                    .and_then_likely(|p| p.split('=').nth(1))
                {
                    result.push(p.to_string());
                }
            } else if l.contains("canReceiveKeys()=false") {
                result.pop();
            }
        }

        result
    }
}
