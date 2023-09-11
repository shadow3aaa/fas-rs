use std::time::Duration;

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
use binder_rust::{BinderService, Parcel, ServiceManager};
use log::error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::policy::PerformanceControllerExt;
use crate::{Config, PerformanceController};

pub struct FrameAwareService<P: PerformanceControllerExt> {
    config: Config,
    controller: P,
}

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq, Eq)]
enum MessageCode {
    FrameData = 1,
}

impl<P: PerformanceControllerExt> BinderService for FrameAwareService<P> {
    fn process_request(&self, code: u32, data: &mut Parcel) -> Parcel {
        let reply = Parcel::empty();

        let Some(code) = MessageCode::from_u32(code) else {
            return reply;
        };

        if code == MessageCode::FrameData {
            let Ok(frametime) = data.read_u32() else {
                return reply;
            };
        
            let Ok(pkg) = data.read_str16() else {
                return reply;
            };

            if let Some(_fps) = self.config.target_fps(pkg) {
                let frametime = Duration::from_nanos(frametime.into());
                self.controller
                    .do_policy(frametime)
                    .unwrap_or_else(|e| error!("{e:?}"));
            }
        }

        reply
    }
}

impl<P: PerformanceController + PerformanceControllerExt> FrameAwareService<P> {
    pub fn run_server(config: Config, controller: P) -> ! {
        let server = Self { config, controller };
        let manager = &mut ServiceManager::new().unwrap();

        manager
            .register_service(
                &server,
                "FrameAwareService",
                "com.shadow3.FrameAwareService",
                true,
                0,
            )
            .unwrap()
            .run()
            .unwrap();
        unreachable!() // run is a forover loop, so
    }
}
