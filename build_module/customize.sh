SKIPUNZIP=0
conf=/sdcard/Android/fas-rs/games.toml
old_conf=/sdcard/Android/fas-rs/games.txt

chmod a+x $MODPATH/fas-rs

if $MODPATH/fas-rs "test"; then
	ui_print "Supported"
else
    ui_print "Unsupported"
    abort
fi

if lsmod | grep -qE "perfmgr_mtk|ged_novsync"; then
	ui_print "Conflicting kernel module"
	abort
fi

if [ -f $old_conf ]; then
	# rename as .toml
	mv $old_conf $conf
fi

if [ -f $conf ]; then
	# merge local std
	$MODPATH/fas-rs merge $conf $MODPATH/games.toml
else
	mkdir -p /sdcard/Android/fas-rs
	cp $MODPATH/games.toml $conf
fi

rm $MODPATH/games.toml
