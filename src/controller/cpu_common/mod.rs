mod policy;

use std::{cell::Cell, fs, path::Path};

use fas_rs_fw::prelude::*;

use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;
use log::debug;

use crate::config::CONFIG;
use policy::Policy;

pub struct CpuCommon {
    target_diff: Cell<Cycles>,
    policies: Vec<Policy>,
}

impl CpuCommon {
    fn set_target_diff(&self, c: Cycles) {
        let updated_target: Cycles = self
            .policies
            .iter()
            .map(|p| p.set_target_diff(c))
            .min()
            .unwrap();

        self.target_diff.set(updated_target);
        debug!("Got taregt diff: {}", c);
    }

    fn get_diff_move() -> Cycles {
        let mhz = CONFIG
            .get_conf("diff_move")
            .and_then_likely(|m| m.as_integer())
            .unwrap();

        Cycles::from_mhz(mhz)
    }

    pub(crate) fn always_on() -> bool {
        CONFIG
            .get_conf("always_on_gov")
            .and_then_likely(|b| b.as_bool())
            .unwrap()
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
            .and_then_likely(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();
        let target_diff = Cell::new(target_diff);

        let cpufreq = fs::read_dir("/sys/devices/system/cpu/cpufreq")?;
        let policies: Vec<_> = cpufreq
            .into_iter()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .map(|p| Policy::new(&p, 1))
            .collect();

        /* policies.sort_by(|a, b| {
            let num_a: u8 = a
                .file_name()
                .and_then_likely(|f| f.to_str()?.split("policy").nth(1)?.parse().ok())
                .unwrap();
            let num_b: u8 = b
                .file_name()
                .and_then_likely(|f| f.to_str()?.split("policy").nth(1)?.parse().ok())
                .unwrap();
            num_b.cmp(&num_a)
        });
        policies.truncate(2); // 保留后两个集群

        let policies = policies
            .into_iter()
            .map(|path| Policy::new(&path, 1))
            .collect(); */

        Ok(Self {
            target_diff,
            policies,
        })
    }

    fn limit(&self) {
        debug!("Cpu controller performance limit");
        let target_diff = self.target_diff.get() - Self::get_diff_move();
        let target_diff = target_diff.max(Cycles::new(0));

        self.set_target_diff(target_diff);
    }

    fn release(&self) {
        debug!("Cpu controller performance release");
        let target_diff = self.target_diff.get() + Self::get_diff_move();

        self.set_target_diff(target_diff);
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        let target_diff = CONFIG
            .get_conf("default_target_diff_fas")
            .and_then_likely(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();
        self.set_target_diff(target_diff);
        self.policies.iter().for_each(Policy::resume);
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        if !Self::always_on() {
            self.policies.iter().for_each(Policy::pause);
            return Ok(());
        }

        let target_diff = CONFIG
            .get_conf("default_target_diff")
            .and_then_likely(|d| Some(Cycles::from_mhz(d.as_integer()?)))
            .unwrap();
        self.set_target_diff(target_diff);
        Ok(())
    }
}
