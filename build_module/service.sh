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

killall fas-rs
export FAS_LOG=info
nohup $MODDIR/fas-rs > $MODDIR/fas_log.txt &
