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
mod power_mode;

use std::{
    collections::HashMap,
    fs,
    path::Path,
    time::{Duration, Instant},
};

use crate::framework::error::{Error, Result};
pub use power_mode::Mode;

const NODE_PATH: &str = "/dev/fas_rs";
const REFRESH_TIME: Duration = Duration::from_secs(1);

pub struct Node {
    map: HashMap<String, (String, Instant)>,
    mode: Mode,
    mode_timer: Instant,
}

impl Node {
    pub fn init() -> Result<Self> {
        let _ = fs::create_dir(NODE_PATH);

        let mut result = Self {
            map: HashMap::new(),
            mode: Mode::Balance,
            mode_timer: Instant::now(),
        };

        let _ = result.remove_node("mode");
        result.create_node("mode", "balance")?;

        Ok(result)
    }

    pub fn create_node<S: AsRef<str>>(&mut self, i: S, d: S) -> Result<()> {
        let id = i.as_ref();
        let default = d.as_ref();

        let path = Path::new(NODE_PATH).join(id);
        fs::write(path, default)?;

        self.map
            .entry(id.to_string())
            .or_insert((default.to_string(), Instant::now()));

        Ok(())
    }

    pub fn remove_node<S: AsRef<str>>(&mut self, i: S) -> Result<()> {
        let id = i.as_ref();

        let path = Path::new(NODE_PATH).join(id);
        fs::remove_file(path)?;

        self.map.remove(id);

        Ok(())
    }

    pub fn get_node<S: AsRef<str>>(&mut self, i: S) -> Result<String> {
        let id = i.as_ref();

        if let Some((value, stamp)) = self.map.get_mut(id) {
            if stamp.elapsed() > REFRESH_TIME {
                let path = Path::new(NODE_PATH).join(id);
                *value = fs::read_to_string(path)?;
                *stamp = Instant::now();
            }

            Ok(value.clone())
        } else {
            Err(Error::NodeNotFound)
        }
    }
}
