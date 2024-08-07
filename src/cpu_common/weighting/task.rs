use anyhow::Result;
use cpu_cycles_reader::{CyclesInstant, CyclesReader};
use libc::pid_t;

#[derive(Debug)]
pub struct TaskMeta {
    pub weight: f64,
    pub cycles_trace: Vec<CyclesInstant>,
    pub cycles_reader: CyclesReader,
}

impl TaskMeta {
    pub fn new(tid: pid_t, num_cpus: usize) -> Result<Self> {
        let cycles_reader = CyclesReader::new(Some(tid))?;
        let mut cycles_trace = Vec::new();

        for cpu in 0..num_cpus {
            cycles_trace.push(cycles_reader.instant(cpu as i32)?);
        }

        Ok(Self {
            weight: 0.0,
            cycles_reader,
            cycles_trace,
        })
    }
}
