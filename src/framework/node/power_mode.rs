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
use std::{str::FromStr, fmt::{Display, Formatter, self}};

use super::Node;
use crate::framework::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mode = match self {
            Self::Powersave => "powersave",
            Self::Balance => "balance",
            Self::Performance => "performance",
            Self::Fast => "fast",
        };
        
        write!(f, "{mode}")
    }
}

impl Node {
    pub fn get_mode(&mut self) -> Result<Mode> {
        let mode = self.get_node("mode").or(Err(Error::NodeNotFound))?;

        Mode::from_str(mode.trim())
    }
}
