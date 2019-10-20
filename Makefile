build-debug:
	@cargo build

build-release:
	@cargo build --release

man:
	scdoc < durduff.1.scd | gzip --no-name --best > durduff.1.gz

debug-unit-tests:
	@cargo test

release-unit-tests:
	@cargo test --release

debug-func-tests: ./target/debug/durduff
	@shelltest --color --execdir "-D{exe}=../../target/debug/durduff" test-data

release-func-tests: ./target/release/durduff
	@shelltest --color --execdir "-D{exe}=../../target/release/durduff" test-data

deb: durduff.1.gz NEWS.gz target/release/durduff
	cargo deb

durduff.1.gz: man

NEWS.gz: NEWS
	gzip --no-name --best < NEWS > NEWS.gz

./target/debug/durduff: build-debug

./target/release/durduff: build-release
