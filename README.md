# Super Speedy Syslog Searcher! (s4)<!-- omit in TOC -->

Speedily search and sort many syslog files by datetime.

_Super Speedy Syslog Searcher_ (s4) is a command-line tool to search
and sort syslog files within compressed files (`.gz`, `.xz`) and
archives (`.tar`, `.zip`).
The first goal of s4 is speedy searching and printing.

[![Build status](https://github.com/jtmoon79/super-speedy-syslog-searcher/actions/workflows/rust.yml/badge.svg)](https://github.com/jtmoon79/super-speedy-syslog-searcher/actions?query=workflow%3Arust)
[![docs.rs](https://docs.rs/super-speedy-syslog-searcher/badge.svg)](https://docs.rs/super-speedy-syslog-searcher)
[![codecov.io](https://codecov.io/gh/jtmoon79/super-speedy-syslog-searcher/branch/main/graph/badge.svg?token=Q2OXTL7U02)](https://codecov.io/gh/jtmoon79/super-speedy-syslog-searcher)
[![crates.io](https://img.shields.io/crates/v/super-speedy-syslog-searcher.svg)](https://crates.io/crates/super-speedy-syslog-searcher)

---

<!-- TOC generated by Markdown All In One -->

- [Use](#use)
  - [Install `super_speedy_syslog_searcher`](#install-super_speedy_syslog_searcher)
  - [Run `s4`](#run-s4)
  - [`--help`](#--help)
- [About](#about)
  - [Features](#features)
  - [Limitations](#limitations)
  - [Hacks](#hacks)
- [Further Reading](#further-reading)

---

## Use

### Install `super_speedy_syslog_searcher`

```lang-text
cargo install super_speedy_syslog_searcher
```

### Run `s4`

For example, print all the syslog lines in syslog files under `/var/log/`

```lang-text
s4 /var/log
```

Print only the syslog lines since yesterday

```lang-text
s4 /var/log -a $(date -d "yesterday" '+%Y-%m-%d')
```

Print only the syslog lines that occurred two days ago

```lang-text
s4 /var/log -a $(date -d "2 days ago" '+%Y-%m-%d') -b $(date -d "1 days ago" '+%Y-%m-%d')
```

Print only the syslog lines that occurred two days ago during the noon hour

```lang-text
s4 /var/log -a $(date -d "2 days ago 12:00" '+%Y-%m-%dT%H:%M:%S') -b $(date -d "2 days ago 13:00" '+%Y-%m-%dT%H:%M:%S')
```

Print only the syslog lines that occurred two days ago during the noon hour in
Bengaluru, India (timezone offset +05:30) and prepended with equivalent UTC
datetime.

```lang-text
s4 /var/log -u -a "$(date -d "2 days ago 12:00" '+%Y-%m-%dT%H:%M:%S') +05:30" -b "$(date -d "2 days ago 13:00" '+%Y-%m-%dT%H:%M:%S') +05:30"
```

### `--help`

```lang-text
Super Speedy Syslog Searcher will search syslog files and sort entries by datetime. DateTime filters
may be passed to narrow the search. It aims to be very fast.

USAGE:
    s4 [OPTIONS] <PATHS>...

ARGS:
    <PATHS>...    Path(s) of syslog files or directories. Directories will be recursed,
                  remaining on the same filesystem. Symlinks will be followed

OPTIONS:
    -a, --dt-after <DT_AFTER>      DateTime After filter - print syslog lines with a datetime that
                                   is at or after this datetime. For example, '20200102T123000'
    -b, --dt-before <DT_BEFORE>    DateTime Before filter - print syslog lines with a datetime that
                                   is at or before this datetime. For example, '20200102T123001'
    -t, --tz-offset <TZ_OFFSET>    DateTime Timezone offset - for syslines with a datetime that does
                                   not include a timezone, this will be used. For example, '-0800'
                                   '+02:00' (with or without ':'). If passing a value with leading
                                   '-', use the '=' to explicitly set the argument, e.g. '-t=-0800'.
                                   Otherwise the CLI argument parsing will fail. Default is local
                                   system timezone offset. [default: -08:00]
    -u, --prepend-utc              Prepend DateTime in the UTC Timezone for every line
    -l, --prepend-local            Prepend DateTime in the Local Timezone for every line
    -n, --prepend-filename         Prepend file basename to every line
    -p, --prepend-filepath         Prepend file full path to every line
    -w, --prepend-file-align       Align column widths of prepended data
    -c, --color <COLOR_CHOICE>     Choose to print to terminal using colors [default: auto]
                                   [possible values: always, auto, never]
    -z, --blocksz <BLOCKSZ>        Read blocks of this size. May pass decimal or hexadecimal
                                   numbers. Using the default value is recommended [default: 65535]
    -s, --summary                  Print a summary of files processed. Printed to stderr
    -h, --help                     Print help information
    -V, --version                  Print version information


DateTime Filter patterns may be:
    "%Y%m%dT%H%M%S"
    "%Y%m%dT%H%M%S%z"
    "%Y-%m-%d %H:%M:%S"
    "%Y-%m-%d %H:%M:%S %z"
    "%Y-%m-%dT%H:%M:%S"
    "%Y-%m-%dT%H:%M:%S %z"
    "%Y/%m/%d %H:%M:%S"
    "%Y/%m/%d %H:%M:%S %z"
    "%Y%m%d"
    "%Y%m%d %z"
    "+%s"

Without a timezone offset (%z or %Z), the Datetime Filter is presumed to be the
system timezone.
Pattern "+%s" is Unix epoch timestamp in seconds with a preceding "+".
Ambiguous timezones will be rejected, e.g. "SST".
Prepended datetime, -u or -l, is printed in format "%Y%m%dT%H%M%S%.6f %z:".
DateTime formatting is described at https://docs.rs/chrono/latest/chrono/format/strftime/

DateTimes supported are only of the Gregorian calendar.
DateTimes supported language is English.
```

## About

### Features

- Prepends datetime and file paths, for easy programmatic parsing or visual
  traversal of varying syslog messages
- Supports many varying datetime formats including (WHAT ARE THOSE OFFICIAL ONES?)
- Tested against "in the wild" log files from varying Linux distributions
  (see `./logs/`)
- Comparable speed as GNU `grep` and `sort`
  (see `./tools/compare-grep-sort.sh`)
- Handles invalid UTF-8 (prints whatever is found)

### Limitations

- Only handles UTF-8 or ASCII encoded log files.
- Cannot handle multi-file `.gz` files (multiple "streams")
  (TODO describe problem)
- Cannot handle multi-file `.xz` files (chooses first file found)
  (TODO describe problem)
- Cannot process archive or compressed files within other archive or compressed
  files.
  e.g. a `.tar` file within another `.tar` file will not be processed, a `.gz`
  file within a `.tar` file will not be processed.

### Hacks

- Entire `.xz` files are read into memory during the initial `open` (see [607a23c00aff0d9b34fb3d678bdfd5c14290582d](https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/607a23c00aff0d9b34fb3d678bdfd5c14290582d#diff-a23d01b527ccc36fa0336ab1789a2f5d2567f21e93c5708b0e5b46ae9f3a708cR783-R836))

## Further Reading

- [`Extended-Thoughts.md`](./Extended-Thoughts.md)
- [`CHANGELOG.md`](./CHANGELOG.md)

---

<a href="https://stackexchange.com/users/216253/jamesthomasmoon1979"><img src="https://stackexchange.com/users/flair/216253.png" width="208" height="58" alt="profile for JamesThomasMoon1979 on Stack Exchange, a network of free, community-driven Q&amp;A sites" title="profile for JamesThomasMoon1979 on Stack Exchange, a network of free, community-driven Q&amp;A sites" /></a>
