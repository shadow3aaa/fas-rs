#!/system/bin/sh
MODDIR=${0%/*}
dir=/sdcard/Android/fas-rs

# wait until the sdcard is decrypted
until [ -d "/sdcard/Android" ]; do
	sleep 1
done

# detect conflicting kernel modules
if lsmod | grep -qE "perfmgr_mtk|ged_novsync"; then
	touch $MODDIR/disable
	exit
fi

killall fas-rs
nohup env FAS_LOG=info $MODDIR/fas-rs >$dir/fas_log.txt 2>&1 &
