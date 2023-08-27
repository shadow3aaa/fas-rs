.DEFAULT_GOAL := package

.PHONY: clean
clean:
	@rm -rf output/*
	@cargo clean
	@cd surfaceflinger_hook && \
    make clean

.PHONY: fas-rs
fas-rs:
	@echo "Building fas-rs(bin)…"
	@cargo b -r --target aarch64-linux-android

.PHONY: hook
hook:
	@cd surfaceflinger_hook && \
	make

.PHONY: package
package: fas-rs hook
	@rm -rf output/*
	@mkdir -p output

	@cp -rf module/* output/
	@cp -f target/aarch64-linux-android/release/fas-rs output/
	@cp -rf surfaceflinger_hook/output output/hook

	@cd output && \
	zip -9 -rq fas-rs.zip .
	@echo "Packaged at output/fas-rs.zip"