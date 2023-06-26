use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
    time::Duration,
};

pub(super) fn usage_thread(
    path: PathBuf,
    usage: Sender<u8>,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
) {
    let affect_cpus: HashSet<String> = fs::read_to_string(path.join("affected_cpus"))
        .unwrap()
        .split_whitespace()
        .map(|cpu| format!("cpu{}", cpu))
        .collect();

    thread::park(); // 等待唤醒

    loop {
        if exit.load(Ordering::Acquire) {
            return;
        } else if pause.load(Ordering::Acquire) {
            thread::park();
        }

        let stat_a = read_stat(&affect_cpus);
        thread::sleep(Duration::from_millis(100));
        let stat_b = read_stat(&affect_cpus);

        let new_usage: u8 = stat_a
            .iter()
            .zip(stat_b.iter())
            .map(|((total_a, idle_a), (total_b, idle_b))| {
                let total = total_b - total_a;
                let idle = idle_b - idle_a;

                100 - idle * 100 / total
            })
            .max()
            .unwrap() as u8;

        usage.send(new_usage).unwrap();
        // println!("{}％", usage.load(Ordering::Acquire));
    }
}

fn read_stat(affect_cpus: &HashSet<String>) -> Vec<(usize, usize)> {
    let stat: Vec<String> = fs::read_to_string("/proc/stat")
        .unwrap()
        .lines()
        .filter(|line| affect_cpus.iter().any(|cpu| line.starts_with(cpu)))
        .map(|s| s.to_owned())
        .collect();

    stat.iter()
        .map(|cpu| {
            (
                cpu.split_whitespace()
                    .skip(1)
                    .map(|time| time.parse::<usize>().unwrap())
                    .sum::<usize>(),
                cpu.split_whitespace()
                    .nth(4)
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
            )
        })
        .collect()
}
