BASEDIR=$(realpath $0)
HOOK_DIR=/dev/surfaceflinger_hook
SO=$HOOK_DIR/libsurfaceflinger_hook.so

# wait for surfaceflinger start
until pidof surfaceflinger; do
	sleep 1s
done

set_perm() {
	chown $2:$3 $1 || return 1
	chmod $4 $1 || return 1
	local CON=$5
	[ -z $CON ] && CON=u:object_r:system_file:s0
	chcon $CON $1 || return 1
}

set_perm_recursive() {
	find $1 -type d 2>/dev/null | while read dir; do
		set_perm $dir $2 $3 $4 $6
	done
	find $1 -type f -o -type l 2>/dev/null | while read file; do
		set_perm $file $2 $3 $5 $6
	done
}

set_dir() {
	mkdir -p $HOOK_DIR
	cp -f $BASEDIR/libsurfaceflinger_hook.so $SO
}

set_permissions() {
	magiskpolicy --live "allow surfaceflinger * * *"
	set_perm_recursive $HOOK_DIR graphics system 0777 0644
}

inject() {
	local pid=$(pidof surfaceflinger)

	# reserve time for something unexpected
	sleep 60s

	$BASEDIR/inject -p $pid -so $SO -symbols handle_hook
}

set_dir
set_permissions
inject
