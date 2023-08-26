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
use std::time::Instant;

use fas_rs_fw::prelude::*;

use atomic::Ordering;
use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;
use log::debug;

use yata::prelude::*;

use super::{super::super::parse_policy, Schedule};

impl Schedule {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    pub fn smooth_pos(&mut self) {
        self.smoothed_pos =
            (self.smooth.next(&(self.pos as f64)).round() as usize).min(self.table.len() - 1);
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    fn freq_clamp(&self, freq: Cycles) -> Cycles {
        let max_pos_per: u8 = Node::read_node("max_freq_per")
            .ok()
            .and_then_likely(|p| p.trim().parse().ok())
            .unwrap();
        assert!(max_pos_per <= 100, "The percentage must be less than 100%");

        let len = (self.table.len() - 1) as f64;
        let max_pos = (len * f64::from(max_pos_per) / 100.0)
            .clamp(0.0, len)
            .round() as usize;
        let max_freq = self.table[max_pos];

        self.max_diff.store(max_freq, Ordering::Release);

        debug!("Available frequency: {max_pos_per}% max freq: {max_freq}");

        freq.clamp(self.min_freq, max_freq)
    }

    pub fn write(&mut self) {
        let (touch_boost, slide_boost, slide_timer) = self.touch_conf;

        let ori_pos = self.smoothed_pos;
        let ori_freq = self.table[ori_pos];
        let ori_freq = self.freq_clamp(ori_freq);
        self.cur_freq.store(ori_freq, Ordering::Release);

        let pos = if let Some(touch_listener) = &self.touch_listener {
            let status = touch_listener.status(); // 触摸屏状态

            if status.0 || self.touch_timer.elapsed() <= slide_timer {
                self.touch_timer = Instant::now();
                ori_pos + slide_boost // on slide
            } else if status.1 {
                ori_pos + touch_boost // on touch
            } else {
                ori_pos // none
            }
        } else {
            ori_pos
        };

        let pos = pos.min(self.table.len() - 1);
        let freq = self.table[pos];
        let freq = self.freq_clamp(freq);

        if let Some(path) = &self.lock_path {
            let policy = self
                .path
                .file_name()
                .and_then_likely(|f| parse_policy(f.to_str()?))
                .unwrap();
            let lock_message = format!(
                "{policy} {} {}",
                self.table.first().unwrap().as_khz(),
                freq.as_khz()
            );

            let _ = self.pool.write(path, &lock_message);
        }

        let _ = self.pool.write(
            &self.path.join("scaling_max_freq"),
            &freq.as_khz().to_string(),
        );
    }

    // 考虑到当余量过大时，如果某个核心的Cycles小，这个核心会被迫跑高频率
    // 因此当余量 > 当前Cycles的时候，切换调速器为默认调速器
    pub fn auto_change_gov(&mut self, target_diff: Cycles, cur_cycles: Cycles) {
        let gov = if target_diff >= cur_cycles {
            &self.default_gov
        } else {
            "performance"
        };

        self.pool
            .write(&self.path.join("scaling_governor"), gov)
            .unwrap();
    }
}
