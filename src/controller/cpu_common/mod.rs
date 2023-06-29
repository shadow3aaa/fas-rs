mod policy;

use std::{
    cell::Cell,
    cmp, fs,
    path::{Path, PathBuf},
};

use crate::config::CONFIG;
use crate::debug;
use fas_rs_fw::prelude::*;
use policy::Policy;

pub struct CpuCommon {
    target_usage: [Cell<u8>; 2],
    policies: Vec<Policy>,
}

impl CpuCommon {
    fn set_target_usage(&self, l: u8, r: u8) {
        self.target_usage[0].set(l);
        self.target_usage[1].set(r);
        self.policies.iter().for_each(|p| {
            p.set_target_usage(self.target_usage[0].get(), self.target_usage[1].get())
        });
        debug! { println!("taregt usage: {:#?}", self.target_usage) }
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
        let (l, r) = CONFIG
            .get_conf("default_target_usage")
            .and_then(|u| {
                let arr = u.as_array()?;
                assert_eq!(arr.len(), 2);
                Some((
                    arr[0].as_integer().unwrap() as u8,
                    arr[1].as_integer().unwrap() as u8,
                ))
            })
            .unwrap_or((74, 77));
        let target_usage = [Cell::new(l), Cell::new(r)];

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
            .map(|path| Policy::new(&path, 2))
            .collect();
        Ok(Self {
            policies,
            target_usage,
        })
    }

    fn limit(&self) {
        debug! { println!("limit") }
        let min = cmp::min(self.target_usage[0].get() + 2, 100);
        let max = cmp::min(min + 2, 100);

        self.set_target_usage(min, max);
    }

    fn release(&self) {
        debug! { println!("release") }
        let min = self.target_usage[0].get().saturating_sub(2);
        let max = cmp::min(min + 2, 100);

        self.set_target_usage(min, max);
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        let (l, r) = CONFIG
            .get_conf("default_fas_target_usage")
            .and_then(|u| {
                let arr = u.as_array()?;
                assert_eq!(arr.len(), 2);
                Some((
                    arr[0].as_integer().unwrap() as u8,
                    arr[1].as_integer().unwrap() as u8,
                ))
            })
            .unwrap_or((74, 77));
        self.set_target_usage(l, r);
        self.policies.iter().for_each(|p| p.resume());
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        let (l, r) = CONFIG
            .get_conf("default_target_usage")
            .and_then(|u| {
                let arr = u.as_array()?;
                assert_eq!(arr.len(), 2);
                Some((
                    arr[0].as_integer().unwrap() as u8,
                    arr[1].as_integer().unwrap() as u8,
                ))
            })
            .unwrap_or((50, 52));
        self.set_target_usage(l, r);
        // self.policies.iter().for_each(|p| p.pause());
        Ok(())
    }
}
