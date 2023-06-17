use std::fs;
use std::path::Path;

use fas_rs_fw::prelude::*;

const CPUFREQ: &str = "/sys/devices/system/cpu/cpufreq";

type Frequency = usize;
type FrequencyTable = Vec<Frequency>;
pub struct CpuCommon {
    freq_table: Vec<FrequencyTable>,
}

impl VirtualPerformanceController for CpuCommon {
    fn support() -> bool
    where
        Self: Sized,
    {
        Path::new(CPUFREQ).exists()
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut freq_table = Vec::with_capacity(2);

        let cpufreq = fs::read_dir(CPUFREQ)?;
        for policy in cpufreq {
            let table_path = policy?.path().join("scaling_available_frequencies");
            let table: FrequencyTable = fs::read_to_string(table_path)?
                .trim()
                .split_whitespace()
                .filter_map(|freq| freq.parse().ok())
                .collect();

            freq_table.push(table);
        }

        freq_table.reverse();
        freq_table.truncate(2); // 保留后两个集群即可

        Ok(Self { freq_table })
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
