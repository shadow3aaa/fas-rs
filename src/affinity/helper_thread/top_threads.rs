use std::{fs, path::Path};

pub struct TopThreads {
    sched_stats: Vec<(u32, usize)>,
}

impl TopThreads {
    pub fn new(threads: &[u32]) -> Self {
        let sched_stats = threads
            .iter()
            .filter_map(|tid| {
                let sched_stat = Path::new("/proc").join(tid.to_string()).join("schedstat");
                let sched_stat = fs::read_to_string(sched_stat).ok()?;
                let sched_stat: usize = sched_stat.split_whitespace().next()?.parse().ok()?;
                Some((*tid, sched_stat))
            })
            .collect();
        Self { sched_stats }
    }

    pub fn result(&self) -> Vec<u32> {
        let mut threads: Vec<_> = self
            .sched_stats
            .iter()
            .filter_map(|(tid, last)| {
                let sched_stat = Path::new("/proc").join(tid.to_string()).join("schedstat");
                let sched_stat = fs::read_to_string(sched_stat).ok()?;
                let now: usize = sched_stat.split_whitespace().next()?.parse().ok()?;
                let time = now.saturating_sub(*last);
                Some((tid, time))
            })
            .collect();
        threads.sort_by_key(|(_, time)| *time);
        threads.reverse();
        threads.into_iter().map(|(tid, _)| *tid).collect()
    }
}
