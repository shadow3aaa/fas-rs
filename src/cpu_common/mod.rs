mod cpu_info;

use std::{fs, time::Duration};

use anyhow::Result;

use cpu_info::Info;
#[cfg(debug_assertions)]
use log::debug;
use log::error;

use crate::{api::ApiV0, Extension};

const BASE_FREQ: isize = 700_000;

#[derive(Debug)]
pub struct Controller {
    max_freq: isize,
    min_freq: isize,
    policy_freq: isize,
    cpu_infos: Vec<Info>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        let cpu_infos: Vec<_> = fs::read_dir("/sys/devices/system/cpu/cpufreq")?
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.is_dir()
                    && path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("policy")
            })
            .map(|path| Info::new(path).unwrap())
            .collect();

        #[cfg(debug_assertions)]
        debug!("cpu infos: {cpu_infos:?}");

        let max_freq = cpu_infos
            .iter()
            .flat_map(|info| info.freqs.iter())
            .max()
            .copied()
            .unwrap();

        let min_freq = cpu_infos
            .iter()
            .flat_map(|info| info.freqs.iter())
            .min()
            .copied()
            .unwrap();

        Ok(Self {
            max_freq,
            min_freq,
            policy_freq: max_freq,
            cpu_infos,
        })
    }

    pub fn init_game(&mut self, extension: &Extension) {
        self.policy_freq = self.max_freq;
        extension.tigger_extentions(ApiV0::InitCpuFreq);

        for cpu in &self.cpu_infos {
            cpu.write_freq(self.max_freq)
                .unwrap_or_else(|e| error!("{e:?}"));
        }
    }

    pub fn init_default(&mut self, extension: &Extension) {
        self.policy_freq = self.max_freq;
        extension.tigger_extentions(ApiV0::ResetCpuFreq);

        for cpu in &self.cpu_infos {
            cpu.reset_freq().unwrap_or_else(|e| error!("{e:?}"));
        }
    }

    pub fn fas_update_freq(&mut self, factor: f64) {
        self.policy_freq = self
            .policy_freq
            .saturating_add((BASE_FREQ as f64 * factor) as isize)
            .clamp(self.min_freq, self.max_freq);
        println!("{} {factor:.4}", self.policy_freq);
        for cpu in &self.cpu_infos {
            cpu.write_freq(self.policy_freq)
                .unwrap_or_else(|e| error!("{e:?}"));
        }
    }

    pub fn scale_factor(target_fps: u32, frame: Duration, target: Duration) -> f64 {
        if frame > target {
            let factor_a = (frame - target).as_nanos() as f64 / target.as_nanos() as f64;
            let factor_b = 120.0 / target_fps as f64;
            factor_a * f64::from(factor_b)
        } else {
            let factor_a = (target - frame).as_nanos() as f64 / target.as_nanos() as f64;
            let factor_b = 120.0 / target_fps as f64;
            factor_a * f64::from(factor_b) * -1.0
        }
    }
}
