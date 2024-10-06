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

DIR=/sdcard/Android/fas-rs
CONF=$DIR/games.toml
MERGE_FLAG=$DIR/.need_merge
LOCALE=$(getprop persist.sys.locale)

local_print() {
	if [ $LOCALE = zh-CN ]; then
		ui_print "$1"
	else
		ui_print "$2"
	fi
}

local_echo() {
	if [ $LOCALE = zh-CN ]; then
		echo "$1"
	else
		echo "$2"
	fi
}

if [ $ARCH != arm64 ]; then
	local_print "设备不支持, 非arm64设备" "Only for arm64 device !"
	abort
elif [ $API -le 30 ]; then
	local_print "系统版本过低, 需要安卓12及以上的系统版本版本" "Required A12+ !"
	abort
elif uname -r | awk -F. '{if ($1 < 5 || ($1 == 5 && $2 < 10)) exit 0; else exit 1}'; then
	local_print "内核版本过低，需要5.10或以上 !" "The kernel version is too low. Requires 5.10+ !"
	abort
fi

if [ -f $CONF ]; then
	touch $MERGE_FLAG
else
	mkdir -p $DIR
	cp $MODPATH/games.toml $CONF
fi

cp -f $MODPATH/README.md $DIR/doc_cn.md
cp -f $MODPATH/README_EN.md $DIR/doc_en.md

sh $MODPATH/vtools/init_vtools.sh $(realpath $MODPATH/module.prop)

set_perm_recursive $MODPATH 0 0 0755 0644
set_perm $MODPATH/fas-rs 0 0 0755

local_print "配置文件夹：/sdcard/Android/fas-rs" "Configuration folder: /sdcard/Android/fas-rs"
local_echo "updateJson=https://github.com/shadow3aaa/fas-rs/raw/master/update/update.json" "updateJson=https://github.com/shadow3aaa/fas-rs/raw/master/update/update_en.json" >>$MODPATH/module.prop

resetprop fas-rs-installed true
