#!/usr/bin/bash
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

BASEDIR="$(dirname $(readlink -f "$0"))"
BIN=${BASEDIR}/target/aarch64-linux-android/release/fas-rs

set -e

cargo b -r --target aarch64-linux-android

if [ ! -f $BIN ]; then
	echo "Fail to build release"
	ls "${BASEDIR}"
	exit 1
fi

echo -e "Build successed"
cp -f $(realpath $BIN) "${BASEDIR}/build_module/"

cd "${BASEDIR}/build_module/"
zip -9 -rq ../fas-rs.zip .

echo -n "Packaging is complete: $(realpath ${BASEDIR}/fas-rs.zip)"
