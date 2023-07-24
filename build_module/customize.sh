SKIPUNZIP=0
conf=/sdcard/Android/fas-rs/games.txt

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

if [ -f $conf ]; then
    # merge local std
	$MODPATH/fas-rs merge $conf $MODPATH/games.txt
else
	mkdir -p /sdcard/Android/fas-rs
	cp $MODPATH/games.txt $conf
fi

rm $MODPATH/games.txt
