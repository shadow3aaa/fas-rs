#/usr/bin/bash

here=${0%/*}

cd "${here}/build_module/"
if [ ! -f "$(pwd)/fas-rs" ]; then
	echo "缺少fas-rs可执行文件，先编译?"
	exit 1
fi

zip -9 -r ../fas-rs.zip .
echo -n "打包完成: $(realpath $(pwd)/../fas-rs.zip)"
