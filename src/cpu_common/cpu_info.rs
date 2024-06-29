use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::Result;

#[derive(Debug)]
pub struct Info {
    pub policy: i32,
    path: PathBuf,
    pub freqs: Vec<isize>,
}

impl Info {
    pub fn new(path: PathBuf) -> Result<Self> {
        let policy = path.file_name().unwrap().to_str().unwrap()[6..].parse()?;

        let mut freqs: Vec<_> = fs::read_to_string(path.join("scaling_available_frequencies"))?
            .split_whitespace()
            .map(|f| f.parse().unwrap())
            .collect();

        if let Ok(boost_freqs) = fs::read_to_string(path.join("scaling_boost_frequencies")) {
            let boost_freqs = boost_freqs
                .split_whitespace()
                .map(|f| f.parse::<isize>().unwrap());
            freqs.extend(boost_freqs);
        }

        freqs.sort_unstable();

        Ok(Self {
            policy,
            path,
            freqs,
        })
    }

    pub fn write_freq(&self, freq: isize) -> Result<()> {
        let freq = freq.to_string();
        let max_freq_path = self.max_freq_path();
        unlock_write(max_freq_path, &freq)?;

        if self.policy != 0 {
            let min_freq_path = self.min_freq_path();
            unlock_write(min_freq_path, &freq)?;
        }

        Ok(())
    }

    pub fn reset_freq(&self) -> Result<()> {
        let max_freq_path = self.max_freq_path();
        let min_freq_path = self.min_freq_path();

        unlock_write(max_freq_path, self.freqs.last().unwrap().to_string())?;
        unlock_write(min_freq_path, self.freqs.first().unwrap().to_string())?;

        Ok(())
    }

    fn max_freq_path(&self) -> PathBuf {
        self.path.join("scaling_max_freq")
    }

    fn min_freq_path(&self) -> PathBuf {
        self.path.join("scaling_min_freq")
    }
}

fn unlock_write<P, C>(path: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let _ = fs::set_permissions(path.as_ref(), PermissionsExt::from_mode(0o644));
    fs::write(path, contents)?;
    Ok(())
}
