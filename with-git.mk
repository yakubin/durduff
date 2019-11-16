last_tag := $(shell git for-each-ref refs/tags \
	--points-at=`git log --tags --no-walk --pretty="format:%H"` \
	--format='%(refname:short)')

semver := $(shell echo "${last_tag}" | sed -e "s/^v//" )
tildaver := $(shell echo "${semver}" | tr '-' '~')

fg_yellow := $(shell tput setaf 3)
fg_reset := $(shell tput sgr 0)

deb: target/assets/durduff.1.gz target/assets/NEWS.gz target/release/durduff release_warnings
	@cargo deb --no-build

source-tarball: target/source-tarball release_warnings
	@rm --recursive --force "target/source-tarball/durduff-${tildaver}"
	@rm --force "target/source-tarball/durduff-${tildaver}.tar.xz"
	@mkdir "target/source-tarball/durduff-${tildaver}"

	@find . \
		-mindepth 1 -maxdepth 1 \
		-not -name '.git*' \
		-not -exec git check-ignore --quiet '{}' ';' \
		-exec cp \
			--recursive \
			"--target-directory=target/source-tarball/durduff-${tildaver}" \
			'{}' '+'

	@tar \
		--directory=target/source-tarball \
		--create \
		--xz \
		--file "target/source-tarball/durduff-${tildaver}.tar.xz" \
		"durduff-${tildaver}"

	@echo "created a new source tarball: target/source-tarball/durduff-${tildaver}.tar.xz"

target/source-tarball:
	@mkdir --parents target/source-tarball

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
