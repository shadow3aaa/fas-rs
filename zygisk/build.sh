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

if [ "$TERMUX_VERSION" = "" ]; then
    alias cargo='cargo ndk -t arm64-v8a'
fi

mkdir -p $SHDIR/output

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

    if [ "$TERMUX_VERSION" = "" ]; then
        xmake f -p android -a arm64-v8a -m release
    fi

    xmake
    ;;
d | -d | debug | --debug)
    cd $SHDIR/rust
    cargo build

    cd $SHDIR
    cp -f rust/target/aarch64-linux-android/debug/librust.a output/librust.a

    if [ "$TERMUX_VERSION" = "" ]; then
        xmake f -p android -a arm64-v8a -m debug
    fi

    xmake
    ;;
esac
