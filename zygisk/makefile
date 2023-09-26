.DEFAULT_GOAL := all
RELEASE ?= true
CXX_RELEASE_FLAG := -O3 -flto -std=c++2b -fno-exceptions -fno-rtti -fvisibility=hidden -fvisibility-inlines-hidden -Wl,-O3,--lto-O3 -Wall
CXX_DEBUG_FLAG := -Og -std=c++2b -fno-exceptions -fno-rtti -fvisibility=hidden -fvisibility-inlines-hidden

.PHONY: all
all: zygisk-lib rust-lib

.PHONY: clean
clean:
	rm -rf build/*
	cd rust && cargo clean

.PHONY: init-build
init-build:
	mkdir -p build

.PHONY: zygisk-lib
zygisk-lib: init-build
ifeq ($(RELEASE), true)
	aarch64-linux-android-clang++ $(CXX_RELEASE_FLAG) --shared src/hook.cpp \
		-Lbuild/rust \
		-Irust/include \
		-nostdlib++ \
		-Wl,-llog,-lc++ \
		-o build/arm64-v8a.so
	strip build/arm64-v8a.so
else
	aarch64-linux-android-clang++ $(CXX_DEBUG_FLAG) --shared src/hook.cpp \
		-Lbuild/rust \
		-Irust/include \
		-nostdlib++ \
		-Wl,-llog,-lc++ \
		-o build/arm64-v8a.so
endif

.PHONY: rust-lib
rust-lib: init-build
ifeq ($(RELEASE), true)
	cd rust && cargo build --release --target aarch64-linux-android
	cp -f rust/target/aarch64-linux-android/release/libfasrs.so build
else
	cd rust && cargo build --target aarch64-linux-android
	cp -f rust/target/aarch64-linux-android/debug/libfasrs.so build
endif