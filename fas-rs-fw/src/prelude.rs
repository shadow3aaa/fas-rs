//! 实现[`crate::VirtualFrameSensor`]或者[`crate::VirtualPerformanceController`]必须使用Api的重导出

// std
pub use std::error::Error;
pub use std::time::Duration;
// sensor
pub use crate::{Fps, FrameTime, TargetFps, VirtualFrameSensor};
// controller
pub use crate::VirtualPerformanceController;
