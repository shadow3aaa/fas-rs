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
use std::{fs, path::Path};

use log::error;

use crate::error::{Error, Result};

const NODE_PATH: &str = "/dev/fas_rs";

#[derive(Debug)]
pub enum Mode {
    Powersave,
    Balance,
    Performance,
    Fast,
}

pub struct Node;

impl Node {
    /// 初始化节点
    pub fn init() -> Result<()> {
        let _ = fs::remove_dir_all(NODE_PATH);
        fs::create_dir(NODE_PATH)?;

        Self::create_node("mode", "balance")?;

        Ok(())
    }

    /// 创建一个新节点
    pub fn create_node<S: AsRef<str>>(i: S, d: S) -> Result<()> {
        let id = i.as_ref();
        let default = d.as_ref();

        let path = Path::new(NODE_PATH).join(id);
        fs::write(path, default)?;

        Ok(())
    }

    /// 读取当前模式
    pub fn read_mode() -> Result<Mode> {
        let path = Path::new(NODE_PATH).join("mode");
        Ok(
            match fs::read_to_string(path)
                .map_err(|_| Error::NodeNotFound)?
                .trim()
            {
                "powersave" => Mode::Powersave,
                "balance" => Mode::Balance,
                "performance" => Mode::Performance,
                "fast" => Mode::Fast,
                _ => {
                    error!("Failed to parse current mode");
                    Mode::Balance
                }
            },
        )
    }

    /// 读取指定的节点
    #[inline]
    pub fn read_node<S: AsRef<str>>(i: S) -> Result<String> {
        let id = i.as_ref();

        let path = Path::new(NODE_PATH).join(id);
        fs::read_to_string(path).map_err(|_| Error::NodeNotFound)
    }
}
