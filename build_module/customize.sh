SKIPUNZIP=0

chmod a+x $MODPATH/fas-rs
if [ ! $($MODPATH/fas-rs test) == "Supported" ]; then
	abort
fi
