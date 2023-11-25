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
NOARG=true
HELP=false
CLEAN=false
DEBUG_BUILD=false
RELEASE_BUILD=false
VERBOSE=false
CFLAGS="
-O3 -flto -fmerge-all-constants -fno-exceptions -fomit-frame-pointer -fshort-enums
-Wl,-O3,--lto-O3,--gc-sections,--as-needed,--icf=all,-z,norelro,--pack-dyn-relocs=android+relr
-std=c++2b -Wall -lc++"

rm -rf $SHDIR/output
mkdir -p $SHDIR/output

if [ "$TERMUX_VERSION" = "" ]; then
	alias RR='cargo ndk -t arm64-v8a'
	export CARGO_NDK_ANDROID_PLATFORM=31

	if [ "$ANDROID_NDK_HOME" = "" ]; then
		echo Missing ANDROID_NDK_HOME
		exit 1
	else
		dir="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
		alias clang++="$dir/aarch64-linux-android31-clang++"
		clang++ -v
	fi
else
	alias RR=cargo
fi

for arg in $@; do
	case $arg in
	clean | --clean)
		CLEAN=true
		;;
	r | -r | release | --release)
		RELEASE_BUILD=true
		;;
	d | -d | debug | --debug)
		DEBUG_BUILD=true
		;;
	-h | h | help | --help)
		HELP=true
		;;
	v | -v | verbose | --verbose)
		VERBOSE=true
		;;
	*)
		echo Illegal parameter: $arg >&2
		echo Try \'./build.sh --help\' >&2
		exit 1
		;;
	esac

	NOARG=false
done

set -e

if $HELP || $NOARG; then
	echo -n "./build.sh:
    --release / release / -r / r:
        release build
    --debug / debug / -d / d:
        debug build
    --verbose / verbose / -v / v:
        print details of build"

	exit
elif $CLEAN; then
	cd $SHDIR/rust
	cargo clean

	cd $SHDIR
	rm -rf output

	exit
fi

if $DEBUG_BUILD; then
	if $VERBOSE; then
		cd $SHDIR/rust
		RR build --target aarch64-linux-android -v

		cd $SHDIR
		cp -f rust/target/aarch64-linux-android/debug/librust.a output/librust.a
		clang++ -v --shared src/*.cpp \
			-I rust/include \
			-L output -L ../prebuilt \
			-fPIC -nostdlib++ -Wl,-lrust,-llog,-lbinder_ndk \
			$CFLAGS \
			-o output/arm64-v8a.so
	else
		cd $SHDIR/rust
		RR build --target aarch64-linux-android

		cd $SHDIR
		cp -f rust/target/aarch64-linux-android/debug/librust.a output/librust.a
		clang++ --shared src/*.cpp \
			-I rust/include \
			-L output -L ../prebuilt \
			-fPIC -nostdlib++ -Wl,-lrust,-llog,-lbinder_ndk \
			$CFLAGS \
			-o output/arm64-v8a.so
	fi
fi

if $RELEASE_BUILD; then
	if $VERBOSE; then
		cd $SHDIR/rust
		RR build --release --target aarch64-linux-android -v

		cd $SHDIR
		cp -f rust/target/aarch64-linux-android/release/librust.a output/librust.a
		clang++ -v --shared src/*.cpp \
			-I rust/include \
			-L output -L ../prebuilt \
			-fPIC -nostdlib++ -Wl,-lrust,-llog,-lbinder_ndk \
			$CFLAGS \
			-o output/arm64-v8a.so
	else
		cd $SHDIR/rust
		RR build --release --target aarch64-linux-android

		cd $SHDIR
		cp -f rust/target/aarch64-linux-android/release/librust.a output/librust.a
		clang++ --shared src/*.cpp \
			-I rust/include \
			-L output -L ../prebuilt \
			-fPIC -nostdlib++ -Wl,-lrust,-llog,-lbinder_ndk \
			$CFLAGS \
			-o output/arm64-v8a.so
	fi
fi
