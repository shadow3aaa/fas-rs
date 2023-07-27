SKIPUNZIP=0
conf=/sdcard/Android/fas-rs/games.toml
old_conf=/sdcard/Android/fas-rs/games.txt

# permission
chmod a+x $MODPATH/fas-rs

# test support
if $MODPATH/fas-rs "test"; then
	ui_print "Supported"
else
	ui_print "Unsupported"
	abort
fi

# detect conflicting kernel modules
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
	# creat new config
	mkdir -p /sdcard/Android/fas-rs
	cp $MODPATH/games.toml $conf
fi

# remove std config
rm $MODPATH/games.toml
