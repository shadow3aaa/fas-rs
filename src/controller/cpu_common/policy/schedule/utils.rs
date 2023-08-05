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
use std::time::{Duration, Instant};

use fas_rs_fw::{config::CONFIG, node::NODE};

use atomic::Ordering;
use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;
use log::debug;

use yata::{methods::SMA, prelude::*};

use super::{Schedule, SMOOTH_COUNT};

impl Schedule {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    pub fn smooth_pos(&mut self) {
        self.smoothed_pos =
            (self.smooth.next(&(self.pos as f64)).round() as usize).min(self.table.len() - 1);
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn reset(&mut self) {
        self.burst = 0;
        self.pos = self.table.len() - 1;
        self.smooth = SMA::new(SMOOTH_COUNT, &(self.pos as f64)).unwrap();
        self.write();
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    fn freq_clamp(&self, freq: Cycles) -> Cycles {
        let max_pos_per: u8 = NODE
            .read_node("max_freq_per")
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

        let ori_pos = self.smoothed_pos;
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

        self.cur_freq.store(freq, Ordering::Release);

        let _ = self.pool.write(
            &self.path.join("scaling_max_freq"),
            &freq.as_khz().to_string(),
        );
    }
}
