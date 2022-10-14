# Super Speedy Syslog Searcher! (s4)<!-- omit in TOC -->

Speedily search and merge many syslog files by datetime.

_Super Speedy Syslog Searcher_ (s4) is a command-line tool to search
and merge plain log files, including log compressed log files (`.gz`, `.xz`) and
within archives (`.tar`).
The first goal of s4 is speedy searching and printing.

[![Build status](https://img.shields.io/github/workflow/status/jtmoon79/super-speedy-syslog-searcher/Rust?style=flat-square&logo=github)](https://github.com/jtmoon79/super-speedy-syslog-searcher/actions?query=workflow%3Arust)
[![docs.rs](https://img.shields.io/docsrs/super_speedy_syslog_searcher/latest?badge.svg&style=flat-square&logo=docsdotrs)](https://docs.rs/super_speedy_syslog_searcher/latest/)
[![License](https://img.shields.io/crates/l/super-speedy-syslog-searcher?style=flat-square)](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/main/LICENSE.txt)

[![crates.io version](https://img.shields.io/crates/v/super-speedy-syslog-searcher.svg?style=flat-square&logo=rust)](https://crates.io/crates/super-speedy-syslog-searcher)
[![crates.io downloads](https://img.shields.io/crates/d/super-speedy-syslog-searcher.svg?style=flat-square&logo=rust)](https://crates.io/crates/super-speedy-syslog-searcher)
[![codecov.io](https://img.shields.io/codecov/c/github/jtmoon79/super-speedy-syslog-searcher/branch?main&token=Q2OXTL7U02&style=flat-square&logo=codecov)](https://codecov.io/gh/jtmoon79/super-speedy-syslog-searcher)
[![Commits since](https://img.shields.io/github/commits-since/jtmoon79/super-speedy-syslog-searcher/latest.svg)](https://github.com/jtmoon79/super-speedy-syslog-searcher/commits/main)
[![Requirements Status](https://requires.io/github/jtmoon79/super-speedy-syslog-searcher/requirements.svg?branch=main)](https://requires.io/github/jtmoon79/super-speedy-syslog-searcher/requirements/?branch=main)

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
- ["syslog" definition](#syslog-definition)
- [logging chaos](#logging-chaos)
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

On Windows under `C:\Windows\Logs`

```lang-text
s4.exe C:\Windows\Logs
```

Print the syslog lines after January 1, 2022 at 00:00

```lang-text
s4 /var/log -a 20220101
```

Print the syslog lines from January 1, 2022 00:00 to January 2, 2022

```lang-text
s4 /var/log -a 20220101 -b 20220102
```

Print the syslog lines on January 1, 2022, from 12:00 to 16:00

```lang-text
s4 /var/log -a 20220101T120000 -b 20220101T160000
```

Print only the syslog lines since yesterday (with the help of GNU `date`)

```lang-text
s4 /var/log -a $(date -d "yesterday" '+%Y-%m-%d')
```

Print only the syslog lines that occurred two days ago
(with the help of GNU `date`)

```lang-text
s4 /var/log -a $(date -d "2 days ago" '+%Y-%m-%d') -b $(date -d "1 days ago" '+%Y-%m-%d')
```

Print only the syslog lines that occurred two days ago during the noon hour
(with the help of GNU `date`)

```lang-text
s4 /var/log -a $(date -d "2 days ago 12:00" '+%Y-%m-%dT%H:%M:%S') -b $(date -d "2 days ago 13:00" '+%Y-%m-%dT%H:%M:%S')
```

Print only the syslog lines that occurred two days ago during the noon hour in
Bengaluru, India (timezone offset +05:30) and prepended with equivalent UTC
datetime (with the help of GNU `date`)

```lang-text
s4 /var/log -u -a "$(date -d "2 days ago 12:00" '+%Y-%m-%dT%H:%M:%S') +05:30" -b "$(date -d "2 days ago 13:00" '+%Y-%m-%dT%H:%M:%S') +05:30"
```

### `--help`

```lang-text
Super Speedy Syslog Searcher will search syslog files and sort entries by datetime.
DateTime filters may be passed to narrow the search. It aims to be very fast.

USAGE:
    s4 [OPTIONS] <PATHS>...

ARGS:
    <PATHS>...    Path(s) of syslog files or directories. Directories will be recursed,
                  remaining on the same filesystem. Symlinks will be followed

OPTIONS:
    -a, --dt-after <DT_AFTER>
            DateTime After filter - print syslog lines with a datetime that is at or after this
            datetime. For example, "20200102T123000"

    -b, --dt-before <DT_BEFORE>
            DateTime Before filter - print syslog lines with a datetime that is at or before this
            datetime. For example, "20200102T123001"

    -t, --tz-offset <TZ_OFFSET>
            DateTime Timezone offset - for syslines with a datetime that does not include a
            timezone, this will be used. For example, "-0800", "+02:00", or "EDT". Ambiguous named
            timezones parsed from logs will use this value, e.g. timezone "IST". (to pass a value
            with leading "-", use ", e.g. "-t=-0800"). Default is local system timezone offset.
            [default: -08:00]

    -u, --prepend-utc
            Prepend DateTime in the UTC Timezone for every line

    -l, --prepend-local
            Prepend DateTime in the Local Timezone for every line

    -d, --prepend-dt-format <PREPEND_DT_FORMAT>
            Prepend DateTime using strftime format string [default: %Y%m%dT%H%M%S%.3f%z:]

    -n, --prepend-filename
            Prepend file basename to every line

    -p, --prepend-filepath
            Prepend file full path to every line

    -w, --prepend-file-align
            Align column widths of prepended data

    -c, --color <COLOR_CHOICE>
            Choose to print to terminal using colors [default: auto] [possible values: always, auto,
            never]

    -z, --blocksz <BLOCKSZ>
            Read blocks of this size in bytes. May pass decimal or hexadecimal numbers. Using the
            default value is recommended. Most useful for developers [default: 65535]

    -s, --summary
            Print a summary of files processed to stderr. Most useful for developers

    -h, --help
            Print help information

    -V, --version
            Print version information


DateTime Filter strftime specifier patterns may be:
    "%Y%m%dT%H%M%S"
    "%Y%m%dT%H%M%S%z"
    "%Y%m%dT%H%M%S%:z"
    "%Y%m%dT%H%M%S%#z"
    "%Y%m%dT%H%M%S%Z"
    "%Y-%m-%d %H:%M:%S"
    "%Y-%m-%d %H:%M:%S %z"
    "%Y-%m-%d %H:%M:%S %:z"
    "%Y-%m-%d %H:%M:%S %#z"
    "%Y-%m-%d %H:%M:%S %Z"
    "%Y-%m-%dT%H:%M:%S"
    "%Y-%m-%dT%H:%M:%S %z"
    "%Y-%m-%dT%H:%M:%S %:z"
    "%Y-%m-%dT%H:%M:%S %#z"
    "%Y-%m-%dT%H:%M:%S %Z"
    "%Y/%m/%d %H:%M:%S"
    "%Y/%m/%d %H:%M:%S %z"
    "%Y/%m/%d %H:%M:%S %:z"
    "%Y/%m/%d %H:%M:%S %#z"
    "%Y/%m/%d %H:%M:%S %Z"
    "%Y%m%d"
    "%Y-%m-%d"
    "%Y/%m/%d"
    "%Y%m%d %z"
    "%Y%m%d %:z"
    "%Y%m%d %#z"
    "%Y%m%d %Z"
    "+%s"

Pattern "+%s" is Unix epoch timestamp in seconds with a preceding "+".
Without a timezone offset ("%z" or "%Z"), the Datetime Filter is presumed to be the local system
timezone.
Ambiguous user-passed named timezones will be rejected, e.g. "SST".

DateTime strftime specifier patterns are described at
https://docs.rs/chrono/latest/chrono/format/strftime/

DateTimes supported are only of the Gregorian calendar.
DateTimes supported language is English.
```

## About

_Super Speedy Syslog Searcher_ (s4) is meant to aid Engineers in reviewing
varying log files in a datetime-sorted manner.
The primary use-case is to aid investigating problems wherein the time of
problem occurrence is known but otherwise there is little source evidence.

Currently, log file formats vary widely. _Most_ logs are an ad-hoc format.
Even separate log files on the same system for the same service may have
different message formats! 😵
Sorting these logged messages by datetime may be prohibitively difficult.
The result is an engineer may have to "hunt and peck" among many log files,
looking for problem clues around some datetime; so tedious!

Enter _Super Speedy Syslog Searcher_ 🦸 ‼

_s4_ will print log messages from multiple log files in datetime-sorted order.
A "window" of datetimes may be passed, to constrain the period of printed
messages. This will assist an engineer that, for example, needs to view all
syslog messages that occured two days ago among log files taken from multiple
systems.

The alterior motive for _Super Speedy Syslog Searcher_ was the [primary
developer](https://github.com/jtmoon79) wanted an excuse to learn rust 🦀,
and wanted to create an open-source tool for a recurring need of some
Software Test Engineers 😄

A longer rambling pontification about this project is in
[`Extended-Thoughts.md`](./Extended-Thoughts.md).

### Features

- Prepends datetime and file paths, for easy programmatic parsing or visual traversal of varying syslog messages
- Recognizes multi-line log messages
- Parses formal datetime formats:
  - [RFC 2822](https://www.rfc-editor.org/rfc/rfc2822#section-3.3)
  - [RFC 3164](https://www.rfc-editor.org/rfc/rfc3164#section-4.1.2)
  - [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339#section-5.8)
  - [RFC 5424](https://www.rfc-editor.org/rfc/rfc5424#section-6.2.3)
  - [ISO 8601](https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1113067353#General_principles)**
- Parses many ad-hoc datetime formats
  - Tested against "in the wild" log files from varying Linux distributions
    (see project `./logs/`)
- Comparable speed as GNU `grep` and `sort`
  (see project tool `./tools/compare-grep-sort.sh`; run in github Actions, Job
  _run s4_, Step _Run script compare-grep-sort_)
- Processes invalid UTF-8

### Limitations

- Only processes UTF-8 or ASCII encoded log files. ([Issue #16](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/16))
- Cannot processes multi-file `.gz` files (only processes first stream found) ([Issue #8](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/8))
- Cannot processes multi-file `.xz` files (only processes first stream found) ([Issue #11](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/11))
- Cannot process archive files or compressed files within other archive files or compressed files, e.g. `logs.tgz`. ([Issue #14](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/14))
  e.g. file `syslog.xz` file within file `logs.tar` will not be processed,
- Cannot process `.zip` archives ([Issue #39](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/39))
- **ISO 8601
  - ISO 8601 forms recognized
  (using [ISO descriptive format](https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Calendar_dates))
    - `YYYY-MM-DDThh:mm:ss`
    - `YYYY-MM-DDThhmmss`
    - `YYYYMMDDThhmmss`
    (may use date-time separator character `'T'` or character blank space `' '`)
  - ISO 8601 forms not recognized:
    - Absent seconds
    - [_Ordinal dates_](https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Ordinal_dates), i.e. "day of the year", format `YYYY-DDD`, e.g. `"2022-321"`
    - [_Week dates_](https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Week_dates), i.e. "week-numbering year", format `YYYY-Www-D`, e.g. `"2022-W25-1"`
    - times [without minutes and seconds](https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Times) (i.e. only `hh`)

### Hacks

- Entire `.xz` files are read into memory during the initial `open` ([Issue #12](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/12))

<br/>

## "syslog" definition

In this project, the term "syslog" is used generously to refer any
log message that has a datetime stamp on the first line of log text.

Technically, "syslog" is [defined among several RFCs](https://en.wikipedia.org/w/index.php?title=Syslog&oldid=1110915683#Internet_standard_documents)
proscribing fields, formats, lengths, and other technical constraints.
In this project, the term "syslog" is interchanged with "log".

## logging chaos

In practice, most log file formats are an ad-hoc format that
may not follow any formal definition.

The following real-world example log files are available in project directory
`./logs`.

For example, the open-source nginx web server
[logs access attempts in an ad-hoc format](https://docs.nginx.com/nginx/admin-guide/monitoring/logging/#setting-up-the-access-log) in the file `access.log`

```text
192.168.0.115 - - [08/Oct/2022:22:26:35 +0000] "GET /DOES-NOT-EXIST HTTP/1.1" 404 0 "-" "curl/7.76.1" "-"
```

which is an entirely dissimilar log format to the neighboring nginx log file,
`error.log`

```text
2022/10/08 22:26:35 [error] 6068#6068: *3 open() "/usr/share/nginx/html/DOES-NOT-EXIST" failed (2: No such file or directory), client: 192.168.0.115, server: _, request: "GET /DOES-NOT-EXIST HTTP/1.0", host: "192.168.0.100"
```

nginx is following the example set by the apache web server (a bad example!).

<br/>

Commercial computer appliance vendors; NAS vendors, router
vendors, etc., often use ad-hoc log message formatting that is even more
unpredictable.

For example, from the Netgear Orbi Router SOAP client per-host log file:

```text
[SOAPClient]{DEBUG}{2022-05-10 16:19:13}[soap.c:1060] generate soap request, action=ParentalControl, method=Authenticate
```

Here is a log snippet from a Synology DiskStation package _DownloadStation_:

```text
2019/06/23 21:13:34	(system) trigger DownloadStation 3.8.13-3519 Begin start-stop-status start
```

And a snippet from a Synology DiskStation OS log file `sfdisk.log`:

```text
2019-04-06T01:07:40-07:00 dsnet sfdisk: Device /dev/sdq change partition.
```

And a snippet from a Synology DiskStation OS log file `synobackup.log` on the
same host:

```text
info	2018/02/24 02:30:04	SYSTEM:	[Local][Backup Task Backup1] Backup task started.
```

(yes, those are tab characters)

Here are is a snippet from a Windows 10 Pro host, log file
`${env:SystemRoot}\debug\mrt.log`

```text
Microsoft Windows Malicious Software Removal Tool v5.83, (build 5.83.13532.1)
Started On Thu Sep 10 10:08:35 2020
```

And a snippet from the same Windows host, log file
`${env:SystemRoot}\comsetup.log`

```text
COM+[12:24:34]: ********************************************************************************
COM+[12:24:34]: Setup started - [DATE:05,27,2020 TIME: 12:24 pm]
COM+[12:24:34]: ********************************************************************************
```

And a snippet from the same Windows host, log file
`${env:SystemRoot}\DirectX.log`

```text
11/01/19 20:03:40: infinst: Installed file C:\WINDOWS\system32\xactengine2_1.dll
```

<br/>

To be fair to nginx, Netgear, Synology, and Microsoft, this chaotic logging
data is typical of commercial and open-source software. But it's a mess!

Hence the need for _Super Speedy Syslog Searcher_!

## Further Reading

- [`CHANGELOG.md`](./CHANGELOG.md)
- [`Extended-Thoughts.md`](./Extended-Thoughts.md)

---

<a href="https://stackexchange.com/users/216253/"><img src="https://stackexchange.com/users/flair/216253.png" width="208" height="58" alt="profile for @JamesThomasMoon on Stack Exchange, a network of free, community-driven Q&amp;A sites" title="profile for @JamesThomasMoon on Stack Exchange, a network of free, community-driven Q&amp;A sites" /></a>
