build-debug:
	@cargo build

build-release:
	@cargo build --release

man: target/assets/durduff.1.gz

test-debug:
	@cargo test

test-release:
	@cargo test --release

target/debug/durduff: build-debug

target/release/durduff: build-release

target/assets/NEWS.gz: target/assets
	@gzip --no-name --best < NEWS > target/assets/NEWS.gz

target/assets/durduff.1.gz: durduff.1.scd target/assets
	@scdoc < durduff.1.scd | gzip --no-name --best > target/assets/durduff.1.gz

target/assets:
	@mkdir --parents target/assets

clean:
	@rm --recursive --force target

ifeq ($(shell [ -d .git ] && echo git),git)
include with-git.mk
endif
