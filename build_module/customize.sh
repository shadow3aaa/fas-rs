SKIPUNZIP=0

chmod a+x $MODPATH/fas-rs
if [ ! $($MODPATH/fas-rs "test") == "Supported" ]; then
	abort
fi

if lsmod | grep -q perfmgr_mtk; then
	ui_print "Conflicting kernel module"
	abort
fi

conf=/sdcard/Android/fas-rs/games.txt
if [ ! -f $conf ]; then
	mkdir -p /sdcard/Android/fas-rs
	cp $MODPATH/games.txt $conf
fi

rm $MODPATH/games.txt
