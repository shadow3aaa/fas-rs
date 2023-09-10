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
    NotPermitted = 2,
}

struct FrameData {
    pub frametime: u64,
    pub package_name: String,
}

impl<P: PerformanceControllerExt> BinderService for FrameAwareService<P> {
    fn process_request(&self, code: u32, data: &mut Parcel) -> Parcel {
        let code = MessageCode::from_u32(code).unwrap();
        if code == MessageCode::FrameData {
            let frame_data: FrameData = data.read_object();
            if let Some(fps) = self.config.target_fps(frame_data.package_name) {
                let frametime = Duration::from_nanos(frame_data.frametime);
                self.controller.do_policy(frametime);
            }
        }
        Parcel::empty()
    }
}

impl<P: PerformanceController + PerformanceControllerExt> FrameAwareService<P> {
    pub fn run_server(config: Config, controller: P) -> ! {
        let server = Self { config, controller };
        let mut manager = &mut ServiceManager::new();

        manager
            .register_service(
                &server,
                "FrameAwareService",
                "com.shadow3.FrameAwareService",
                true,
                0,
            )
            .run();
        unreachable!() // run is a forover loop, so
    }
}
