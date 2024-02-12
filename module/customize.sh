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
DIR=/data/media/0/Android/fas-rs
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

if [ $ARCH != arm64 ]; then
	local_print "设备不支持, 非arm64设备" "Only for arm64 device !"
	abort
elif [ $API -le 30 ]; then
	local_print "系统版本过低, 需要安卓12及以上的系统版本版本" "Required A12+ !"
	abort
elif [[ ! $KSU == true && $MAGISK_VER_CODE -lt 24000 ]]; then
	local_print "Magisk版本过低, 需要Magisk v24.0及以上的Magisk版本" "Required Magisk v24.0+, since we use Zygisk v2"
	abort
fi

local_print "
    免责声明和警告：

    fas-rs 目前通过 Zygisk 注入游戏，拦
    截 libgui 的函数调用以获取帧渲染间隔。

    但是，这也可能导致 root 检测或目标游
    戏的注入检测被触发。

    当这种情况发生时，你应该自己承担风险。
    
    同时，我保证 fas-rs 的Zygisk 注入仅
    用于获取帧时间以进行帧感知调度。这个项
    目将永远是开源的，任何人都可以查看它以
    确认其中确实没有恶意代码。
" "
    Disclaimer & Warning:
    
    fas-rs currently injects the game via Zygisk,
    intercepting libgui's function calls to get
    frame rendering time.
    
    However, this may also causes root detection
    or injection detection of the target game
    to be triggered.
    
    So when this happens,
    YOU SHOULD TAKE RESPONSIBILITY YOURSELF.
    
    At the same time, I guarantee that fas-rs' s
    zygisk injection is only for the sole purpose
    of fetching frametime for frame-aware scheduling.
    This project will always be open source,
    and anyone can review it to confirm that
    there is indeed no malicious code in it.
"

if [ -f $CONF ]; then
	touch $MERGE_FLAG
else
	mkdir -p $DIR
	cp $MODPATH/games.toml $CONF
fi

mv -f $MODPATH/README.md $DIR/doc_cn.md
mv -f $MODPATH/README_EN.md $DIR/doc_en.md

sh $MODPATH/vtools/init_vtools.sh $(realpath $MODPATH/module.prop)

set_perm_recursive $MODPATH 0 0 0755 0644
set_perm $MODPATH/fas-rs 0 0 0755

local_print "配置文件夹：/sdcard/Android/fas-rs" "Configuration folder: /sdcard/Android/fas-rs"

resetprop fas-rs-installed true
