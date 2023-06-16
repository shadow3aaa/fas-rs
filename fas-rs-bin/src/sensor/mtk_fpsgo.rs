use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use fas_rs_fw::prelude::*;

const FPSGO: &str = "/sys/kernel/fpsgo";
const BUFFER_CAP: usize = 512;

pub struct MtkFpsGo {
    // 接收FrameTime
    frametime_receiver: Receiver<Vec<FrameTime>>,
    // 指定接收个数
    frametime_count_sender: SyncSender<u32>,
    // 接收Fps
    fps_receiver: Receiver<Vec<Fps>>,
    // 指定接收时间段
    fps_time_sender: SyncSender<Duration>,
    // 控制启停
    thread_handle: (JoinHandle<()>, JoinHandle<()>),
    pause: Arc<AtomicBool>,
}

impl VirtualFrameSensor for MtkFpsGo {
    fn support() -> bool
    where
        Self: Sized,
    {
        Path::new(FPSGO).join("fbt/fbt_info").exists()
            && Path::new(FPSGO).join("common/fpsgo_enable").exists() // 检查路径
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        // 控制启停的原子bool
        let pause = Arc::new(AtomicBool::new(false));

        // 管道
        let (frametime_sender, frametime_receiver) = mpsc::sync_channel(1);
        let (frametime_count_sender, frametime_count_receiver) = mpsc::sync_channel(1);

        let (fps_sender, fps_receiver) = mpsc::sync_channel(1);
        let (fps_time_sender, fps_time_receiver) = mpsc::sync_channel(1);

        let pause_frametime = pause.clone();
        let pause_fps = pause.clone();

        let thread_handle = (
            thread::spawn(move || {
                frametime_thread(frametime_sender, frametime_count_receiver, pause_frametime)
            }),
            thread::spawn(move || fps_thread(fps_sender, fps_time_receiver, pause_fps)),
        );

        Ok(Self {
            frametime_receiver,
            frametime_count_sender,
            fps_receiver,
            fps_time_sender,
            pause,
            thread_handle,
        })
    }

    fn frametimes(&self, count: u32) -> Vec<FrameTime> {
        self.frametime_count_sender.send(count).unwrap();
        self.frametime_receiver.recv().unwrap()
    }

    fn fps(&self, time: Duration) -> Vec<Fps> {
        self.fps_time_sender.send(time).unwrap();
        self.fps_receiver.recv().unwrap()
    }

    fn pause(&self) -> Result<(), Box<dyn Error>> {
        self.pause.store(true, Ordering::Release);
        Ok(())
    }

    fn resume(&self) -> Result<(), Box<dyn Error>> {
        fs::write(Path::new(FPSGO).join("common/fpsgo_enable"), "1")?;

        self.thread_handle.0.thread().unpark();
        self.thread_handle.1.thread().unpark();

        Ok(())
    }
}

fn frametime_thread(
    frametime_sender: SyncSender<Vec<FrameTime>>,
    frametime_count_receiver: Receiver<u32>,
    pause: Arc<AtomicBool>,
) {
    let mut buffer = VecDeque::with_capacity(BUFFER_CAP);

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }

        if buffer.len() > BUFFER_CAP {
            buffer.pop_front();
        }

        let stamps = [0, 0];

        if let Some(stamp) = parse_fbt_info() {
            stamps[0] = stamp
        } else {
            continue;
        }

        loop {
            if let Some(stamp) = parse_fbt_info() {
                if stamps[0] < stamp {
                    stamps[1] = stamp;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(6));
        }

        let frametime = FrameTime::from_nanos(stamps[1] - stamps[0]);

        buffer.push_back(frametime);
    }
}

fn parse_fbt_info() -> Option<u64> {
    /* 解析第8(从0开始)行
    1(状态)	0		37	19533	0x4c2e00000021	60(屏幕刷新率)	24029340996131(最新帧的vsync时间戳) */
    let fbt_info = fs::read_to_string(Path::new(FPSGO).join("/fbt/fbt_info")).unwrap();
    let parse_line = fbt_info.lines().nth(8)?.split_whitespace();

    let enabled = parse_line.nth(0)?.trim().parse::<u64>().ok()? == 1;

    if !enabled {
        fs::write(Path::new(FPSGO).join("common/fpsgo_enable"), "1").unwrap();
        return None; // 需要重新读取
    }

    return Some(parse_line.nth(6)?.trim().parse::<u64>().ok()?);
}

fn fps_thread(
    fps_sender: SyncSender<Vec<Fps>>,
    fps_time_receiver: Receiver<Duration>,
    pause: Arc<AtomicBool>,
) {
    let buffer = VecDeque::with_capacity(BUFFER_CAP);

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }
    }
}
