use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

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
