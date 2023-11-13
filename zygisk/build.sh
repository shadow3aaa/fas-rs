#!/bin/sh
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
CFLAGS="
-O3 -flto -fmerge-all-constants -fno-exceptions -fomit-frame-pointer -fshort-enums
-Wl,-O3,--lto-O3,--gc-sections,--as-needed,--icf=all,-z,norelro,--pack-dyn-relocs=android+relr
-std=c++2b -Wall -lc++"

if [ "$TERMUX_VERSION" = "" ]; then
	alias cargo='cargo ndk -t arm64-v8a'

	if [ "$ANDROID_NDK_HOME" = "" ]; then
		echo Missing ANDROID_NDK_HOME
		exit 1
	else
		dir=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
		clang_latest=$(ls $dir | grep aarch64-linux-android | grep clang | tail -n 1)

		echo Find clang: $dir/$clang_latest
		alias clang++="$dir/$clang_latest"
		clang++ -v
	fi
fi

mkdir -p $SHDIR/output

set -e

case $1 in
clean | --clean)
	cd $SHDIR/rust
	cargo clean

	cd $SHDIR
	rm -rf output

	exit
	;;
r | -r | release | --release)
	cd $SHDIR/rust
	cargo build --release

	cd $SHDIR
	cp -f rust/target/aarch64-linux-android/release/librust.a output/librust.a

	clang++ --shared src/*.cpp \
		-I rust/include \
		-L output -L ../prebuilt \
		-fPIC -nostdlib++ -Wl,-lrust,-llog,-lbinder_ndk \
		$CFLAGS \
		-o output/arm64-v8a.so
	;;
d | -d | debug | --debug)
	cd $SHDIR/rust
	cargo build

	cd $SHDIR
	cp -f rust/target/aarch64-linux-android/debug/librust.a output/librust.a

	clang++ --shared src/*.cpp \
		-I rust/include \
		-L output -L ../prebuilt \
		-fPIC -nostdlib++ -Wl,-lrust,-llog,-lbinder_ndk \
		$CFLAGS \
		-o output/arm64-v8a.so
	;;
esac
