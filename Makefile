EXE=dirdiff

build-debug:
	@cargo build

build-release:
	@cargo build --release

build-completions:
	mkdir -p completions/bash
	mkdir -p completions/fish
	mkdir -p completions/zsh
	sed -e "s/{exe}/${EXE}/g" completions.in/bash/dirdiff > completions/bash/${EXE}
	sed -e "s/{exe}/${EXE}/g" completions.in/fish/dirdiff.fish > completions/fish/${EXE}.fish
	sed -e "s/{exe}/${EXE}/g" completions.in/zsh/_dirdiff > completions/zsh/_${EXE}

build-man:
	sed -e "s/{exe}/${EXE}/g" dirdiff.1.scd | scdoc | gzip > ${EXE}.1.gz

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
