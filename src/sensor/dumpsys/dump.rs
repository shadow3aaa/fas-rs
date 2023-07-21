use std::process::Command;

use fas_rs_fw::prelude::*;

use super::DumpSys;

impl DumpSys {
    pub fn get_cur_view() -> Option<String> {
        let dump = Command::new("dumpsys")
            .args(["SurfaceFlinger", "--list"])
            .output()
            .ok()?;

        let views = String::from_utf8_lossy(&dump.stdout).into_owned();

        views
            .lines()
            .find(|s| s.contains("SurfaceView[") && s.contains("BLAST"))
            .map_or_else(
                || {
                    views
                        .lines()
                        .find(|s| s.contains("SurfaceView -"))
                        .map(|view| view.trim().to_owned())
                },
                |view| Some(view.trim().to_owned()),
            )
    }

    pub fn dump_frametimes(&self, target_fps: TargetFps) -> Vec<FrameTime> {
        let Some(view) = Self::get_cur_view() else {
            return Vec::new();
        };

        let take_count = self.count.get();

        let ori_data = Command::new("dumpsys")
            .args(["SurfaceFlinger", "--latency", &view])
            .output()
            .unwrap();
        let ori_data = String::from_utf8_lossy(&ori_data.stdout).into_owned();

        let frametimes: Vec<_> = ori_data
            .lines()
            .skip(2)
            .filter_map(|l| l.split_whitespace().nth(1))
            .map(|v| v.parse::<u64>().unwrap())
            .filter(|v| v != &0 && v <= &10_000_000_000_000_000)
            .collect();

        frametimes
            .windows(2)
            .map(|ft| Duration::from_nanos(ft[1] - ft[0]))
            .take(take_count as usize)
            .map(|f| self.ignore.ign(f, target_fps))
            .map(|f| f + Duration::from_micros(100))
            .collect()
    }
}
