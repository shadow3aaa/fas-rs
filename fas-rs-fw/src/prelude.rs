//! 实现[`crate::VirtualFrameSensor`]或者[`crate::VirtualPerformanceController`]必须使用Api的重导出
//! ```
//! // sensor
//! use fas_rs_fw::prelude::*;
//! ```

// std
pub use std::error::Error;
pub use std::time::Duration;
// sensor
pub use crate::{Fps, TargetFps, FrameTime, VirtualFrameSensor};
// controller
pub use crate::VirtualPerformanceController;
