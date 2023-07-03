#/usr/bin/bash

here=${0%/*}
bin="${here}/target/release/fas-rs"
if [ ! -f $bin ]; then
    bin="${here}/target/aarch64-linux-android/release/fas-rs"
fi

if [ ! -f $bin ]; then
    echo "缺少fas-rs release可执行文件，先编译?"
    exit 1
fi

cp -f $(realpath $bin) "${here}/build_module/"
cd "${here}/build_module/"

zip -9 -r ../fas-rs.zip .
echo -n "Packaging is complete: $(realpath ${here}/fas-rs.zip)"
