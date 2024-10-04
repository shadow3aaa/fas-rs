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

use std::time::Duration;

use anyhow::Result;
use likely_stable::unlikely;
use rand::Rng;
use rusqlite::{params, Connection};

use crate::{framework::scheduler::looper::buffer::Buffer, Config, Mode};

use super::PidParams;

pub const DATABASE_PATH: &str = "/sdcard/Android/fas-rs/database.db";

pub fn open_database() -> Result<Connection> {
    let conn = Connection::open(DATABASE_PATH)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pid_params (
            id TEXT PRIMARY KEY,
            kp REAL NOT NULL,
            ki REAL NOT NULL,
            kd REAL NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn load_pid_params(conn: &Connection, package_name: &str) -> Result<PidParams> {
    let mut stmt = conn.prepare("SELECT kp, ki, kd FROM pid_params WHERE id = ?1")?;

    let params = stmt.query_row(params![package_name], |row| {
        Ok(PidParams {
            kp: row.get(0)?,
            ki: row.get(1)?,
            kd: row.get(2)?,
        })
    })?;

    Ok(params)
}

pub fn save_pid_params(conn: &Connection, package_name: &str, pid_params: PidParams) -> Result<()> {
    conn.execute(
        "INSERT INTO pid_params (id, kp, ki, kd) 
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET 
            kp = excluded.kp, 
            ki = excluded.ki, 
            kd = excluded.kd",
        params![package_name, pid_params.kp, pid_params.ki, pid_params.kd,],
    )?;
    Ok(())
}

pub fn mutate_params(params: PidParams) -> PidParams {
    let mut rng = rand::thread_rng();
    PidParams {
        kp: (params.kp + rng.gen_range(-0.000_1..0.000_1)).clamp(0.000_4, 0.000_8),
        ki: (params.ki + rng.gen_range(-0.000_01..0.000_01)).clamp(0.000_015, 0.000_08),
        kd: (params.kd + rng.gen_range(-0.000_001..0.000_001)).clamp(0.000_025, 0.000_035),
    }
}

pub fn evaluate_fitness(buffer: &Buffer, config: &mut Config, mode: Mode) -> Option<f64> {
    let target_fps = buffer.target_fps?;

    if unlikely(buffer.frametimes.len() < target_fps.try_into().unwrap()) {
        return None;
    }

    let margin = config.mode_config(mode).margin;
    let margin = Duration::from_millis(margin);
    let target = Duration::from_secs(1) + margin;

    let fitness_frametime = buffer
        .frametimes
        .iter()
        .copied()
        .map(|frametime| frametime * target_fps)
        .map(|frametime| (frametime.as_nanos() as f64 - target.as_nanos() as f64).powi(2))
        .sum::<f64>()
        / buffer.frametimes.len() as f64
        * -1.0;

    Some(fitness_frametime)
}
