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
SKIPUNZIP=0
DIR=/data/media/0/Android/fas-rs
CONF=$DIR/games.toml
MERGE_FLAG=$DIR/.need_merge

[ -v $KSU ] && KSU=false

if [ $ARCH != "arm64" ]; then
	ui_print "Only for arm64 device !"
	abort
elif [ $API -le 30 ]; then
	ui_print "Required A12+ !"
	abort
elif ! $KSU && [ $MAGISK_VER_CODE -lt 26000 ]; then
	ui_print "Required magisk v26.0+, since we use zygisk api v4"
	abort
elif ! $KSU && [ $ZYGISK_ENABLED -ne 1 ]; then
	ui_print "Required zygisk option to be opened"
	abort
elif $KSU; then
	ui_print "⚠ KSU detected, make sure you has installed zygisk-on-ksu"
fi

# warnings
ui_print "
    Disclaimer & Warning:
    
    FAS-RS currently injects the game via Zygisk,
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

cp -f $MODPATH/README.md $DIR/doc.md

sh $MODPATH/vtools/init_vtools.sh $(realpath $MODPATH/module.prop)

set_perm_recursive $MODPATH 0 0 0755 0644
set_perm $MODPATH/fas-rs 0 0 0755

resetprop fas_rs_installed true

ui_print "Configuration folder: /sdcard/Android/fas-rs"
