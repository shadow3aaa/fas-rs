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
use log::info;
use surfaceflinger_hook_api::{Connection, JankLevel, JankType};

use super::Scheduler;
use crate::{config::Config, error::Result, node::Node, PerformanceController};

impl<P: PerformanceController> Scheduler<P> {
    pub(super) fn main_loop(
        config: &mut Config,
        node: &Node,
        controller: &P,
        connection: &Connection,
    ) -> Result<()> {
        let mut status = None;

        loop {
            let update_config = config.cur_game_fps();

            if status != update_config {
                status = update_config;
                if let Some((game, target_fps)) = &status {
                    info!("Loaded on game: {game}");
                    Self::init_load_game(*target_fps, connection, controller, config, node)?;
                } else {
                    Self::init_load_default(connection, controller, config, node)?;
                }

                continue;
            }

            let JankLevel(level) = connection.recv()?;
            controller.perf(level);
        }
    }

    fn init_load_game(
        target_fps: u32,
        connection: &Connection,
        controller: &P,
        config: &Config,
        node: &Node,
    ) -> Result<()> {
        connection.set_input(Some(target_fps), JankType::Vsync)?;
        controller.plug_out(config, node)?;
        controller.plug_in(config, node)
    }

    fn init_load_default(
        connection: &Connection,
        controller: &P,
        config: &Config,
        node: &Node,
    ) -> Result<()> {
        connection.set_input(None, JankType::Soft)?;
        controller.plug_out(config, node)?;
        controller.plug_in(config, node)
    }
}
