// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod evolution;
pub mod pid_controll;

#[derive(Debug, Copy, Clone)]
pub struct PidParams {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

impl Default for PidParams {
    fn default() -> Self {
        Self {
            kp: 0.000_3,
            ki: 0.000_03,
            kd: 0.000_003,
        }
    }
}
