mod policy;

use fas_rs_fw::prelude::*;
use policy::Policy;

use std::{
    cell::Cell,
    cmp, fs,
    path::{Path, PathBuf},
};

pub struct CpuCommon {
    target_usage: Cell<u8>,
    policies: Vec<Policy>,
}

impl VirtualPerformanceController for CpuCommon {
    fn support() -> bool
    where
        Self: Sized,
    {
        Path::new("/sys/devices/system/cpu/cpufreq").exists()
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let target_usage = Cell::new(60);

        let cpufreq = fs::read_dir("/sys/devices/system/cpu/cpufreq")?;
        let mut policies: Vec<PathBuf> = cpufreq.into_iter().map(|e| e.unwrap().path()).collect();

        policies.sort_by(|a, b| {
            let num_a: u8 = a
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split("policy")
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();
            let num_b: u8 = b
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split("policy")
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();
            num_b.cmp(&num_a)
        });
        policies.truncate(2); // 保留后两个集群
        let policies = policies
            .into_iter()
            .map(|path| Policy::new(&path, target_usage.get()))
            .collect();
        Ok(Self {
            policies,
            target_usage,
        })
    }

    fn limit(&self) {
        let new_usage = cmp::min(self.target_usage.get() + 1, 100);
        self.target_usage.set(new_usage);
        self.policies
            .iter()
            .for_each(|p| p.set_target_usage(new_usage));
    }

    fn release(&self) {
        let new_usage = self.target_usage.get().saturating_sub(1);
        self.target_usage.set(new_usage);
        self.policies
            .iter()
            .for_each(|p| p.set_target_usage(new_usage));
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        self.policies.iter().for_each(|p| p.resume());
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        self.policies.iter().for_each(|p| p.pause());
        Ok(())
    }
}
