use fas_rs_fw::prelude::*;

pub struct CpuCommon {}

impl VirtualPerformanceController for CpuCommon {
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

    fn limit(&self) {
        todo!()
    }

    fn release(&self) {
        todo!()
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
