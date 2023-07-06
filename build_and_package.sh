#/usr/bin/bash

here=${0%/*}
bin="${here}/target/aarch64-linux-android/release/fas-rs"

cargo b -r --target aarch64-linux-android

if [ ! -f $bin ]; then
    echo "Fail to build release"
    exit 1
fi

echo -e "Build successed"

cp -f $(realpath $bin) "${here}/build_module/"
cd "${here}/build_module/"
zip -9 -rq ../fas-rs.zip .

echo -n "Packaging is complete: $(realpath ${here}/fas-rs.zip)"
