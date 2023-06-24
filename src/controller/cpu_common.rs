use super::write_pool::WritePool;
use bimap::BiHashMap;
use fas_rs_fw::prelude::*;

use std::{
    cell::{Cell, RefCell},
    fs,
    path::{Path, PathBuf},
};

const POLICY_PATH: &str = "/sys/devices/system/cpu/cpufreq";
// 升频: 向频率上限写入`当前频率pos + BALANCE + 升频次数`的频率
// 降频: 向频率上限写入`当前频率pos + BALANCE - 降频次数`的频率
const JUMP_BALANCE: usize = 3;

type Frequency = usize;
type FrequencyTable = BiHashMap<usize, Frequency>;
type Policy = (PathBuf, FrequencyTable);
pub struct CpuCommon {
    write_pool: RefCell<WritePool>,
    policies: Vec<Policy>,
    jump: Cell<isize>,
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
        Path::new(POLICY_PATH).exists()
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut policies = Vec::with_capacity(5);

        let cpufreq = fs::read_dir(POLICY_PATH)?;
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

            policies.push((path, freq_table));
        }

        // 按policy降序排列
        policies.sort_by(|a, b| {
            let name_a = a.0.file_name().unwrap().to_str().unwrap();
            let name_b = b.0.file_name().unwrap().to_str().unwrap();

            let num_a: u32 = name_a.split("policy").nth(1).unwrap().parse().unwrap();
            let num_b: u32 = name_b.split("policy").nth(1).unwrap().parse().unwrap();
            num_b.cmp(&num_a)
        });
        policies.truncate(2); // 保留后两个集群即可

        Ok(Self {
            write_pool: RefCell::new(WritePool::new(4)),
            policies,
            jump: Cell::new(0),
        })
    }

    fn limit(&self) {
        if self.jump.get() > 0 {
            self.jump.set(0);
        }

        self.policies.iter().for_each(|policy| {
            let (pos, _) = Self::read_cur_freq(policy);
            let (path, table) = policy;

            let pos = (pos + JUMP_BALANCE).saturating_sub(self.jump.get().wrapping_abs() as usize);
            let write_freq = table
                .get_by_left(&pos)
                .unwrap_or_else(|| table.right_values().max().unwrap())
                .to_string();

            self.write_pool
                .borrow_mut()
                .write(&path.join("scaling_max_freq"), &write_freq)
                .unwrap();
        });

        self.jump.set(self.jump.get() - 1);
    }

    fn release(&self) {
        if self.jump.get() < 0 {
            self.jump.set(0);
        }

        self.policies.iter().for_each(|policy| {
            let (pos, _) = Self::read_cur_freq(policy);
            let (path, table) = policy;

            let pos = pos + JUMP_BALANCE + self.jump.get() as usize;
            let write_freq = table
                .get_by_left(&pos)
                .unwrap_or_else(|| table.right_values().max().unwrap())
                .to_string();

            self.write_pool
                .borrow_mut()
                .write(&path.join("scaling_max_freq"), &write_freq)
                .unwrap();
        });

        self.jump.set(self.jump.get() + 1);
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        self.policies.iter().for_each(|policy| {
            let max_freq = Self::max_freq(policy).unwrap().to_string();
            self.write_pool
                .borrow_mut()
                .write(&policy.0.join("scaling_max_freq"), &max_freq)
                .unwrap();
        });
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        self.policies.iter().for_each(|policy| {
            let max_freq = Self::max_freq(policy).unwrap();
            self.write_pool
                .borrow_mut()
                .write(&policy.0.join("scaling_max_freq"), &max_freq.to_string())
                .unwrap();
        });
        Ok(())
    }
}
