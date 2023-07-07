SKIPUNZIP=0

chmod a+x $MODPATH/fas-rs
if [ ! $($MODPATH/fas-rs "test") == "Supported" ]; then
	abort
fi

conf=/sdcard/Android/fas-rs/games.txt
if [ ! -f $conf ]; then
	mkdir -p /sdcard/Android/fas-rs
	cp $MODPATH/games.txt $conf
fi

rm $MODPATH/games.txt

killall fas-rs
nohup $MODPATH/fas-rs >/dev/null 2>&1 &