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
use std::{fs, path::Path, str::FromStr};

use crate::error::{Error, Result};

const NODE_PATH: &str = "/dev/fas_rs";

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Powersave,
    Balance,
    Performance,
    Fast,
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "powersave" => Self::Powersave,
            "balance" => Self::Balance,
            "performance" => Self::Performance,
            "fast" => Self::Fast,
            _ => return Err(Error::ParseNode),
        })
    }
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Self::Powersave => "powersave",
            Self::Balance => "balance",
            Self::Performance => "performance",
            Self::Fast => "fast",
        }
        .into()
    }
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

        Mode::from_str(
            fs::read_to_string(path)
                .map_err(|_| Error::NodeNotFound)?
                .trim(),
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
