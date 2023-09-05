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

chmod +x $MODDIR/hook/install_hook.sh
$MODDIR/hook/install_hook.sh

# wait until the sdcard is decrypted
until [ -d "/sdcard/Android" ]; do
	sleep 1
done

nohup env FAS_LOG=info $MODDIR/fas-rs --run >$dir/fas_log.txt 2>&1 &

# vtools support
sh $MODDIR/vtools/init_vtools.sh $(realpath $MODDIR/module.prop)
