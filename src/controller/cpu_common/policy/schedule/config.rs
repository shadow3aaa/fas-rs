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

use anyhow::Result;
use fas_rs_fw::prelude::*;

use crate::error::Error;

use super::Schedule;

impl Schedule {
    pub fn touch_boost_conf(config: &Config) -> Result<(usize, usize, Duration)> {
        let touch_boost = config
            .get_conf("touch_boost")?
            .as_integer()
            .ok_or(Error::ParseConfig)?;

        let slide_boost = config
            .get_conf("slide_boost")?
            .as_integer()
            .ok_or(Error::ParseConfig)?;

        let slide_timer = config
            .get_conf("slide_timer")?
            .as_integer()
            .ok_or(Error::ParseConfig)?;
        let slide_timer = Duration::from_millis(slide_timer.try_into()?);

        Ok((
            touch_boost.try_into()?,
            slide_boost.try_into()?,
            slide_timer,
        ))
    }
}
