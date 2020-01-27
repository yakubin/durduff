#!/usr/bin/env sh

if [ "$#" -ne 1 ]; then
    echo "make-prebuilt-tarball.sh: exactly one argument should be provided" >&2
    exit 1
fi

tildaver="$1"

target_triple=`./target/release/durduff --version | awk '/target triple/{print $3}'`
target_dir="target/prebuilt-tarball/durduff-$tildaver-$target_triple"
target_tarball="$target_dir.tar.xz"

rm --recursive --force "$target_dir"
rm --force "$target_tarball"

mkdir --parents "$target_dir"

install -D --target-directory="$target_dir/bin/" -m 755 target/release/durduff
install -D --target-directory="$target_dir/share/bash-completion/completions/" -m 644 completions/bash/durduff
install -D --target-directory="$target_dir/share/zsh/vendor-completions/" -m 644 completions/zsh/_durduff
install -D --target-directory="$target_dir/share/fish/vendor_completions.d/" -m 644 completions/fish/durduff.fish
install -D --target-directory="$target_dir/share/man/man1/" -m 644 target/assets/durduff.1.gz
install -D --target-directory="$target_dir/share/doc/durduff/changelog.gz" -m 644 target/assets/NEWS.gz

tar \
    --directory="target/prebuilt-tarball" \
    --create \
    --xz \
    --file "$target_tarball" \
    "durduff-$tildaver-$target_triple"

rm --recursive --force "${target_dir}"

echo "created a new prebuilt tarball: $target_tarball"
