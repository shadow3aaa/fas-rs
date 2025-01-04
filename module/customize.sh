#!/system/bin/sh
# Copyright 2023-2025, shadow3 (@shadow3aaa)
#
# This file is part of fas-rs.
#
# fas-rs is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option)
# any later version.
#
# fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
# details.
#
# You should have received a copy of the GNU General Public License along
# with fas-rs. If not, see <https://www.gnu.org/licenses/>.

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

cp -f $MODPATH/README_CN.md $DIR/doc_cn.md
cp -f $MODPATH/README_EN.md $DIR/doc_en.md

sh $MODPATH/vtools/init_vtools.sh $(realpath $MODPATH/module.prop)

set_perm_recursive $MODPATH 0 0 0755 0644
set_perm $MODPATH/fas-rs 0 0 0755

local_print "配置文件夹：/sdcard/Android/fas-rs" "Configuration folder: /sdcard/Android/fas-rs"
local_echo "updateJson=https://github.com/shadow3aaa/fas-rs/raw/master/update/update.json" "updateJson=https://github.com/shadow3aaa/fas-rs/raw/master/update/update_en.json" >>$MODPATH/module.prop

resetprop fas-rs-installed true
