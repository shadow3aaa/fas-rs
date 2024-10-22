// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io;

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
    #[error("Lua extension error: {source:?}")]
    Lua {
        #[from]
        source: mlua::Error,
    },
    #[error("Got an error: {0}")]
    #[allow(dead_code)]
    Other(&'static str),
}
