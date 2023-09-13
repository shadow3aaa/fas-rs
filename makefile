.DEFAULT_GOAL := package
RELEASE ?= false

.PHONY: clean
clean:
	rm -rf output/*
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
hook:
	cd zygisk && \
	make RELEASE=$(RELEASE)

.PHONY: package
package: fas-rs hook
	rm -rf output/*
	mkdir -p output
	cp -rf module/* output/

ifeq ($(RELEASE), true)
	cp -f target/aarch64-linux-android/release/fas-rs output/
else
	cp -f target/aarch64-linux-android/debug/fas-rs output/
endif

	mkdir output/zygisk
	cp -f libgui-analyze/build/arm64-v8a.so output/zygisk

	cd output && \
	zip -9 -rq fas-rs.zip .
	@echo "Packaged at output/fas-rs.zip"
