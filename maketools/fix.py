#!/bin/python3
#
# Copyright 2023 shadow3aaa@gitbub.com
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
import os
from maketools.toolchains import cargo
from pathlib import Path


def __clippy_fix():
    cargo("clippy --fix --allow-dirty --allow-staged --target aarch64-linux-android")
    cargo(
        "clippy --fix --allow-dirty --allow-staged --target aarch64-linux-android --release"
    )


def task():
    __clippy_fix()

    zygisk = Path("zygisk").joinpath("rust")
    os.chdir(zygisk)

    __clippy_fix()
