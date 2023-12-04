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
fix_codes() {
	local FIX_COMMAND='cargo clippy --fix --allow-dirty --allow-staged'

	cd $SHDIR && $FIX_COMMAND
	cd $SHDIR/fas-rs-fw && $FIX_COMMAND
	cd $SHDIR/zygisk/rust && $FIX_COMMAND
}
