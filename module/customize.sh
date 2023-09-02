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
conf=/sdcard/Android/fas-rs/games.toml
lib=/system/lib64/libsurfaceflinger.so

if [ -f $lib ]; then
	if readelf -s $lib | grep postComposition | grep -q SurfaceFlinger; then
		ui_print "Passed support test"
	else
		ui_print "Could not find the required hook point"
		abort
	fi
else
	ui_print "$lib does not exists"
	abort
fi

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
