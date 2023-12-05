#!/bin/bash
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
SHDIR="$(dirname $(readlink -f "$0"))"
SCRIPT=$SHDIR/script

case $1 in
build)
	source $SCRIPT/build.sh
	shift
	build $@
	;;
format | fmt)
	source $SCRIPT/format.sh
	format_codes
	;;
fix)
	source $SCRIPT/fix.sh
	fix_codes
	;;
help)
	echo "./make.sh:
    build:
        build and package fas-rs module
        sugg: try ./make.sh build --help to get details
    format:
        format codes of fas-rs
    fix:
        fix codes of fas-rs"
	;;
*)
	echo Illegal parameter: $1 >&2
	echo Try \'./make.sh help\' >&2
	exit 1
	;;
esac
