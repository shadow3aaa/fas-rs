#!/system/bin/sh
MODDIR=${0%/*}

chmod a+x $MODDIR/fas-rs
nohup $MODDIR/fas-rs >/dev/null 2>&1
