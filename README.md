# dirdiff

dirdiff lets you compare directories file by file.

## Installation

Packages for Linux distros, BSDs, Mac and Windows are a TODO. In the mean time
you can install it using
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) from
[crates.io](https://crates.io):

    $ cargo install dirdiff

## Building

### The executable

You will need make and
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to do
that.

To build the debug version, run:

    $ make build-debug

The resulting binary will be located at `target/debug/dirdiff`.

To build the release version, run:

    $ make build-release

The resulting binary will be located at `target/release/dirdiff`.

### The man page

You will need make, gzip and [scdoc](https://git.sr.ht/~sircmpwn/scdoc) to do
that.

    $ make build-man

The resulting gzipped man page will be located at `dirdiff.1.gz`.

## Running tests

You will need make,
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and
[shelltestrunner](https://github.com/simonmichael/shelltestrunner) to do that.

Unit tests of the debug version:

    $ make debug-unit-tests

Unit tests of the release version:

    $ make release-unit-tests

Functional tests of the debug version:

    $ make debug-func-tests

Functional tests of the release version:

    $ make release-func-tests

## Options

    dirdiff(1)                  General Commands Manual                 dirdiff(1)

    NAME
           dirdiff - compare directories file by file

    SYNOPSIS
           dirdiff [option]... <old> <new>

    OPTIONS
           -q, --brief
               Report only when directories differ.

           -p, --progress
               Show progress bar.

           --color <when>
               Print output in color (<when> may be one of: never, always, auto).

           --percent
               Utf-8 percent-encode paths using the path percent-encode set. If
               you want to parse dirdiff's output in a script, then you should use
               this option.

           -b, --block-size <block-size>
               Read files in blocks of <block-size> bytes.

           -h, --help
               Print help information and exit.

           --version
               Print version information and exit.

    AUTHOR
           Written by Jakub Alba <jakub@yakubin.com>.

    SEE ALSO
           diff(1) hashdeep(1)

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
    $ dirdiff old new
    + b
    - bar
    ~ c
    ~ foo/a

## Contributing

Send patches/bug reports to <jakub@yakubin.com>.
