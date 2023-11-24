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
    fs::{self, OpenOptions},
    time::{Duration, Instant},
};

use binder::binder_impl::IBinderInternal;

const TOPAPP_TEMP: &str = "/dev/fas_rs_topapp_temp";
const REFRESH_TIME: Duration = Duration::from_secs(2);

pub struct TimedWatcher {
    cache: Vec<i32>,
    last_refresh: Instant,
}

impl TimedWatcher {
    pub fn new() -> Self {
        let cache = Vec::new();

        let mut result = Self {
            cache,
            last_refresh: Instant::now(),
        };

        result.get_top_pids();
        result
    }

    pub fn is_topapp(&mut self, pid: i32) -> bool {
        if self.last_refresh.elapsed() > REFRESH_TIME {
            self.get_top_pids();
            self.last_refresh = Instant::now();
        }

        self.cache.contains(&pid)
    }

    fn get_top_pids(&mut self) {
        let temp = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(TOPAPP_TEMP)
            .unwrap();

        let Some(mut service) = binder::get_service("window") else {
            return;
        };

        if service.dump(&temp, &["visible-apps"]).is_err() {
            return;
        }

        let dump = fs::read_to_string(TOPAPP_TEMP).unwrap();

        self.cache = dump
            .lines()
            .filter(|l| l.contains("Session{"))
            .filter_map(|l| l.split_whitespace().nth(3))
            .filter_map(|s| s.split(':').next())
            .map(|p| p.trim().parse().unwrap())
            .collect();
    }
}
