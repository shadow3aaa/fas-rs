//! 实现[`crate::VirtualFrameSensor`]或者[`crate::VirtualPerformanceController`]必须使用Api的重导出
//! ```
//! // sensor
//! use fas_rs_fw::prelude::sensor::*;
//! // controller
//! use fas_rs_fw::prelude::contoller::*;
//! ```


pub mod sensor {
    pub use crate::{
        VirtualFrameSensor,
        Fps,
        FrameTime,
    };
    pub use std::time::Duration;
}

pub mod controller {
    pub use crate::{
        VirtualPerformanceController,
    };
}