/// 返回第一个当前设备支持的[`self::VirtualFrameSensor`]
#[macro_export]
macro_rules! support_sensor {
    ($($sensor: ty),*) => {
        #[allow(clippy::useless_let_if_seq)]
        {
            let result: Result<Box<dyn VirtualFrameSensor>, Box<dyn Error>>;
            $(if <$sensor>::support() {
                result = match <$sensor>::new() {
                    Ok(o) => Ok(Box::new(o)),
                    Err(e) => Err(e)
                };
            }else)* {
                result = Err("No supported sensor".into())
            }
            result
        }
    };
}

/// 返回第一个当前设备支持的[`self::VirtualPerformanceController`]
#[macro_export]
macro_rules! support_controller {
    ($($controller: ty),*) => {
        #[allow(clippy::useless_let_if_seq)]
        {
            let result: Result<Box<dyn VirtualPerformanceController>, Box<dyn Error>>;
            $(if <$controller>::support() {
                result = match  <$controller>::new() {
                    Ok(o) => Ok(Box::new(o)),
                    Err(e) => Err(e)
                };
            }else)* {
                result = Err("No supported controller".into());
            }
            result
        }
    };
}
