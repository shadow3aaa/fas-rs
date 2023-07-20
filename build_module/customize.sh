SKIPUNZIP=0

chmod a+x $MODPATH/fas-rs
if $MODPATH/fas-rs "test"; then
	ui_print "Supported"
else
    ui_print "Unsupported"
fi

if lsmod | grep -qE "perfmgr_mtk|ged_novsync"; then
	ui_print "Conflicting kernel module"
	abort
fi

conf=/sdcard/Android/fas-rs/games.txt
if [ ! -f $conf ]; then
	mkdir -p /sdcard/Android/fas-rs
	cp $MODPATH/games.txt $conf
fi

rm $MODPATH/games.txt
