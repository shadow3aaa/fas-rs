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
//! 根据cgroup.procs和进程pid鉴定是否为顶层应用

use std::{
    collections::HashSet,
    fs,
    time::{Duration, Instant},
};

use crate::error::{Error, Result};

const REFRESH_TIME: Duration = Duration::from_secs(1);

// 定时刷新cgroup
pub struct TimedWatcher {
    cache: HashSet<i32>,
    last_refresh: Instant,
}

impl TimedWatcher {
    pub fn new() -> Result<Self> {
        let cache = Self::read_pids()?;
        Ok(Self {
            cache,
            last_refresh: Instant::now(),
        })
    }

    pub fn is_topapp(&mut self, pid: i32) -> Result<bool> {
        if self.last_refresh.elapsed() > REFRESH_TIME {
            self.cache = Self::read_pids()?;
            self.last_refresh = Instant::now();
        }

        Ok(self.cache.contains(&pid))
    }

    fn read_pids() -> Result<HashSet<i32>> {
        let mut pids = HashSet::new();

        let mut no_cpuset = false;
        fs::read_to_string("/dev/cpuset/top-app/cgroup.procs").map_or_else(
            |_| {
                no_cpuset = true;
            },
            |cpuset| {
                pids.extend(cpuset.lines().filter_map(|p| p.trim().parse::<i32>().ok()));
            },
        );

        let mut no_cpuctl = false;
        fs::read_to_string("/dev/cpuctl/top-app/cgroup.procs").map_or_else(
            |_| {
                no_cpuctl = true;
            },
            |cpuctl| {
                pids.extend(cpuctl.lines().filter_map(|p| p.trim().parse::<i32>().ok()));
            },
        );

        if no_cpuset && no_cpuctl {
            Err(Error::Other("cgroup.procs not found"))
        } else {
            Ok(pids)
        }
    }
}
