pub mod prelude;
mod scheduler;

use std::{error::Error, time::Duration};

pub use scheduler::Scheduler;

/// 这里的[`self::FrameTime`]可能是 `帧渲染间隔` 或 `帧渲染时间`
/// 一般来说, 后者比较难从系统获得
pub type FrameTime = Duration;
pub type TargetFps = u32;
pub type Fps = u32;

/// 帧传感器接口
/// `Frame Aware` 意为感知帧变化
/// 目前没有发现通用且高效的获取[`self::FrameTime`]方法, 需要针对不同设备实现
pub trait VirtualFrameSensor: Send {
    /// 设备是否支持此实现
    fn support() -> bool
    where
        Self: Sized;
    /// 务必在此实现构造函数
    /// 初始化操作(比如创建线程/任务也要在这里完成)
    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    /// 获取指定数量的历史[`self::FrameTime`]的平均数
    fn frametimes(&self, count: usize, target_fps: TargetFps) -> Vec<FrameTime>;
    /// 获取指定时间内的历史[`self::Fps`]
    fn fps(&self, time: Duration) -> Vec<Fps>;
    /// 很多时候, 监视帧状态是开销较大的
    /// 因此[`self::Scheduler`]在每次从调度中退出后
    /// 会调用此方法关闭监视
    fn pause(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    /// 实现了暂停监视自然还要实现恢复监视
    /// [`self::Scheduler`]在每次从调度开始时调用此方法
    fn resume(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

/// 性能控制器接口
/// 控制设备性能状态的控制器
pub trait VirtualPerformanceController: Send {
    /// 设备是否支持此实现
    fn support() -> bool
    where
        Self: Sized;
    /// 务必在此实现构造函数
    /// 因为会被[`self::support_controller`]调用创建实例
    /// 初始化操作(比如创建线程/任务也要在这里完成)
    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    /// 限制一级性能
    fn limit(&self);
    /// 释放一级性能
    fn release(&self);
    /// 有时候控制器需要一些操作初始化才可用
    /// [`self::Scheduler`]每次开始调度的时候会调用此方法初始化(插入)控制器
    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    /// 有时候需要还原(拔出)控制器
    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

/// 返回第一个当前设备支持的[`self::VirtualFrameSensor`]
#[macro_export]
macro_rules! support_sensor {
    ($($sensor: ty: VirtualFrameSensor),*) => {
        {
            let result: Result<<$sensor>, ()>;
            $(if <$sensor>::support() {
                result = <$sensor>::new();
            }else)* {
                result = Err(());
            }
            result
        }
    };
}

/// 返回第一个当前设备支持的[`self::VirtualPerformanceController`]
#[macro_export]
macro_rules! support_controller {
    ($($controller: ty: VirtualPerformanceController),*) => {
        {
            let result: Result<<$controller>, ()>;
            $(if <$controller>::support() {
                result = <$controller>::new();
            }else)* {
                result = Err(());
            }
            if let Ok(o) = result {
                o
            } else {
                Err(())
            }
        }
    };
}
