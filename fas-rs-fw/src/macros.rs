/// 返回第一个当前设备支持的[`self::VirtualFrameSensor`]
#[macro_export]
macro_rules! support_sensor {
    ($($sensor: ty),*) => {
        {
            let result: Box<dyn VirtualFrameSensor>;
            $(if <$sensor>::support() {
                result = Box::new(<$sensor>::new().unwrap());
            }else)* {
                eprintln!("No supported sensor");
                std::process::exit(1);
            }
            result
        }
    };
}

/// 返回第一个当前设备支持的[`self::VirtualPerformanceController`]
#[macro_export]
macro_rules! support_controller {
    ($($controller: ty),*) => {
        {
            let result: Box<dyn VirtualPerformanceController>;
            $(if <$controller>::support() {
                result = Box::new(<$controller>::new().unwrap());
            }else)* {
                eprintln!("No supported controller");
                std::process::exit(1);
            }
            result
        }
    };
}
