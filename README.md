# Super Speedy Syslog Searcher! (`s4`) <!-- omit in toc -->

Speedily search and merge log file entries by datetime.

_Super Speedy Syslog Searcher_ (`s4`) is a command-line tool to search
and merge log files by datetime, including log files that are compressed
(`.gz`, `.xz`), archived (`.tar`), utmpx login records (`utmp`, `wtmp`),
or Microsoft Event Logs (`.evtx`).
It will parse a variety of formal and ad-hoc log message datetime formats.

The first goal of `s4` is speedy searching and printing.

[![Build status](https://img.shields.io/github/actions/workflow/status/jtmoon79/super-speedy-syslog-searcher/rust.yml?branch=main&style=flat-square&logo=github)](https://github.com/jtmoon79/super-speedy-syslog-searcher/actions?query=workflow%3Arust)
[![docs.rs](https://img.shields.io/docsrs/super_speedy_syslog_searcher/latest?badge.svg&style=flat-square&logo=docsdotrs)](https://docs.rs/super_speedy_syslog_searcher/latest/)
[![License](https://img.shields.io/crates/l/super-speedy-syslog-searcher?style=flat-square)](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/main/LICENSE.txt)

[![crates.io version](https://img.shields.io/crates/v/super-speedy-syslog-searcher.svg?style=flat-square&logo=rust)](https://crates.io/crates/super-speedy-syslog-searcher)
[![crates.io downloads](https://img.shields.io/crates/d/super-speedy-syslog-searcher.svg?style=flat-square&logo=rust)](https://crates.io/crates/super-speedy-syslog-searcher)
[![codecov.io](https://img.shields.io/codecov/c/github/jtmoon79/super-speedy-syslog-searcher?style=flat-square&logo=codecov)](https://codecov.io/gh/jtmoon79/super-speedy-syslog-searcher)
[![Commits since](https://img.shields.io/github/commits-since/jtmoon79/super-speedy-syslog-searcher/latest.svg)](https://github.com/jtmoon79/super-speedy-syslog-searcher/commits/main)

---

<!-- TOC generated by Markdown All In One -->

<!---toc start--->
- [Use](#use)
  - [Install `super_speedy_syslog_searcher`](#install-super_speedy_syslog_searcher)
  - [Run `s4`](#run-s4)
  - [`--help`](#--help)
- [About](#about)
  - [Features](#features)
  - [Limitations](#limitations)
  - [Hacks](#hacks)
- [More](#more)
  - [Requesting Support For DateTime Formats; your particular log file](#requesting-support-for-datetime-formats-your-particular-log-file)
  - ["syslog" and other project definitions](#syslog-and-other-project-definitions)
  - [logging chaos; the problem `s4` solves](#logging-chaos-the-problem-s4-solves)
  - [Further Reading](#further-reading)
<!---toc end--->

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

Print the syslog lines after January 1, 2022 at 00:00:00

```lang-text
s4 /var/log -a 20220101
```

Print the syslog lines from January 1, 2022 00:00:00 to January 2, 2022

```lang-text
s4 /var/log -a 20220101 -b 20220102
```

or

```lang-text
s4 /var/log -a 20220101 -b @+1d
```

Print the syslog lines on January 1, 2022, from 12:00:00 to 16:00:00

```lang-text
s4 /var/log -a 20220101T120000 -b 20220101T160000
```

Print only the syslog lines since yesterday at this time

```lang-text
s4 /var/log -a=-1d
```

Print only the syslog lines that occurred two days ago
(with the help of GNU `date`)

```lang-text
s4 /var/log -a $(date -d "2 days ago" '+%Y%m%d') -b @+1d
```

Print only the syslog lines that occurred two days ago during the noon hour
(with the help of GNU `date`)

```lang-text
s4 /var/log -a $(date -d "2 days ago 12" '+%Y%m%dT%H%M%S') -b @+1h
```

Print only the syslog lines that occurred two days ago during the noon hour in
Bengaluru, India (timezone offset +05:30) and prepended with equivalent UTC
datetime (with the help of GNU `date`)

```lang-text
s4 /var/log -u -a $(date -d "2 days ago 12" '+%Y%m%dT%H%M%S+05:30') -b @+1h
```

### `--help`

```lang-text
Speedily search and merge log file entries by datetime.
DateTime filters may be passed to narrow the search. It aims to be very fast.

Usage: s4 [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Path(s) of log files or directories.
              Directories will be recursed. Symlinks will be followed.
              Paths may also be passed via STDIN, one per line. The user must
              supply argument "-" to signify PATHS are available from STDIN.

Options:
  -a, --dt-after <DT_AFTER>
          DateTime Filter After: print syslog lines with a datetime that is at
          or after this datetime. For example, "20200102T120000" or "-5d".
  -b, --dt-before <DT_BEFORE>
          DateTime Filter Before: print syslog lines with a datetime that is at
          or before this datetime.
          For example, "20200103T230000" or "@+1d+11h"
  -t, --tz-offset <TZ_OFFSET>
          Default timezone offset for datetimes without a timezone.
          For example, datetime string "20200102T120000" does not have a timezone
          offset so the -t value would be used.
          Example values, "-0800", "+02:00", or "EDT".
          Ambiguous named timezones will be rejected, e.g. "SST".
          To pass a value with leading "-" use "=" notation, e.g. "-t=-0800".
          If not passed then the local system timezone offset is used.
          [default: -08:00]
  -u, --prepend-utc
          Prepend DateTime in the UTC Timezone for every line.
  -l, --prepend-local
          Prepend DateTime in the Local Timezone for every line.
  -d, --prepend-dt-format <PREPEND_DT_FORMAT>
          Prepend DateTime using strftime format string.
          [default: %Y%m%dT%H%M%S%.3f%z]
  -n, --prepend-filename
          Prepend file basename to every line.
  -p, --prepend-filepath
          Prepend file full path to every line.
  -w, --prepend-file-align
          Align column widths of prepended data.
      --prepend-separator <PREPEND_SEPARATOR>
          Separator string for prepended data. [default: :]
      --separator <LOG_MESSAGE_SEPARATOR>
          An extra separator string between printed log messages.
          One "syslog line", or "sysline", may have multiple lines of text.
          Accepts a basic set of backslash escape sequences,
          e.g. "\0" for the null character.
  -c, --color <COLOR_CHOICE>
          Choose to print to terminal using colors.
          [default: auto] [possible values: always, auto, never]
  -z, --blocksz <BLOCKSZ>
          Read blocks of this size in bytes.
          May pass value as any radix (hexadecimal, decimal, octal, binary).
          Using the default value is recommended.
          Most useful for developers. [default: 65535]
  -s, --summary
          Print a summary of files processed to stderr.
          Most useful for developers.
  -h, --help
          Print help
  -V, --version
          Print version

DateTime Filters may be strftime specifier patterns:
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

Or, DateTime Filter may be custom relative offset patterns:
    "+DwDdDhDmDs" or "-DwDdDhDmDs"
    "@+DwDdDhDmDs" or "@-DwDdDhDmDs"

Pattern "+%s" is Unix epoch timestamp in seconds with a preceding "+".
For example, value "+946684800" is be January 1, 2000 at 00:00, GMT.

Custom relative offset pattern "+DwDdDhDmDs" and "-DwDdDhDmDs" is the offset
from now (program start time) where "D" is a decimal number.
Each lowercase identifier is an offset duration:
"w" is weeks, "d" is days, "h" is hours, "m" is minutes, "s" is seconds.
For example, value "-1w22h" is one week and twenty-two hours in the past.
Value "+30s" is thirty seconds in the future.

Custom relative offset pattern "@+DwDdDhDmDs" and "@-DwDdDhDmDs" is relative
offset from the other datetime.
Arguments "-a 20220102 -b @+1d" are equivalent to "-a 20220102 -b 20220103".
Arguments "-a @-6h -b 20220101T120000" are equivalent to
"-a 20220101T060000 -b 20220101T120000".

Without a timezone offset (strftime specifier "%z" or "%Z"),
the Datetime Filter is presumed to be the local system timezone.

Ambiguous named timezones will be rejected, e.g. "SST".

Backslash escape sequences accepted by "--separator" are:
    "\0",
    "\a",
    "\b",
    "\e",
    "\f",
    "\n",
    "\r",
    "\\",
    "\t",
    "\v",

Resolved values of "--dt-after" and "--dt-before" can be reviewed in
the "--summary" output.

DateTime strftime specifiers are described at https://docs.rs/chrono/latest/chrono/format/strftime/

DateTimes supported are only of the Gregorian calendar.

DateTimes supported language is English.
```

## About

_Super Speedy Syslog Searcher_ (`s4`) is meant to aid Engineers in reviewing
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

`s4` will print log messages from multiple log files in datetime-sorted order.
A "window" of datetimes may be passed, to constrain the period of printed
messages. This will assist an engineer that, for example, needs to view all
syslog messages that occurred two days ago among log files taken from multiple
systems.

The ulterior motive for _Super Speedy Syslog Searcher_ was the [primary
developer](https://github.com/jtmoon79) wanted an excuse to learn rust 🦀,
and wanted to create an open-source tool for a recurring need of some
Software Test Engineers 😄

A longer rambling pontification about this project is in
[`Extended-Thoughts.md`](./Extended-Thoughts.md).

### Features

- Prepends datetime and file paths, for easy programmatic parsing or visual traversal of varying
  syslog messages
- Recognizes multi-line log messages
- Parses [utmpx login record format](https://en.wikipedia.org/w/index.php?title=Utmp&oldid=1143684808#utmpx,_wtmpx_and_btmpx) files
- Parses formal datetime formats:
  - [RFC 2822](https://www.rfc-editor.org/rfc/rfc2822#section-3.3)
  - [RFC 3164](https://www.rfc-editor.org/rfc/rfc3164#section-4.1.2)
  - [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339#section-5.8)
  - [RFC 5424](https://www.rfc-editor.org/rfc/rfc5424#section-6.2.3)
  - [ISO 8601](https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1113067353#General_principles) \*\*
- Parses [Windows Event Log] files \*\*\*
- Parses many ad-hoc datetime formats
  - Tested against "in the wild" log files from varying Linux distributions
    (see project path `./logs/`)
- Comparable speed as GNU `grep` and `sort`
  (see project tool `./tools/compare-grep-sort.sh`; run in github Actions, Job
  _run `s4`_, Step _Run script compare-grep-sort_)
- Processes invalid UTF-8
- Accepts arbitrarily large files \*\*\*\*

[Windows Event Log]: https://learn.microsoft.com/en-us/windows/win32/wes/eventmanifestschema-schema

### Limitations

- Only processes UTF-8 or ASCII encoded log files.
  ([Issue #16](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/16))
- Cannot processes multi-file `.gz` files (only processes first stream found)
  ([Issue #8](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/8))
- Cannot processes multi-file `.xz` files (only processes first stream found)
  ([Issue #11](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/11))
- Cannot process archive files or compressed files within other archive files
  or compressed files,
  e.g. `logs.tgz` ([Issue #14](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/14))
  e.g. file `syslog.xz` file within archive `logs.tar`
- Cannot process `.zip` archives ([Issue #39](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/39))
- \*\* ISO 8601
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
- \*\*\* Does not reorder chronologically "out of order" Windows Event Log
  Events. [Issue #86]
- \*\*\*\* Only for unarchived, uncompressed files
  ([Issue #9](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/9),
  [Issue #12](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/12),
  [Issue #13](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/13))

[Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86

### Hacks

- Entire `.xz` files are read into memory during the initial `open`
  ([Issue #12](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/12))

<br/>

## More

### Requesting Support For DateTime Formats; your particular log file

If you have found a log file that _Super Speedy Syslog Searcher_ does not parse
then you may create a [new Issue type _Feature request (datetime format)_](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/new/choose).

Here is [an example user-submitted Issue](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/81).

### "syslog" and other project definitions

In this project, the term "_syslog_" is used generously to refer to any
log message that has a datetime stamp on the first line of log text.

Technically, "_syslog_" is [defined among several RFCs](https://en.wikipedia.org/w/index.php?title=Syslog&oldid=1110915683#Internet_standard_documents)
proscribing fields, formats, lengths, and other technical constraints.
In this project, the term "_syslog_" is interchanged with "_log_".

The term "_sysline_" refers to a one log message which may comprise
multiple text lines.

See [docs section _Definitions of data_](https://docs.rs/super_speedy_syslog_searcher/latest/s4lib/data/index.html)
for more project definitions.

### logging chaos; the problem `s4` solves

In practice, most log file formats are an ad-hoc format that
may not follow any formal definition. Sorting varying log messages by datetime
is prohibitively tedious.

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

nginx is following the bad example set by the apache web server.

<br/>

Commercial software and computer hardware vendors nearly always use
ad-hoc log message formatting that is even more unpredictable among each log
file on the same system.

<br/>

Here is a log snippet from a Debian 11 host, file `/var/log/alternatives.log`:

```text
update-alternatives 2022-10-10 23:59:47: run with --quiet --remove rcp /usr/bin/ssh
```

And a snippet from the same Debian 11 host, file `/var/log/alternatives.log`:

```text
2022-10-10 15:15:02 upgrade gpgv:amd64 2.2.27-2 2.2.27-2+deb11u1
```

And a snippet from the same Debian 11 host, file `/var/log/kern.log`:

```text
Oct 10 23:07:16 debian11-b kernel: [    0.10034] Linux version 5.10.0-11-amd64
```

And a snippet from the same Debian 11 host,
file `/var/log/unattended-upgrades/unattended-upgrades-shutdown.log`:

```text
2022-10-10 23:07:16,775 WARNING - Unable to monitor PrepareForShutdown() signal, polling instead.
```

<br/>

Here is a log snippet from a Synology DiskStation package _DownloadStation_:

```text
2019/06/23 21:13:34	(system) trigger DownloadStation 3.8.13-3519 Begin start-stop-status start
```

And a snippet from a Synology DiskStation OS log file `sfdisk.log` on the same
host:

```text
2019-04-06T01:07:40-07:00 dsnet sfdisk: Device /dev/sdq change partition.
```

And a snippet from a Synology DiskStation OS log file `synobackup.log` on the
same host:

```text
info	2018/02/24 02:30:04	SYSTEM:	[Local][Backup Task Backup1] Backup task started.
```

(yes, those are tab characters)

<br/>

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

And a snippet from the same Windows host, log file
`${env:SystemRoot}/Microsoft.NET/Framework/v4.0.30319/ngen.log`

```text
09/15/2022 14:13:22.951 [515]: 1>Warning: System.IO.FileNotFoundException: Could not load file or assembly
```

And a snippet from the same Windows host, log file
`${env:SystemRoot}/Performance/WinSAT/winsat.log`

```text
68902359 (21103) - exe\logging.cpp:0841: --- START 2022\5\17 14:26:09 PM ---
68902359 (21103) - exe\main.cpp:4363: WinSAT registry node is created or present
```

(yes, it reads hour `14`, and `PM`…  🙄)

<br/>

This chaotic logging approach is typical of commercial and open-source software.
And it's a mess!
Attempting to sort log messages by their natural sort mechanism,
a datetime stamp, is difficult to impossible.

Hence the need for _Super Speedy Syslog Searcher_! 🦸

### Further Reading

- [`CHANGELOG.md`](./CHANGELOG.md)
- [`Extended-Thoughts.md`](./Extended-Thoughts.md)

---

<a href="https://stackexchange.com/users/216253/">
<img src="https://stackexchange.com/users/flair/216253.png" width="208" height="58" alt="profile for @JamesThomasMoon on Stack Exchange, a network of free, community-driven Q&amp;A sites" title="profile for @JamesThomasMoon on Stack Exchange, a network of free, community-driven Q&amp;A sites" />
</a>
