#!/system/bin/sh
MODDIR=${0%/*}

chmod a+x $MODDIR/fas-rs

until [ -d "/sdcard/Android" ]; do
	sleep 1
done

if lsmod | grep -qE "perfmgr_mtk|ged_novsync"; then
	touch $MODDIR/disable
	exit
fi

dir=/sdcard/Android/fas-rs

killall fas-rs
nohup env FAS_LOG=info $MODDIR/fas-rs > $dir/fas_log.txt 2>&1 &
