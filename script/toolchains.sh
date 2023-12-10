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
if [ "$TERMUX_VERSION" = "" ]; then
	RR='cargo ndk -p 31 -t arm64-v8a'

	if [ "$ANDROID_NDK_HOME" = "" ]; then
		echo Missing ANDROID_NDK_HOME >&2
		exit 1
	else
		dir="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
		STRIP="$dir/llvm-strip"
	fi
else
	RR=cargo
	STRIP=strip
fi
