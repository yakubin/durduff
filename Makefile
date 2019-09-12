build-debug:
	@cargo build

build-release:
	@cargo build --release

build-man:
	scdoc < dirdiff.1.scd | gzip > dirdiff.1.gz

debug-unit-tests:
	@cargo test

release-unit-tests:
	@cargo test --release

debug-func-tests: ./target/debug/dirdiff
	@shelltest --color --execdir "-D{exe}=../../target/debug/dirdiff" test-data

release-func-tests: ./target/release/dirdiff
	@shelltest --color --execdir "-D{exe}=../../target/release/dirdiff" test-data

./target/debug/dirdiff: build-debug

./target/release/dirdiff: build-release
