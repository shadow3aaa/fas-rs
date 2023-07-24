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
    cmp::{self, Ordering as CmpOrdering},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use fas_rs_fw::write_pool::WritePool;

use atomic::{Atomic, Ordering};
use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;
use log::debug;
use touch_event::TouchListener;

use crate::config::CONFIG;

const BURST_DEFAULT: usize = 0;
const BURST_MAX: usize = 2;

pub struct Schedule {
    path: PathBuf,
    pub target_diff: Arc<Atomic<Cycles>>,
    pub cur_cycles: Arc<Atomic<Cycles>>,
    touch_listener: TouchListener,
    touch_timer: Instant,
    burst: usize,
    pool: WritePool,
    table: Vec<Cycles>,
    pos: usize,
}

impl Schedule {
    pub fn new(path: &Path) -> Self {
        let target_diff = Arc::new(Atomic::new(Cycles::from_mhz(200)));

        let count = fs::read_to_string(path.join("affected_cpus"))
            .unwrap()
            .split_whitespace()
            .count();
        let pool = WritePool::new(cmp::max(count / 2, 2));

        let mut table: Vec<Cycles> = fs::read_to_string(path.join("scaling_available_frequencies"))
            .unwrap()
            .split_whitespace()
            .map(|freq| Cycles::from_khz(freq.parse().unwrap()))
            .collect();

        table.sort_unstable();

        let cur_cycles = Arc::new(Atomic::new(table.last().copied().unwrap()));

        debug!("Got cpu freq table: {:#?}", &table);

        let pos = table.len() - 1;

        Self {
            path: path.to_owned(),
            target_diff,
            cur_cycles,
            touch_listener: TouchListener::new().unwrap(),
            touch_timer: Instant::now(),
            burst: BURST_DEFAULT,
            pool,
            table,
            pos,
        }
    }

    pub fn run(&mut self, diff: Cycles) {
        if diff < Cycles::new(0) {
            return;
        }

        let max = self.table[self.pos];
        self.cur_cycles.store(max, Ordering::Release);

        let target_diff = self.target_diff.load(Ordering::Acquire);
        let target_diff = target_diff.min(self.cur_cycles.load(Ordering::Acquire));

        assert!(
            target_diff.as_hz() >= 0,
            "Target diff should never be less than zero, but got {target_diff}"
        );

        match target_diff.cmp(&diff) {
            CmpOrdering::Less => {
                self.pos = self.pos.saturating_sub(1);
                self.burst = BURST_DEFAULT;
            }
            CmpOrdering::Greater => {
                self.pos = cmp::min(self.pos + self.burst, self.table.len() - 1);
                self.burst = cmp::min(BURST_MAX, self.burst + 1);
            }
            CmpOrdering::Equal => self.burst = BURST_DEFAULT,
        }

        self.write();
    }

    pub fn reset(&mut self) {
        self.burst = 0;
        self.pos = self.table.len() - 1;
        self.write();
    }

    fn write(&mut self) {
        let touch_boost = CONFIG
            .get_conf("touch_boost")
            .and_then_likely(|b| b.as_integer())
            .unwrap();
        let touch_boost = usize::try_from(touch_boost).unwrap();

        let slide_boost = CONFIG
            .get_conf("slide_boost")
            .and_then_likely(|b| b.as_integer())
            .unwrap();
        let slide_boost = usize::try_from(slide_boost).unwrap();

        let slide_timer = CONFIG
            .get_conf("slide_timer")
            .and_then_likely(|t| t.as_integer())
            .unwrap();
        let slide_timer = Duration::from_millis(slide_timer.try_into().unwrap());

        let status = self.touch_listener.status();
        let pos = if status.0 || self.touch_timer.elapsed() <= slide_timer {
            self.touch_timer = Instant::now();
            self.pos + slide_boost
        } else if status.1 {
            self.pos + touch_boost
        } else {
            self.pos
        };
        let pos = pos.min(self.table.len() - 1);

        let _ = self.pool.write(
            &self.path.join("scaling_max_freq"),
            &self.table[pos].as_khz().to_string(),
        );
    }
}
