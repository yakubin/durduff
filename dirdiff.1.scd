dirdiff(1)

# NAME

dirdiff - compare directories file by file

# SYNOPSIS

*dirdiff* [option]... <old> <new>

# OPTIONS

*-q, --brief*
	Report only when directories differ.

*-p, --progress*
	Show progress bar.

*--color* <when>
	Print output in color (<when> may be one of: never, always, auto).

*--percent*
	Utf-8 percent-encode paths using the path percent-encode set. If you want to
	parse dirdiff's output in a script, then you should use this option.

*-b, --block-size* <block-size>
	Read files in blocks of <block-size> bytes.

*-h, --help*
	Print help information and exit.

*--version*
	Print version information and exit.

# AUTHOR

Written by Jakub Alba <jakub@yakubin.com>.

# SEE ALSO

*diff*(1) *hashdeep*(1)
