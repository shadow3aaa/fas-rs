mod policy;

use std::{
    cell::Cell,
    cmp, fs,
    path::{Path, PathBuf},
};

use cpu_cycles_reader::Cycles;
use fas_rs_fw::prelude::*;

use crate::config::CONFIG;
use crate::debug;
use policy::Policy;

pub struct CpuCommon {
    target_diff: Cell<Cycles>,
    policies: Vec<Policy>,
}

impl CpuCommon {
    fn set_target_diff(&self, c: Cycles) {
        self.target_diff.set(c);
        self.policies
            .iter()
            .for_each(|p| p.set_target_diff(self.target_diff.get()));
        debug! {
            println!("taregt diff: {}", self.target_diff.get());
        }
    }
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
        let target_diff = CONFIG
            .get_conf("default_target_diff")
            .and_then(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();
        let target_diff = Cell::new(target_diff);

        let cpufreq = fs::read_dir("/sys/devices/system/cpu/cpufreq")?;
        let mut policies: Vec<PathBuf> = cpufreq.into_iter().map(|e| e.unwrap().path()).collect();

        policies.sort_by(|a, b| {
            let num_a: u8 = a
                .file_name()
                .and_then(|f| f.to_str()?.split("policy").nth(1)?.parse().ok())
                .unwrap();
            let num_b: u8 = b
                .file_name()
                .and_then(|f| f.to_str()?.split("policy").nth(1)?.parse().ok())
                .unwrap();
            num_b.cmp(&num_a)
        });
        policies.truncate(2); // 保留后两个集群
        let policies = policies
            .into_iter()
            .map(|path| Policy::new(&path, 1))
            .collect();
        Ok(Self {
            policies,
            target_diff,
        })
    }

    fn limit(&self) {
        debug! {
            println!("limit");
        }
        let target_diff = self.target_diff.get() - Cycles::from_mhz(100);
        let target_diff = cmp::max(target_diff, Cycles::new(0));

        self.set_target_diff(target_diff);
    }

    fn release(&self) {
        debug! {
            println!("release");
        }
        let target_diff = self.target_diff.get() - Cycles::from_mhz(100);
        let target_diff = cmp::max(target_diff, Cycles::from_ghz(1));

        self.set_target_diff(target_diff);
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        let target_diff = CONFIG
            .get_conf("default_target_diff_fas")
            .and_then(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();
        self.set_target_diff(target_diff);
        self.policies.iter().for_each(|p| p.resume());
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        let always_on = CONFIG
            .get_conf("always_on_gov")
            .and_then(|b| b.as_bool())
            .unwrap();

        if !always_on {
            self.policies.iter().for_each(|p| p.pause());
            return Ok(());
        }

        let target_diff = CONFIG
            .get_conf("default_target_diff")
            .and_then(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();
        self.set_target_diff(target_diff);
        Ok(())
    }
}
