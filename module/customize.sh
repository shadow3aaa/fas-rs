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
dir=/sdcard/Android/fas-rs
conf=$dir/games.toml

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
    
    At the same time, I guarantee that fas-rs'
    zygisk injection is only for the sole purpose
    of fetching frametime for frame-aware scheduling.
    This project will always be open source,
    and anyone can review it to confirm that
    there is indeed no malicious code in it.
"

# permission
chmod +x $MODPATH/fas-rs

if [ -f $conf ]; then
	# merge local std
	$MODPATH/fas-rs --merge --local-profile $conf --std-profile $MODPATH/games.toml
else
	# creat new config
	mkdir -p $dir
	cp $MODPATH/games.toml $conf
fi

cp -f $MODPATH/README.md $dir/doc.md

# vtools support
sh $MODPATH/vtools/init_vtools.sh $(realpath $MODPATH/module.prop)
