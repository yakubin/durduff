build-debug:
	@cargo build

build-release:
	@cargo build --release

man:
	scdoc < durduff.1.scd | gzip > durduff.1.gz

debug-unit-tests:
	@cargo test

release-unit-tests:
	@cargo test --release

debug-func-tests: ./target/debug/durduff
	@shelltest --color --execdir "-D{exe}=../../target/debug/durduff" test-data

release-func-tests: ./target/release/durduff
	@shelltest --color --execdir "-D{exe}=../../target/release/durduff" test-data

deb: durduff.1.gz target/release/durduff
	cargo deb

durduff.1.gz: man

./target/debug/durduff: build-debug

./target/release/durduff: build-release
