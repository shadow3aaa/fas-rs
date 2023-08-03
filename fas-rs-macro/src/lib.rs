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

/// 生成Scheduler
#[macro_export]
macro_rules! get_scheduler {
    ($($sensor: ty),*; $($controller: ty),*) => {
        {
            use std::process;
            use log::error;

            let s_result: Box<dyn VirtualFrameSensor>;
            $(if <$sensor>::support() {
                s_result = Box::new(<$sensor>::new().unwrap());
            } else )* {
                error!("No supported sensor");
                process::exit(1);
            }

            let c_result: Box<dyn VirtualPerformanceController>;
            $(if <$controller>::support() {
                c_result = Box::new(<$controller>::new().unwrap());
            } else )* {
                error!("No supported controller");
                process::exit(1);
            }

            Scheduler::new(s_result, c_result).unwrap()
        }
    }
}

/// 检测是否支持
#[macro_export]
macro_rules! support {
    ($($sensor: ty),*; $($controller: ty),*) => {
        {
            let s_result = $(<$sensor>::support() || )* false;
            let c_result = $(<$controller>::support() || )* false;

            c_result && s_result
        }
    }
}

/// 启动模块
#[macro_export]
macro_rules! run_modules {
    ($scheduler: expr; $($module: ty),*) => {
        {
            use std::thread;

            $(if <$module>::support() {
                let mut module = <$module>::new();
                thread::Builder::new()
                    .name(<$module>::NAME.into())
                    .spawn(move || loop {
                        module.run(&CONFIG, &NODE)
                    })
                    .unwrap();
            })*

            $scheduler.load_loop()
        }
    }
}
