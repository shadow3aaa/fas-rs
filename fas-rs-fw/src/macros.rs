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
