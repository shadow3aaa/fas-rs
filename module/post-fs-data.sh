#!/system/bin/sh
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
MODDIR=${0%/*}
dir=/sdcard/Android/fas-rs

# start with std profile
nohup env FAS_LOG=info $MODDIR/fas-rs --local-profile $MODDIR/games.toml --std-profile $MODDIR/games.toml --run >$MODDIR/init_log.txt 2>&1 &

# so it won't block post-data
{
	# wait until the sdcard is decrypted
	until [ -d $dir ]; do
		sleep 1
	done

	# start with user profile
	killall fas-rs
	nohup env FAS_LOG=info $MODDIR/fas-rs --run --local-profile $dir/games.toml --std-profile $MODDIR/games.toml >$dir/fas_log.txt 2>&1 &
} &
