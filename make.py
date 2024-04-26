#!/bin/python3
# Copyright 2023 shadow3aaa@gitbub.com
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import sys
import maketools.format_codes as format_codes
import maketools.build as build
import maketools.fix as fix
import maketools.update as update
from maketools.misc import eprint

help_text = """\
./make.py:
    build:
        build and package fas-rs module
        sugg: try ./make.sh build --help to get details
    format:
        format codes of fas-rs
    fix:
        fix codes of fas-rs
    update:
        recursive update all depended crates
    help:
        print this help\
"""

try:
    arg = sys.argv[1]
except IndexError:
    eprint("Missing argument")
    eprint(help_text)
    exit(-1)

match arg:
    case "help":
        print(help_text)
    case "fmt" | "format":
        format_codes.task()
    case "build":
        build.task(sys.argv[2:])
    case "fix":
        fix.task()
    case "update":
        update.task()
    case _:
        eprint("Invalid argument")
        eprint(help_text)
        exit(-1)
