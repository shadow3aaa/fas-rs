use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use fas_rs_fw::prelude::*;

use super::enable_fpsgo;
use super::{BUFFER_CAP, FPSGO};

pub(super) fn frametime_thread(frametime: Arc<Mutex<Vec<FrameTime>>>, pause: Arc<AtomicBool>) {
    let mut buffer = VecDeque::with_capacity(BUFFER_CAP);

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }

        if buffer.len() > BUFFER_CAP {
            buffer.pop_back();
        }

        // 尝试复制buffer到读取区
        if let Ok(mut lock) = frametime.try_lock() {
            *lock = buffer.clone().into();
            drop(lock);
        }

        let mut stamps = [0, 0];

        // 获取第一个时间戳
        let fbt_info = fs::read_to_string(Path::new(FPSGO).join("fbt/fbt_info")).unwrap();
        if let Some(stamp) = parse_frametime(&fbt_info) {
            stamps[0] = stamp
        }

        // 轮询(sysfs不可用inotify监听)
        // 值变化后保存为第二个时间戳
        loop {
            let fbt_info = fs::read_to_string(Path::new(FPSGO).join("fbt/fbt_info")).unwrap();
            if let Some(stamp) = parse_frametime(&fbt_info) {
                if stamps[0] < stamp {
                    stamps[1] = stamp;
                    break;
                }
            } else {
                enable_fpsgo().unwrap();
                break;
            }

            // 轮询间隔6ms
            thread::sleep(Duration::from_millis(6));
        }

        // 检查是否解析失败
        if stamps[0] == 0 || stamps[1] == 0 {
            continue; // 失败就重新解析
        }

        let frametime = FrameTime::from_nanos(stamps[1] - stamps[0]);

        buffer.push_front(frametime);
    }
}

pub(super) fn fps_thread(fps: Arc<Mutex<Vec<(Instant, Fps)>>>, pause: Arc<AtomicBool>) {
    let mut buffer = VecDeque::with_capacity(BUFFER_CAP);

    loop {
        if pause.load(Ordering::Acquire) {
            thread::park();
        }

        if buffer.len() > BUFFER_CAP {
            buffer.pop_back();
        }

        thread::sleep(Duration::from_millis(10));

        // 尝试复制buffer到读取区
        if let Ok(mut lock) = fps.try_lock() {
            *lock = buffer.clone().into();
            drop(lock);
        }

        let fpsgo_status = fs::read_to_string(Path::new(FPSGO).join("fstb/fpsgo_status")).unwrap();
        if let Some(fps) = parse_fps(&fpsgo_status) {
            buffer.push_front((Instant::now(), fps));
        } else {
            enable_fpsgo().unwrap();
            continue;
        }
    }
}

/* 解析第9行:
1(状态)	0		37	19533	0x4c2e00000021	60(屏幕刷新率)	24029340996131(最新帧的vsync时间戳) */
fn parse_frametime(fbt_info: &str) -> Option<u64> {
    let mut parse_line = fbt_info.lines().nth(8)?.split_whitespace();

    let enabled = parse_line.next()?.trim().parse::<u64>().ok()? == 1;

    // fpsgo未开启
    if !enabled {
        return None; // 需要重新读取
    }

    return parse_line.nth(5)?.trim().parse::<u64>().ok();
}

/* 解析需跳过第0行和最后3行，提取第3个元素
tid	bufID		name		currentFPS	targetFPS	FPS_margin	FPS_margin_GPU	FPS_margin_thrs	sbe_state	HWUI
23480	0x5b9800000038	bin.mt.plus	60		60		0		0		0		0		1
8606	0x136d0000000d	curitycenter:ui	60		60		0		0		0		0		1
fstb_self_ctrl_fps_enable:1
fstb_is_cam_active:0
dfps_ceiling:60 */
fn parse_fps(fpsgo_status: &str) -> Option<Fps> {
    use std::cmp::max;

    let mut max_fps = None;

    for line in fpsgo_status.lines().skip(1) {
        if line.contains("fstb_self_ctrl_fps_enable")
            || line.contains("fstb_is_cam_active")
            || line.contains("dfps_ceiling")
        {
            continue;
        }

        let mut parsed_line = line.split_whitespace();
        let this_fps = parsed_line.nth(3)?;

        let this_fps = this_fps.parse().unwrap_or(0);

        if let Some(fps) = max_fps {
            max_fps = Some(max(fps, this_fps));
        } else {
            max_fps = Some(this_fps)
        }
    }

    max_fps
}

#[test]
fn test_parse() {
    let fbt_info = r"##clus	max	min	
    3	2	0	
    
    clus	num	c	r	
    0	4	-1	-1
    1	3	-1	-1
    2	1	-1	-1
    enable	idleprefer	max_blc	max_pid	max_bufID	dfps	vsync
    1	0		13	8606	0x136d00000021	120	15827015268850
    
    pid	bufid		perfidx	
    2898	0x9d700000001	0
    8606	0x136d00000021	13
    26994	0x6955000001a7	12##";
    assert_eq!(parse_frametime(fbt_info), Some(15827015268850));

    let fpsgo_status = r"##tid	bufID		name		currentFPS	targetFPS	FPS_margin	FPS_margin_GPU	FPS_margin_thrs	sbe_state	HWUI
    26994	0x69550000025d	bin.mt.plus	120		120		0		0		0		0		1
    26994	0x69550000025c	bin.mt.plus	115		120		0		0		0		0		1
    8606	0x136d0000003f	curitycenter:ui	120		120		0		0		0		0		1
    2756	0x9d9000002ce	com.miui.home	0		120		0		0		0		0		1
    2898	0x9d700000001	ndroid.systemui	-1		10		0		0		0		0		1
    24678	0x12fb00000009	m.omarea.vtools	0		10		0		0		0		0		1
    24678	0x12fb0000000b	m.omarea.vtools	0		10		0		0		0		0		1
    fstb_self_ctrl_fps_enable:1
    fstb_is_cam_active:0
    dfps_ceiling:120##";

    assert_eq!(parse_fps(fpsgo_status), Some(120));
}
