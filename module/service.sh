#!/system/bin/sh
# Copyright 2023 shadow3aaa@gitbub.com
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

MODDIR=${0%/*}
DIR=/sdcard/Android/fas-rs
MERGE_FLAG=$DIR/.need_merge
LOG=$DIR/fas_log.txt

# $1:value $2:path
lock_val() {
	umount $2
	chmod +w $2

	echo "$1" | tee /dev/fas_rs_mask $2
	/bin/find $2 -exec mount /dev/fas_rs_mask {} \;
	rm /dev/fas_rs_mask
}

sh $MODDIR/vtools/init_vtools.sh $(realpath $MODDIR/module.prop)

resetprop fas-rs-installed true

until [ -d $DIR ]; do
	sleep 1
done

stop vendor.oplus.ormsHalService-aidl-default

for i in $(seq 0 2); do
	lock_val "$i 20000" /proc/shell-temp
done

if [ -f $MERGE_FLAG ]; then
	$MODDIR/fas-rs merge $MODDIR/games.toml >$DIR/.update_games.toml
	rm $MERGE_FLAG
	mv $DIR/.update_games.toml $DIR/games.toml
fi

killall fas-rs
nohup $MODDIR/fas-rs run $MODDIR/games.toml >$LOG 2>&1 &
