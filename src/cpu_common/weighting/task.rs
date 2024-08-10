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

use anyhow::Result;
use cpu_instructions_reader::{InstructionNumberInstant, InstructionNumberReader};
use libc::pid_t;

#[derive(Debug)]
pub struct TaskMeta {
    pub weight: f64,
    pub instructions_trace: Vec<InstructionNumberInstant>,
    pub instructions_reader: InstructionNumberReader,
}

impl TaskMeta {
    pub fn new(tid: pid_t, num_cpus: usize) -> Result<Self> {
        let instructions_reader = InstructionNumberReader::new(Some(tid))?;
        let mut instructions_trace = Vec::new();

        for cpu in 0..num_cpus {
            instructions_trace.push(instructions_reader.instant(cpu as i32)?);
        }

        Ok(Self {
            weight: 0.0,
            instructions_reader,
            instructions_trace,
        })
    }
}
