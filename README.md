# durduff

durduff lets you compare directories file by file.

## Installation

Packages for Linux distros and BSDs are a TODO. In the mean time you can install
it using
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) from
[crates.io](https://crates.io):

    $ cargo install durduff

## Building

### The executable

You will need make and
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to do
that.

To build the debug version, run:

    $ make build-debug

The resulting binary will be located at `target/debug/durduff`.

To build the release version, run:

    $ make build-release

The resulting binary will be located at `target/release/durduff`.

### The man page

You will need make, gzip and [scdoc](https://git.sr.ht/~sircmpwn/scdoc) to do
that.

    $ make man

The resulting gzipped man page will be located at `target/assets/durduff.1.gz`.

## Running tests

You will need make and
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Run tests of the debug version:

    $ make test-debug

Run tests of the release version:

    $ make test-release

## Options

    SYNOPSIS
           durduff [option]... <old> <new>

    OPTIONS
           -q, --brief
               Report only when directories differ.

           -0, --null
               Print raw NUL-separated paths. In particular, do no percent-encode
               them.

           --progress <when>
               Print progress reports (<when> may be one of: never, always, auto).

           --color <when>
               Print output in color (<when> may be one of: never, always, auto).

           -b, --block-size <block-size>
               Read files in blocks of <block-size> bytes.

           -h, --help
               Print help information and exit.

           -V, --version
               Print version information and exit.

## Example usage

    $ tree old new
    old
    ├── bar
    ├── c
    ├── d
    └── foo
        ├── a
        └── baz
    new
    ├── b
    ├── c
    ├── d
    └── foo
        ├── a
        └── baz

    5 directories, 7 files
    $ diff -q {old,new}/foo/a
    Files old/foo/a and new/foo/a differ
    $ diff -q {old,new}/c
    Files old/c and new/c differ
    $ diff -q {old,new}/d
    $ durduff old new
    + b
    - bar
    ~ c
    ~ foo/a

## Bugs, patches, support

Report bugs to: <~yakubin/durduff@todo.sr.ht>, or via web:
<https://todo.sr.ht/~yakubin/durduff>.

Send patches to: <~yakubin/durduff-devel@lists.sr.ht>.

If you need help with durduff, send a mail to:
<~yakubin/durduff-user@lists.sr.ht>.

Please, remember about the [mailing list
ettiquette](https://man.sr.ht/lists.sr.ht/etiquette.md) when using these mailing
lists.
