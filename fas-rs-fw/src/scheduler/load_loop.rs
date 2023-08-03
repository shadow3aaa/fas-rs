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
use std::{thread, time::Duration};

use super::Scheduler;
use crate::config::CONFIG;

use likely_stable::if_unlikely;
use log::debug;

impl Scheduler {
    pub fn load_loop(&self) -> ! {
        let mut temp = None;
        loop {
            let current = CONFIG.cur_game_fps();

            #[allow(unused_variables)]
            if temp != current {
                temp = current;
                if_unlikely! {
                    let Some((ref game, fps, frame_windows)) = &temp => {
                        self.load(*fps, *frame_windows);
                        debug!("Loaded {} {}", game, fps);
                    } else {
                        self.unload();
                        debug!("Unloaded");
                    }
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    }
}
