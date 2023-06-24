use super::write_pool::WritePool;
use bimap::BiHashMap;
use fas_rs_fw::prelude::*;

use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};

const CPUFREQ: &str = "/sys/devices/system/cpu/cpufreq";

type Frequency = usize;
type FrequencyTable = BiHashMap<usize, Frequency>;
type Policy = (PathBuf, FrequencyTable);
pub struct CpuCommon {
    write_pool: RefCell<WritePool>,
    policys: Vec<Policy>,
}

impl CpuCommon {
    fn read_cur_freq(policy: &Policy) -> (usize, Frequency) {
        let freq = fs::read_to_string(policy.0.join("scaling_cur_freq"))
            .unwrap()
            .trim()
            .parse::<usize>()
            .unwrap();
        let pos = policy.1.get_by_right(&freq).unwrap();
        (*pos, freq)
    }

    fn max_freq(policy: &Policy) -> Result<Frequency, Box<dyn Error>> {
        let table = &policy.1;
        Ok(*table.right_values().max().ok_or("None")?)
    }
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
        let mut policys = Vec::with_capacity(5);

        let cpufreq = fs::read_dir(CPUFREQ)?;
        for policy in cpufreq {
            let path = policy?.path();
            let mut freq_table: Vec<Frequency> =
                fs::read_to_string(path.join("scaling_available_frequencies"))?
                    .split_whitespace()
                    .filter_map(|freq| freq.parse().ok())
                    .collect();
            freq_table.sort_unstable();
            let freq_table: BiHashMap<usize, Frequency> =
                freq_table.into_iter().enumerate().collect();

            policys.push((path, freq_table));
        }

        // 按policy降序排列
        policys.sort_by(|a, b| {
            let name_a = a.0.file_name().unwrap().to_str().unwrap();
            let name_b = b.0.file_name().unwrap().to_str().unwrap();

            let num_a: usize = name_a.split("policy").nth(1).unwrap().parse().unwrap();
            let num_b: usize = name_b.split("policy").nth(1).unwrap().parse().unwrap();
            num_b.cmp(&num_a)
        });
        policys.truncate(2); // 保留后两个集群即可

        Ok(Self {
            write_pool: RefCell::new(WritePool::new(4)),
            policys,
        })
    }

    fn limit(&self) {
        self.policys.iter().for_each(|policy| {
            let (pos, _) = Self::read_cur_freq(policy);
            let (path, table) = policy;

            let pos = pos.saturating_sub(1);
            if let Some(freq) = table.get_by_left(&pos) {
                self.write_pool
                    .borrow_mut()
                    .write(&path.join("scaling_max_freq"), &freq.to_string())
                    .unwrap();
            }
        })
    }

    fn release(&self) {
        self.policys.iter().for_each(|policy| {
            let (pos, _) = Self::read_cur_freq(policy);
            let (path, table) = policy;

            let pos = pos + 1;
            if let Some(freq) = table.get_by_left(&pos) {
                self.write_pool
                    .borrow_mut()
                    .write(&path.join("scaling_max_freq"), &freq.to_string())
                    .unwrap();
            }
        })
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        self.policys.iter().for_each(|policy| {
            let max_freq = Self::max_freq(policy).unwrap();
            self.write_pool
                .borrow_mut()
                .write(&policy.0.join("scaling_max_freq"), &max_freq.to_string())
                .unwrap();
        });
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        self.policys.iter().for_each(|policy| {
            let max_freq = Self::max_freq(policy).unwrap();
            self.write_pool
                .borrow_mut()
                .write(&policy.0.join("scaling_max_freq"), &max_freq.to_string())
                .unwrap();
        });
        Ok(())
    }
}
