use std::{fmt::Debug, process};

pub trait ThisResult<T, E> {
    /// Result附加方法, 类似[`Result::unwrap`]，但是更加精简
    ///
    /// 行为: 直接退出，错误码1
    fn this_unwrap(self) -> T;
}

impl<T, E: Debug> ThisResult<T, E> for Result<T, E> {
    fn this_unwrap(self) -> T {
        match self {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{e:?}");
                process::exit(1)
            }
        }
    }
}

pub trait ThisOption<T> {
    /// Option附加方法, 类似[`Option::unwrap`]，但是更加精简
    ///
    /// 行为: 打印"Null error occurred"直接退出，错误码1
    fn this_unwrap(self) -> T;
}

impl<T> ThisOption<T> for Option<T> {
    fn this_unwrap(self) -> T {
        self.map_or_else(
            || {
                eprintln!("Null error occurred");
                process::exit(1);
            },
            |o| o,
        )
    }
}
