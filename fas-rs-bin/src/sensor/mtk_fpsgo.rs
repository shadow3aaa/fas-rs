use fas_rs_fw::prelude::sensor::*;

pub struct SensorMtkFpsGo {}

impl VirtualFrameSensor for SensorMtkFpsGo {
    fn support() -> bool
    where
        Self: Sized,
    {
        todo!()
    }
    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        todo!()
    }
    fn frametimes(&self, count: u32) -> Vec<FrameTime> {
        todo!()
    }

    fn fps(&self, time: Duration) -> Fps {
        todo!()
    }

    fn pause(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn resume(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
