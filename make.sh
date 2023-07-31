#/usr/bin/bash

here=$(pwd)
bin=${here}/target/aarch64-linux-android/release/fas-rs

set -e

cargo b -r --target aarch64-linux-android

if [ ! -f $bin ]; then
	echo "Fail to build release"
	ls "${here}"
	exit 1
fi

echo -e "Build successed"
cp -f $(realpath $bin) "${here}/build_module/"

cd "${here}/build_module/"
zip -9 -rq ../fas-rs.zip .

echo -n "Packaging is complete: $(realpath ${here}/fas-rs.zip)"
