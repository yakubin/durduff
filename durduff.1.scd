durduff(1)

# NAME

durduff - compare directories file by file

# SYNOPSIS

*durduff* [option]... <old> <new>

# OPTIONS

*-q, --brief*
	Report only when directories differ.

*-0, --null*
	Print raw NUL-separated paths. In particular, do no percent-encode them.

*--progress* <when>
	Print progress reports (<when> may be one of: never, always, auto).

*--color* <when>
	Print output in color (<when> may be one of: never, always, auto).

*-b, --block-size* <block-size>
	Read files in blocks of <block-size> bytes.

*-h, --help*
	Print help information and exit.

*-V, --version*
	Print version information and exit.

# AUTHOR

Written by Jakub Alba <jakub@yakubin.com>.

# BUGS, PATCHES, SUPPORT

Report bugs to: <~yakubin/durduff@todo.sr.ht>, or via web:
<https://todo.sr.ht/~yakubin/durduff>.

Send patches to: <~yakubin/durduff-devel@lists.sr.ht>.

If you need help with durduff, send a mail to:
<~yakubin/durduff-user@lists.sr.ht>.

Please, remember about the mailing list ettiquette when using these mailing
lists: <https://man.sr.ht/lists.sr.ht/etiquette.md>.

# SEE ALSO

*diff*(1) *hashdeep*(1)
