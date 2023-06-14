mod sensor;
mod controller;

use std::error::Error;

use fas_rs_fw::Scheduler;

fn main() -> Result<(), Box<dyn Error>> {
    todo!()
    let scheduler = Scheduler::new()?;
    
    Ok(())
}
