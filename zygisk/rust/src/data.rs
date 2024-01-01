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
use std::time::Instant;

use dobby_api::Address;
use libc::c_int;

#[derive(Debug, Clone, Copy)]
pub struct Data {
    pub buffer: Address,
    pub instant: Instant,
    pub cpu: c_int,
}

unsafe impl Sync for Data {}
unsafe impl Send for Data {}
