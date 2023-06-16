use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;

use super::{BUFFER_CAP, FPSGO};
use fas_rs_fw::prelude::*;

pub fn frametime_thread(frametime: Arc<Mutex<Vec<FrameTime>>>, pause: Arc<AtomicBool>) {
    let mut buffer = VecDeque::with_capacity(BUFFER_CAP);
    let mut stamps = [0, 0];

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }

        if buffer.len() > BUFFER_CAP {
            buffer.pop_front();
        }

        // 尝试复制buffer到读取区
        *frametime.try_lock().unwrap() = buffer.clone().into();

        // 获取第一个时间戳
        if let Some(stamp) = parse_frametime() {
            stamps[0] = stamp
        } else {
            continue;
        }

        // 轮询(sysfs不可用inotify监听)
        // 值变化后保存为第二个时间戳
        loop {
            if let Some(stamp) = parse_frametime() {
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

pub fn fps_thread(fps: Arc<Mutex<Vec<Fps>>>, pause: Arc<AtomicBool>) {
    let buffer = VecDeque::with_capacity(BUFFER_CAP);

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }

        if buffer.len() > BUFFER_CAP {
            buffer.pop_front();
        }
    }
}

/* 解析第8(从0开始)行
1(状态)	0		37	19533	0x4c2e00000021	60(屏幕刷新率)	24029340996131(最新帧的vsync时间戳) */
fn parse_frametime() -> Option<u64> {
    let fbt_info = fs::read_to_string(Path::new(FPSGO).join("/fbt/fbt_info")).unwrap();
    let mut parse_line = fbt_info.lines().nth(8)?.split_whitespace();

    let enabled = parse_line.next()?.trim().parse::<u64>().ok()? == 1;

    // fpsgo未开启
    if !enabled {
        fs::write(Path::new(FPSGO).join("common/fpsgo_enable"), "1").unwrap();
        return None; // 需要重新读取
    }

    return parse_line.nth(6)?.trim().parse::<u64>().ok();
}

/* 解析需跳过第0行和最后3行，提取第三个元素
tid	bufID		name		currentFPS	targetFPS	FPS_margin	FPS_margin_GPU	FPS_margin_thrs	sbe_state	HWUI
23480	0x5b9800000038	bin.mt.plus	60		60		0		0		0		0		1
8606	0x136d0000000d	curitycenter:ui	60		60		0		0		0		0		1
23480	0x5b9800000037	bin.mt.plus	57		60		0		0		0		0		1
2756	0x9d900000001	com.miui.home	60		60		0		0		0		0		1
23480	0x5b9800000036	bin.mt.plus	0		60		0		0		0		0		1
23480	0x5b9800000035	bin.mt.plus	-1		60		0		0		0		0		1
8520	0x1d94000001a3	nputmethod.miui	-1		60		0		0		0		0		1
8520	0x1d94000001a2	nputmethod.miui	-1		60		0		0		0		0		1
23480	0x5b9800000031	bin.mt.plus	59		60		0		0		0		0		1
2898	0x9d700000001	ndroid.systemui	-1		60		0		0		0		0		1
fstb_self_ctrl_fps_enable:1
fstb_is_cam_active:0
dfps_ceiling:60 */
fn parse_fps() -> Option<Fps> {
    todo!()
}
