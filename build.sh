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
TEMPDIR=$SHDIR/output/.temp

if [ "$TERMUX_VERSION" = "" ]; then
	alias cargo='cargo ndk -t arm64-v8a'
fi

cd $SHDIR
mkdir -p output
cp -r module output/.temp

set -e

case $1 in
clean | --clean)
	rm -rf output
	zygisk/build.sh --clean

	exit
	;;
r | -r | release | --release)
	cargo build --release
	zygisk/build.sh --release

	cp -f target/aarch64-linux-android/release/fas-rs $TEMPDIR/fas-rs
	cp -f zygisk/output/arm64-v8a.so $TEMPDIR/zygisk/arm64-v8a.so

	strip $TEMPDIR/fas-rs
	strip $TEMPDIR/zygisk/arm64-v8a.so

	cd $TEMPDIR
	zip -9 -rq ../fas-rs.zip .

	echo Flashable Module Packaged: output/fas-rs.zip
	;;
d | -d | debug | --debug)
	cargo build --debug
	zygisk/build.sh --debug

	cp -f target/aarch64-linux-android/debug/fas-rs $TEMPDIR/fas-rs
	cp -f zygisk/output/arm64-v8a.so $TEMPDIR/zygisk/arm64-v8a.so

	cd $TEMPDIR
	zip -9 -rq ../fas-rs.zip .

	echo Flashable Module Packaged: output/fas-rs.zip
	;;
*)
	echo -n "Help:
    build.sh --release / release / -r / r:
        release build
    build.sh --debug / debug / -d / d:
        debug build"
	;;
esac
