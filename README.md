# extcnt

Show files grouped by extension.

MSRV: 1.74.1

## Usage

```
extcnt [OPTIONS]

Options:
  -d, --directory <DIRECTORY>  Directory to scan [default: .]
  -a, --all                    Include hidden files (names starting with '.')
  -i, --ignore-case            Group extensions case-insensitively (e.g. TXT and txt together)
  -c, --count-only             Print only the number of files per extension
  -h, --help                   Print help
  -V, --version                Print version
```

Only regular files in the given directory are listed; subdirectories are not
descended into. Symlinks are followed, so links to files are listed and links to
directories are skipped. Files without an extension are grouped under `(none)`.
Names that are not valid UTF-8 are shown with `�` replacement characters.

## Example

```
$ extcnt -d project
(none) (1):
- Makefile

rs (2):
- lib.rs
- main.rs

txt (1):
- notes.txt

$ extcnt -d project --count-only
(none): 1
rs: 2
txt: 1
```

The exit status is non-zero if the directory cannot be read.
