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
use std::time::Duration;

use fas_rs_fw::config::CONFIG;

use likely_stable::LikelyOption;

use super::Schedule;

impl Schedule {
    pub fn touch_boost() -> usize {
        let touch_boost = CONFIG
            .get_conf("touch_boost")
            .and_then_likely(|b| b.as_integer())
            .unwrap();
        usize::try_from(touch_boost).unwrap()
    }

    pub fn slide_boost() -> usize {
        let slide_boost = CONFIG
            .get_conf("slide_boost")
            .and_then_likely(|b| b.as_integer())
            .unwrap();
        usize::try_from(slide_boost).unwrap()
    }

    pub fn slide_timer() -> Duration {
        let slide_timer = CONFIG
            .get_conf("slide_timer")
            .and_then_likely(|t| t.as_integer())
            .unwrap();
        Duration::from_millis(slide_timer.try_into().unwrap())
    }
}
