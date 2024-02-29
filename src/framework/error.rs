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
use std::io;

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Got an error when parsing config")]
    ParseConfig,
    #[error("Got an error when parsing node")]
    ParseNode,
    #[error("No such a node")]
    NodeNotFound,
    #[error("Got an error when serialize toml: {source:?}")]
    SerToml {
        #[from]
        source: toml::ser::Error,
    },
    #[error("Got an error when deserialize toml: {source:?}")]
    DeToml {
        #[from]
        source: toml::de::Error,
    },
    #[error("Got an error when (de)serialize xml: {source:?}")]
    SerXml {
        #[from]
        source: quick_xml::DeError,
    },
    #[error("Missing {0} when building Scheduler")]
    SchedulerMissing(&'static str),
    #[error("Got an io error: {source:?}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("Got an error: {0}")]
    Other(&'static str),
}
