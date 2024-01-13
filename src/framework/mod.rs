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
#![allow(dead_code)]

pub mod config;
mod error;
mod node;
pub mod prelude;
mod scheduler;

pub use config::Config;
pub use error::Result;
pub use node::Mode;
pub use scheduler::Scheduler;

pub trait PerformanceController: Send {
    fn limit(&self, m: Mode, c: &Config) -> Result<()>;
    fn release(&self, m: Mode, c: &Config) -> Result<()>;
    fn release_max(&self, m: Mode, c: &Config) -> Result<()>;
    fn init_game(&self, m: Mode, c: &Config) -> Result<()>;
    fn init_default(&self, m: Mode, c: &Config) -> Result<()>;
}
