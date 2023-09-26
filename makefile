.DEFAULT_GOAL := package
RELEASE ?= true

.PHONY: clean
clean:
	rm -rf output/*
	cargo clean
	cd zygisk && \
    make clean

.PHONY: fas-rs
fas-rs:
	@echo "Building fas-rs(bin)…"
ifeq ($(RELEASE), true)
	cargo build --release --target aarch64-linux-android
else
	cargo build --target aarch64-linux-android
endif

.PHONY: zygisk
zygisk:
	cd zygisk && \
	make RELEASE=$(RELEASE)

.PHONY: package
package: fas-rs zygisk
	rm -rf output/.temp
	mkdir output/.temp
	cp -rf module/* output/.temp

ifeq ($(RELEASE), true)
	cp -f target/aarch64-linux-android/release/fas-rs output/.temp
else
	cp -f target/aarch64-linux-android/debug/fas-rs output/.temp
endif
	strip output/.temp/fas-rs

	mkdir -p output/.temp/zygisk
	mkdir -p output/.temp/system/lib64
	cp -f zygisk/build/arm64-v8a.so output/.temp/zygisk
	cp -f zygisk/build/libfasrs.so output/.temp/system/lib64

	cd output/.temp && \
	zip -9 -rq fas-rs.zip . && \
	mv fas-rs.zip ..
	
	@echo "Packaged at output/fas-rs.zip"
