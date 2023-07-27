/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
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
                result = match <$controller>::new() {
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
