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
use std::process::Command;

use log::{info, trace};
use surfaceflinger_hook_api::Connection;

use super::Scheduler;
use crate::{
    config::Config,
    error::{Error, Result},
    PerformanceController,
};

impl<P: PerformanceController> Scheduler<P> {
    pub(super) fn main_loop(
        config: &mut Config,
        controller: &P,
        connection: &mut Connection,
        jank_level_max: Option<u32>,
    ) -> Result<()> {
        let mut status = None;
        let mut buffer_size: usize = 15;
        let mut buffer = Vec::with_capacity(144);

        Self::init_load_default(connection, controller, config)?;

        loop {
            let update_config = config.cur_game_fps();

            if status != update_config {
                status = update_config;
                if let Some((game, target_fps)) = &status {
                    info!("Loaded on game: {game}");

                    buffer_size = *target_fps as usize / 4;
                    buffer_size = buffer_size.max(5);
                    buffer_size = (u32::try_from(buffer_size)
                        .map_err(|_| Error::Other("Failed to trans usize to u32"))?
                        * get_refresh_rate().unwrap_or_default()
                        / target_fps) as usize;

                    Self::init_load_game(*target_fps, connection, controller, config)?;
                } else {
                    Self::init_load_default(connection, controller, config)?;
                }

                continue;
            }

            let Ok(level) = connection.recv() else {
                continue;
            };

            trace!("Recv jank: {level:?}");

            controller.perf(*level, config);

            if buffer.len() < buffer_size {
                buffer.push(*level);
            } else {
                let max_level = buffer.iter().copied().max().unwrap_or_default();
                let jank = jank_level_max.map_or(max_level, |max| max_level.min(max));

                controller.perf(jank, config);

                buffer.clear();
            }
        }
    }

    fn init_load_game(
        target_fps: u32,
        connection: &Connection,
        controller: &P,
        config: &Config,
    ) -> Result<()> {
        connection.set_input(Some(target_fps))?;
        controller.init_game(config)
    }

    fn init_load_default(connection: &Connection, controller: &P, config: &Config) -> Result<()> {
        connection.set_input(None)?;
        controller.init_default(config)
    }
}

fn get_refresh_rate() -> Option<u32> {
    let dumpsys_data = Command::new("dumpsys")
        .arg("SurfaceFlinger")
        .output()
        .ok()?;
    let dumpsys_data = String::from_utf8_lossy(&dumpsys_data.stdout);

    let parse_line = dumpsys_data
        .lines()
        .find(|line| line.contains("refresh-rate"))?;
    Some(
        parse_line
            .split(':')
            .nth(1)?
            .split('.')
            .next()?
            .trim()
            .parse()
            .unwrap(),
    )
}
