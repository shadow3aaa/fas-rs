use std::{fmt::Debug, hint::unreachable_unchecked};

pub trait ThisResult<T, E> {
    /// Result附加方法, 类似[`Result::unwrap`]，但是更加精简
    ///
    /// release构建: 输出错误信息到[`std::io::stderr`]
    ///
    /// debug构建: 输出错误信息到[`std::io::stderr`]，然后painc展开错误位置
    fn this_unwrap(self) -> T;
}

impl<T, E: Debug> ThisResult<T, E> for Result<T, E> {
    fn this_unwrap(self) -> T {
        match self {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{e:?}");
                use std::process;
                process::exit(1)
            }
        }
    }
}

pub trait ThisOption<T> {
    /// Option附加方法, 类似[`Option::unwrap`]，但是更加精简
    ///
    /// release构建: 直接退出，错误码1
    ///
    /// debug构建: painc展开错误位置
    fn this_unwrap(self) -> T;
}

impl<T> ThisOption<T> for Option<T> {
    fn this_unwrap(self) -> T {
        self.map_or_else(
            || {
                #[cfg(debug_assertions)]
                {
                    use std::panic::Location;
                    let location = Location::caller();
                    panic!(
                        "Errors occurred at file '{}', line {}",
                        location.file(),
                        location.line()
                    );
                }
                #[cfg(release_assertions)]
                {
                    use std::process;
                    process::exit(1);
                }
                #[allow(unreachable_code)]
                unsafe {
                    unreachable_unchecked()
                }
            },
            |o| o,
        )
    }
}
