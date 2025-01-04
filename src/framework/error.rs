// Copyright 2023-2025, shadow3 (@shadow3aaa)
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

use std::{ffi::NulError, io};

use frame_analyzer::AnalyzerError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    FrameAnalyzer(#[from] AnalyzerError),
    #[error("Got an error when parsing config")]
    ParseConfig,
    #[error("Got an error when parsing node")]
    ParseNode,
    #[error("No such a node")]
    NodeNotFound,
    #[error(transparent)]
    SerToml(#[from] toml::ser::Error),
    #[error(transparent)]
    DeToml(#[from] toml::de::Error),
    #[error(transparent)]
    SerXml(#[from] quick_xml::DeError),
    #[error("Missing {0} when building Scheduler")]
    SchedulerMissing(&'static str),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Lua {
        #[from]
        source: mlua::Error,
    },
    #[error(transparent)]
    Null {
        #[from]
        source: NulError,
    },
    #[error("Got an error: {0}")]
    #[allow(dead_code)]
    Other(&'static str),
}
