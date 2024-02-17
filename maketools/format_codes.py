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
from pathlib import Path
from maketools.toolchains import clang_format


def task():
    os.system("ruff format make.py")
    os.system("ruff format maketools")

    os.system("shfmt -s -w -p {}".format(Path("module").joinpath("*.sh")))

    os.system("cargo fmt -v")

    os.chdir("zygisk")
    cpp_src = Path("src").joinpath("*")
    clang_format("-i --verbose {}".format(cpp_src))

    os.chdir("rust")
    os.system("cargo fmt -v")
