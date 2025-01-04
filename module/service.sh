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

MODDIR=${0%/*}
DIR=/sdcard/Android/fas-rs
MERGE_FLAG=$DIR/.need_merge
LOG=$DIR/fas_log.txt

sh $MODDIR/vtools/init_vtools.sh $(realpath $MODDIR/module.prop)

resetprop fas-rs-installed true

until [ -d $DIR ]; do
	sleep 1
done

if [ -f $MERGE_FLAG ]; then
	$MODDIR/fas-rs merge $MODDIR/games.toml >$DIR/.update_games.toml
	rm $MERGE_FLAG
	mv $DIR/.update_games.toml $DIR/games.toml
fi

killall fas-rs
RUST_BACKTRACE=1 nohup $MODDIR/fas-rs run $MODDIR/games.toml >$LOG 2>&1 &
