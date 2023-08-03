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
pub mod prelude {
    pub use super::Module;
    pub use crate::{config::Config, node::Node, Scheduler};
}

use prelude::*;

/// 模块
/// 把其它功能解耦为模块，通过[`macros::run_modules`]宏统一调用
pub trait Module {
    const NAME: &'static str;

    /// 构造
    fn new() -> Self;

    /// 设备是否支持此模块
    fn support() -> bool;

    /// 守护进程执行内容
    fn run(&mut self, config: &Config, node: &Node);
}
