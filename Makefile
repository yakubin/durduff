last_tag := $(shell git for-each-ref refs/tags \
	--points-at=`git log --tags --no-walk --pretty="format:%H"` \
	--format='%(refname:short)')

semver := $(shell echo "${last_tag}" | sed -e "s/^v//" )
tildaver := $(shell echo "${semver}" | tr '-' '~')

fg_yellow := $(shell tput setaf 3)
fg_reset := $(shell tput sgr 0)

build-debug:
	@cargo build

build-release:
	@cargo build --release

man: target/assets/durduff.1.gz

debug-unit-tests:
	@cargo test

release-unit-tests:
	@cargo test --release

debug-func-tests: target/debug/durduff
	@shelltest --color --execdir "-D{exe}=../../target/debug/durduff" test-data

release-func-tests: target/release/durduff
	@shelltest --color --execdir "-D{exe}=../../target/release/durduff" test-data

deb: target/assets/durduff.1.gz target/assets/NEWS.gz target/release/durduff
	cargo deb

target/debug/durduff: build-debug

target/release/durduff: build-release

target/assets/NEWS.gz: target/assets
	gzip --no-name --best < NEWS > target/assets/NEWS.gz

target/assets/durduff.1.gz: target/assets
	scdoc < durduff.1.scd | gzip --no-name --best > target/assets/durduff.1.gz

target/assets:
	mkdir --parents target/assets

release_warnings: warn_if_tree_is_dirty warn_if_last_commit_is_not_tagged warn_if_cargo_and_git_disagree_what_the_current_version_is warn_if_changelog_is_outdated

warn_if_tree_is_dirty: warn_if_tree_has_untracked_files warn_if_tree_has_uncommitted_changes

warn_if_tree_has_untracked_files:
	@git ls-files \
		--exclude-standard \
		--others \
		--error-unmatch \
		. \
		>/dev/null \
		2>&1 \
		&& echo "${fg_yellow}warning: tree has untracked files${fg_reset}" \
		|| true

warn_if_tree_has_uncommitted_changes:
	@git diff-index --quiet --cached HEAD -- && git diff-files --quiet \
		|| echo "${fg_yellow}warning: tree has uncommitted changes${fg_reset}"

warn_if_last_commit_is_not_tagged:
	@[ -n "`git for-each-ref refs/tags --points-at=HEAD`" ] \
		|| echo "${fg_yellow}warning: the last commit is not tagged${fg_reset}"

warn_if_cargo_and_git_disagree_what_the_current_version_is:
	@[ `cargo metadata --no-deps --format-version 1 \
		| jq '.packages[0].version' \
		| tr -d '"'` \
		= \
		"${semver}" ] \
		|| echo "${fg_yellow}warning: cargo and git disagree what the current version is${fg_reset}"

warn_if_changelog_is_outdated:
	@[ `awk 'NR==1{print $$2}' NEWS | sed -e 's/,$$//'` \
		= \
		"${semver}" ] \
		|| echo "${fg_yellow}warning: changelog is out of date${fg_reset}"
