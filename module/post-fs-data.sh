# $1:value $2:path
lock_val() {
    umount $2
    chmod +w $2

    echo "$1" | tee /dev/fas_rs_mask $2
    /bin/find $2 -exec mount /dev/fas_rs_mask {} \;
    rm /dev/fas_rs_mask
}

lock_val "" "/odm/bin/hw/vendor.oplus.hardware.ormsHalService-aidl-service"
