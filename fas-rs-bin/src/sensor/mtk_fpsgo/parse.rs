use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::Arc;
use std::thread;

use super::FPSGO;
use fas_rs_fw::prelude::*;

const BUFFER_CAP: usize = 512;

pub fn frametime_thread(
    frametime_sender: SyncSender<Vec<FrameTime>>,
    frametime_count_receiver: Receiver<u32>,
    pause: Arc<AtomicBool>,
) {
    let mut buffer = VecDeque::with_capacity(BUFFER_CAP);
    let mut stamps = [0, 0];

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }

        if buffer.len() > BUFFER_CAP {
            buffer.pop_front();
        }

        // 获取第一个时间戳
        if let Some(stamp) = parse_fbt_info() {
            stamps[0] = stamp
        } else {
            continue;
        }

        // 轮询(sysfs不可用inotify监听)
        // 值变化后保存为第二个时间戳
        loop {
            if let Some(stamp) = parse_fbt_info() {
                if stamps[0] < stamp {
                    stamps[1] = stamp;
                    break;
                }
            }
            // 轮询间隔
            thread::sleep(Duration::from_millis(6));
        }

        // todo: 消除屏幕刷新率和目标帧率不一样的时候产生的误差
        let frametime = FrameTime::from_nanos(stamps[1] - stamps[0]);

        buffer.push_back(frametime);
    }
}

pub fn fps_thread(
    fps_sender: SyncSender<Vec<Fps>>,
    fps_time_receiver: Receiver<Duration>,
    pause: Arc<AtomicBool>,
) {
    todo!()
    /*
    let buffer = VecDeque::with_capacity(BUFFER_CAP);

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }

        if buffer.len() > BUFFER_CAP {
            buffer.pop_front();
        }


    }
    */
}

fn parse_fbt_info() -> Option<u64> {
    /* 解析第8(从0开始)行
    1(状态)	0		37	19533	0x4c2e00000021	60(屏幕刷新率)	24029340996131(最新帧的vsync时间戳) */
    let fbt_info = fs::read_to_string(Path::new(FPSGO).join("/fbt/fbt_info")).unwrap();
    let mut parse_line = fbt_info.lines().nth(8)?.split_whitespace();

    let enabled = parse_line.next()?.trim().parse::<u64>().ok()? == 1;

    if !enabled {
        fs::write(Path::new(FPSGO).join("common/fpsgo_enable"), "1").unwrap();
        return None; // 需要重新读取
    }

    return parse_line.nth(6)?.trim().parse::<u64>().ok();
}
