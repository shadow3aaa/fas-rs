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
    map: HashMap<String, String>,
    timer: Instant,
}

impl Node {
    pub fn init() -> Result<Self> {
        let _ = fs::create_dir(NODE_PATH);

        let mut result = Self {
            map: HashMap::new(),
            timer: Instant::now(),
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
        self.refresh()
    }

    pub fn remove_node<S: AsRef<str>>(&mut self, i: S) -> Result<()> {
        let id = i.as_ref();

        let path = Path::new(NODE_PATH).join(id);
        fs::remove_file(path)?;

        self.refresh()
    }

    pub fn get_node<S: AsRef<str>>(&mut self, id: S) -> Result<String> {
        let id = id.as_ref();

        if self.timer.elapsed() > REFRESH_TIME {
            self.refresh()?;
        }

        self.map
            .get_mut(id)
            .map_or_else(|| Err(Error::NodeNotFound), |value| Ok(value.clone()))
    }

    fn refresh(&mut self) -> Result<()> {
        for entry in fs::read_dir(NODE_PATH)? {
            let Ok(entry) = entry else {
                continue;
            };

            if entry.file_type()?.is_file() {
                let id = entry.file_name().into_string().unwrap();
                let value = fs::read_to_string(entry.path())?;
                self.map.insert(id, value);
            }
        }

        Ok(())
    }
}
