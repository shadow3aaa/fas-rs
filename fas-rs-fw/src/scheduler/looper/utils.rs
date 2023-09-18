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
use std::{collections::hash_map::Entry, time::Duration};

use super::{super::FasData, Looper};
use crate::PerformanceController;

impl<P: PerformanceController> Looper<P> {
    /* 检查是否为顶层应用，并且删除不是顶层应用的buffer **/
    pub fn retain_topapp(&mut self) {
        self.buffers
            .retain(|(_, p), _| self.topapp_checker.is_topapp(*p).unwrap_or(false));
    }

    pub fn buffer_update(&mut self, d: &FasData) {
        if d.frametime.is_zero() {
            return;
        } else if d.target_fps == 0 {
            panic!("Target fps must be bigger than zero");
        }

        let process = (d.pkg.clone(), d.pid);
        let scale_time = Duration::from_secs(1)
            .checked_div(d.target_fps)
            .unwrap_or_default()
            .as_nanos() as isize;
        let jank_time = d.frametime.as_nanos() as isize - scale_time;

        match self.buffers.entry(process) {
            Entry::Occupied(mut o) => {
                let value = o.get_mut();
                if value.0 == scale_time {
                    value.1 += jank_time;
                    value.1 = value.1.max(-value.0);
                } else {
                    value.0 = scale_time;
                    value.1 = 0;
                }
            }
            Entry::Vacant(v) => {
                v.insert((scale_time, jank_time));
            }
        }
    }
}
