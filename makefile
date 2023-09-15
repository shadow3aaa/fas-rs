.DEFAULT_GOAL := package
RELEASE ?= true

.PHONY: clean
clean:
	rm -rf output/temp/*
	cargo clean
	cd surfaceflinger_hook && \
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
	rm -rf output/*
	mkdir -p output/temp
	cp -rf module/* output/temp

ifeq ($(RELEASE), true)
	cp -f target/aarch64-linux-android/release/fas-rs output/temp
else
	cp -f target/aarch64-linux-android/debug/fas-rs output/temp
endif

	mkdir output/temp/zygisk
	cp -f zygisk/build/arm64-v8a.so output/temp/zygisk

	cd output/temp && \
	zip -9 -rq fas-rs.zip . && \
	mv fas-rs.zip ..
	
	@echo "Packaged at output/fas-rs.zip"
