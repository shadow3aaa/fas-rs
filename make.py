#!/bin/python3
# Copyright 2023-2025, shadow3 (@shadow3aaa)
#
# This file is part of fas-rs.
#
# fas-rs is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option)
# any later version.
#
# fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
# details.
#
# You should have received a copy of the GNU General Public License along
# with fas-rs. If not, see <https://www.gnu.org/licenses/>.

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
