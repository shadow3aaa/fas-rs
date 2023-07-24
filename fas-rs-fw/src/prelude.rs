/* Copyright 2023 shadow3aaa@gitbub.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License. */
//! 实现[`crate::VirtualFrameSensor`]或者[`crate::VirtualPerformanceController`]必须使用Api的重导出

// std
pub use std::error::Error;
pub use std::time::Duration;
// sensor
pub use crate::{Fps, FrameTime, TargetFps, VirtualFrameSensor};
// controller
pub use crate::VirtualPerformanceController;
