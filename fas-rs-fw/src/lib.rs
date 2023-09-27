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
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

#[cfg(not(target_os = "android"))]
#[cfg(not(target_arch = "aarch64"))]
compile_error!("Only for aarch64 android");

pub mod config;
mod error;
mod node;
pub mod prelude;
mod scheduler;

pub use config::Config;
pub use error::{Error, Result};
pub use node::Node;
pub use scheduler::Scheduler;

/// 性能控制器接口
pub trait PerformanceController: Send {
    fn limit(&self, c: &Config) -> Result<()>;
    fn release(&self, c: &Config) -> Result<()>;
    fn release_max(&self, c: &Config) -> Result<()>;
    /// 游戏状态初始化
    ///
    /// # Errors
    ///
    /// 初始化失败
    fn init_game(&self, c: &Config) -> Result<()>;
    /// 默认状态初始化
    ///
    /// # Errors
    ///
    /// 还原失败
    fn init_default(&self, c: &Config) -> Result<()>;
}
