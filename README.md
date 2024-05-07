# Super Speedy Syslog Searcher! (`s4`) <!-- omit in toc -->

Speedily search and merge log messages by datetime.

<!--
* Must update parameters `branch=` and `version=` per release; replace the version
-->

[![MSRV](https://img.shields.io/crates/msrv/super_speedy_syslog_searcher/0.6.71?logo=rust&logoColor=800000&cacheSeconds=6000)](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/0.6.71/Cargo.toml#L15)
[![License](https://img.shields.io/crates/l/super-speedy-syslog-searcher?style=flat-square)](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/main/LICENSE.txt)
[![docs.rs](https://img.shields.io/docsrs/super_speedy_syslog_searcher/0.6.71?badge.svg&style=flat-square&logo=docsdotrs)](https://docs.rs/super_speedy_syslog_searcher/0.6.71/)

[![crates.io version](https://img.shields.io/crates/v/super-speedy-syslog-searcher.svg?style=flat-square&logo=rust&logoColor=800000?branch=0.6.71&version=0.6.71)](https://crates.io/crates/super-speedy-syslog-searcher/0.6.71)
[![crates.io downloads](https://img.shields.io/crates/d/super-speedy-syslog-searcher.svg?style=flat-square&logo=rust&logoColor=800000)](https://crates.io/crates/super-speedy-syslog-searcher#:~:text=Downloads%20all%20time)
[![crates.io downloads (version)](https://img.shields.io/crates/dv/super_speedy_syslog_searcher/0.6.71?style=flat-square&logo=rust&logoColor=800000)](https://crates.io/crates/super-speedy-syslog-searcher/0.6.71)
[![lib.rs](https://img.shields.io/badge/lib.rs-white?style=flat-square&logo=rust&logoColor=202020)](https://lib.rs/crates/super_speedy_syslog_searcher/)

[![Build status](https://img.shields.io/github/actions/workflow/status/jtmoon79/super-speedy-syslog-searcher/rust.yml?branch=0.6.71&style=flat-square&logo=github&logoColor=000000)](https://github.com/jtmoon79/super-speedy-syslog-searcher/actions?query=workflow%3Arust)
[![coveralls.io](https://img.shields.io/coverallsCoverage/github/jtmoon79/super-speedy-syslog-searcher?style=flat-square&logo=coveralls&logoColor=b94947&branch=0.6.71&version=0.6.71)](https://coveralls.io/github/jtmoon79/super-speedy-syslog-searcher?branch=0.6.71)
[![Commits since](https://img.shields.io/github/commits-since/jtmoon79/super-speedy-syslog-searcher/0.6.71.svg?logo=github&logoColor=000000)](https://github.com/jtmoon79/super-speedy-syslog-searcher/commits/main)

_Super Speedy Syslog Searcher_ (`s4`) is a command-line tool to search
and merge varying log messages from varying log files, sorted by datetime.
Datetime filters may be passed to narrow the search to a datetime range.

`s4` can read standardized log message formats like RFC 3164 and RFC 5424
("syslog"),
Red Hat Audit logs, and can read many non-standardized ad-hoc log message
formats, including multi-line log messages.
It also parses binary accounting records acct, lastlog, and utmp
(`acct`, `pacct`, `lastlog`, `utmp`, `utmpx`, `wtmp`),
systemd journal logs (`.journal`), and Microsoft Event Logs (`.evtx`).
`s4` can read compressed logs (`.gz`, `.lz4`, `.xz`), or archived logs (`.tar`).<sup><a href="#f3">\[3\]</a></sup>

The first goal of `s4` is speedy searching and printing.

---

<!-- TOC generated by Markdown All In One -->

<!---toc start--->
- [Use](#use)
  - [Install `super_speedy_syslog_searcher`](#install-super_speedy_syslog_searcher)
  - [Run `s4`](#run-s4)
  - [`--help`](#--help)
- [About](#about)
  - [Why `s4`?](#why-s4)
  - [Features](#features)
  - [Limitations](#limitations)
  - [Hacks](#hacks)
- [More](#more)
  - [Comparisons](#comparisons)
    - [General Features](#general-features)
    - [Formal Log DateTime Supported](#formal-log-datetime-supported)
    - [Other Log or File Formats Supported](#other-log-or-file-formats-supported)
    - [Archive Formats Supported](#archive-formats-supported)
    - [Speed Comparison](#speed-comparison)
  - [Building locally](#building-locally)
  - [Parsing `.journal` files](#parsing-journal-files)
  - [Requesting Support For DateTime Formats; your particular log file](#requesting-support-for-datetime-formats-your-particular-log-file)
  - ["syslog" and other project definitions](#syslog-and-other-project-definitions)
    - [syslog](#syslog)
    - [log message](#log-message)
- [logging chaos: the problem `s4` solves](#logging-chaos-the-problem-s4-solves)
  - [open-source software examples](#open-source-software-examples)
    - [nginx webserver](#nginx-webserver)
    - [Debian 11](#debian-11)
    - [binary files](#binary-files)
  - [commercial software examples](#commercial-software-examples)
    - [Synology DiskStation](#synology-diskstation)
    - [Mac OS 12](#mac-os-12)
    - [Microsoft Windows 10](#microsoft-windows-10)
  - [Summary](#summary)
- [Further Reading](#further-reading)
- [Stargazers](#stargazers)
<!---toc end--->

---

## Use

### Install `super_speedy_syslog_searcher`

Assuming [rust is installed], run

```lang-text
cargo install --locked super_speedy_syslog_searcher
```

[rust is installed]: https://www.rust-lang.org/tools/install

### Run `s4`

For example, print all the log messages in syslog files under `/var/log/`

```lang-text
s4 /var/log
```

On Windows, print the ad-hoc logs under `C:\Windows\Logs`

```lang-text
s4.exe C:\Windows\Logs
```

On Windows, print all `.log` files under `C:\Windows` (with the help of Powershell)

```lang-powershell
Get-ChildItem -Filter '*.log' -File -Path "C:\Windows" -Recurse -ErrorAction SilentlyContinue `
  | Select-Object -ExpandProperty FullName `
  | s4.exe -
```

<sup>(note that UTF-16 encoded logs cannot be parsed, see [Issue #16])</sup>

<sup>(note that opening too many files causes error _too many files open_, see [Issue #270], so `Get-ChildItem -Filter` lessens the number of files opened by `s4.exe`)</sup>

On Windows, print the [Windows Event logs]

```lang-text
s4.exe C:\Windows\System32\winevt\Logs
```

Print the log messages after January 1, 2022 at 00:00:00

```lang-text
s4 /var/log -a 20220101
```

Print the log messages from January 1, 2022 00:00:00 to January 2, 2022

```lang-text
s4 /var/log -a 20220101 -b 20220102
```

or

```lang-text
s4 /var/log -a 20220101 -b @+1d
```

Print the log messages on January 1, 2022, from 12:00:00 to 16:00:00

```lang-text
s4 /var/log -a 20220101T120000 -b 20220101T160000
```

Print the record-keeping log messages from up to a day ago
(with the help of `find`)

```lang-text
find /var -xdev -type f \( \
    -name 'lastlog' \
    -or -name 'wtmp' \
    -or -name 'wtmpx' \
    -or -name 'utmp' \
    -or -name 'utmpx' \
    -or -name 'acct' \
    -or -name 'pacct' \
  \) \
    2>/dev/null \
    | s4 - -a=-1d
```

Print the journal log messages from up to an hour ago,
prepending the journal file name
(with the help of `find`)

```lang-text
find / -xdev -name '*.journal' -type f 2>/dev/null \
    | s4 - -a=-1h -n
```

Print only the log messages that occurred two days ago
(with the help of GNU `date`)

```lang-text
s4 /var/log -a $(date -d "2 days ago" '+%Y%m%d') -b @+1d
```

Print only the log messages that occurred two days ago during the noon hour
(with the help of GNU `date`)

```lang-text
s4 /var/log -a $(date -d "2 days ago 12" '+%Y%m%dT%H%M%S') -b @+1h
```

Print only the log messages that occurred two days ago during the noon hour in
Bengaluru, India (timezone offset +05:30) and prepended with equivalent UTC
datetime (with the help of GNU `date`)

```lang-text
s4 /var/log -u -a $(date -d "2 days ago 12" '+%Y%m%dT%H%M%S+05:30') -b @+1h
```

[Windows Event logs]: https://github.com/libyal/libevtx/blob/20221101/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
[Issue #16]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/16
[Issue #270]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/270

### `--help`

```lang-text
Speedily search and merge log messages by datetime.
DateTime filters may be passed to narrow the search.
It aims to be very fast.

Usage: s4 [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Path(s) of log files or directories.
              Directories will be recursed. Symlinks will be followed.
              Paths may also be passed via STDIN, one per line. The user must
              supply argument "-" to signify PATHS are available from STDIN.

Options:
  -a, --dt-after <DT_AFTER>
          DateTime Filter After: print log messages with a datetime that is at
          or after this datetime. For example, "20200102T120000" or "-5d".
  -b, --dt-before <DT_BEFORE>
          DateTime Filter Before: print log messages with a datetime that is at
          or before this datetime.
          For example, "2020-01-03T23:00:00.321-05:30" or "@+1d+11h"
  -t, --tz-offset <TZ_OFFSET>
          Default timezone offset for datetimes without a timezone.
          For example, log message "_\[20200102T120000\] Starting service_" has a
          datetime substring "20200102T120000".
          The datetime substring does not have a timezone offset
          so the TZ_OFFSET value would be used.
          Example values, "+12", "-0800", "+02:00", or "EDT".
          To pass a value with leading "-" use "=" notation, e.g. "-t=-0800".
          If not passed then the local system timezone offset is used.
          [default: -07:00]
  -z, --prepend-tz <PREPEND_TZ>
          Prepend a DateTime in the timezone PREPEND_TZ for every line.
          Used in PREPEND_DT_FORMAT.
  -u, --prepend-utc
          Prepend a DateTime in the UTC timezone offset for every line.
          This is the same as "--prepend-tz Z".
          Used in PREPEND_DT_FORMAT.
  -l, --prepend-local
          Prepend DateTime in the local system timezone offset for every line.
          This is the same as "--prepend-tz +XX" where +XX is the local system
          timezone offset.
          Used in PREPEND_DT_FORMAT.
  -d, --prepend-dt-format <PREPEND_DT_FORMAT>
          Prepend a DateTime using the strftime format string.
          If PREPEND_TZ is set then that value is used for any timezone offsets,
          i.e. strftime "%z" "%:z" "%Z" values, otherwise the timezone offset value
          is the local system timezone offset.
          [Default: %Y%m%dT%H%M%S%.3f%z]
  -n, --prepend-filename
          Prepend file basename to every line.
  -p, --prepend-filepath
          Prepend file full path to every line.
  -w, --prepend-file-align
          Align column widths of prepended data.
      --prepend-separator <PREPEND_SEPARATOR>
          Separator string for prepended data.
          [default: :]
      --separator <LOG_MESSAGE_SEPARATOR>
          An extra separator string between printed log messages.
          Per log message not per line of text.
          Accepts a basic set of backslash escape sequences,
          e.g. "\0" for the null character.
      --journal-output <JOURNAL_OUTPUT>
          The format for .journal file log messages.
          Matches journalctl --output options.
          [default: short]
          [possible values: short, short-precise, short-iso, short-iso-precise,
          short-full, short-monotonic, short-unix, verbose, export, cat]
  -c, --color <COLOR_CHOICE>
          Choose to print to terminal using colors.
          [default: auto]
          [possible values: always, auto, never]
      --blocksz <BLOCKSZ>
          Read blocks of this size in bytes.
          May pass value as any radix (hexadecimal, decimal, octal, binary).
          Using the default value is recommended.
          Most useful for developers.
          [default: 65535]
  -s, --summary
          Print a summary of files processed to stderr.
          Most useful for developers.
  -h, --help
          Print help
  -V, --version
          Print version

DateTime Filters may be strftime specifier patterns:
    "%Y%m%dT%H%M%S*"
    "%Y-%m-%d %H:%M:%S*"
    "%Y-%m-%dT%H:%M:%S*"
    "%Y/%m/%d %H:%M:%S*"
    "%Y%m%d"
    "%Y-%m-%d"
    "%Y/%m/%d"
    "+%s"
Each * is an optional trailing 3-digit fractional sub-seconds,
or 6-digit fractional sub-seconds, and/or timezone.

Pattern "+%s" is Unix epoch timestamp in seconds with a preceding "+".
For example, value "+946684800" is be January 1, 2000 at 00:00, GMT.

DateTime Filters may be custom relative offset patterns:
    "+DwDdDhDmDs" or "-DwDdDhDmDs"
    "@+DwDdDhDmDs" or "@-DwDdDhDmDs"

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

Without a timezone, the Datetime Filter is presumed to be the local
system timezone.

Command-line passed timezones may be numeric timezone offsets,
e.g. "+09:00", "+0900", or "+09", or named timezone offsets, e.g. "JST".
Ambiguous named timezones will be rejected, e.g. "SST".

--prepend-tz and --dt-offset function independently:
--dt-offset is used to interpret processed log message datetime stamps that
do not have a timezone offset.
--prepend-tz affects what is pre-printed before each printed log message line.

--separator accepts backslash escape sequences:
    "\0","\a","\b","\e","\f","\n","\r","\\","\t","\v",

Resolved values of "--dt-after" and "--dt-before" can be reviewed in
the "--summary" output.

DateTime strftime specifiers are described at
https://docs.rs/chrono/latest/chrono/format/strftime/

DateTimes supported are only of the Gregorian calendar.

DateTimes supported language is English.

Is s4 failing to parse a log file? Report an Issue at
https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/new/choose
```

---

## About

### Why `s4`?

_Super Speedy Syslog Searcher_ (`s4`) is meant to aid Engineers in reviewing
varying log files in a datetime-sorted manner.
The primary use-case is to aid investigating problems wherein the time of
a problem occurrence is known and there are many available logs
but otherwise there is little source evidence.

Currently, log file formats vary widely. _Most_ logs are an ad-hoc format.
Even separate log files on the same system for the same service may have
different message formats!
Sorting these logged messages by datetime may be prohibitively difficult.
The result is an engineer may have to "hunt and peck" among many log files,
looking for problem clues around some datetime; so tedious!

Enter _Super Speedy Syslog Searcher_ 🦸 ‼

`s4` will print log messages from multiple log files in datetime-sorted order.
A "window" of datetimes may be passed, to constrain the period of printed
messages. This will assist an engineer that, for example, needs to view all
log messages that occurred two days ago between 12:00 and 12:05 among log files taken from multiple
systems.

The ulterior motive for _Super Speedy Syslog Searcher_ was the [primary
developer](https://github.com/jtmoon79) wanted an excuse to learn rust 🦀,
and wanted to create an open-source tool for a recurring need of some
Software Test Engineers 😄

See the real-world example rationale in the section below,
[_logging chaos: the problem `s4` solves_].

[_logging chaos: the problem `s4` solves_]: #logging-chaos-the-problem-s4-solves

### Features

- Parses:
  - Ad-hoc log messages using <span id="formal-datetimes">formal datetime formats</span>:
    - [Internet Message Format (RFC 2822)]<br/>e.g. _Wed, 1 Jan 2020 22:00:00 PST message…_
    - [The BSD syslog Protocol (RFC 3164)]<br/>e.g. _\<8\>Jan 1 22:00:00 message…_
    - [Date and Time on the Internet: Timestamps (RFC 3339)]<br/>e.g. _2020-01-01T22:00:00-08:00 message…_
    - [The Syslog Protocol (RFC 5424)]<br/>e.g. _2020-01-01T22:00:00-08:00 message…_
    - [ISO 8601]<br/>e.g. _2020-01-01T22:00:00-08:00 message…_, _20200101T220000-0800 message…_, etc. <sup><a href="#f1">\[1\]</a></sup>
  - [Red Hat Audit Log] files
  - binary user accounting records files
    ([`acct`, `pacct`], [`lastlog`], [`utmp`, `utmpx`])
    from multiple Operating Systems and CPU architectures
  - binary [Windows Event Log] files
  - binary [systemd journal] files with printing options matching [`journalctl`]
  - many varying text log messages with ad-hoc datetime formats
  - multi-line log messages
- Inspects `.tar` archive files for parseable log files <sup><a href="#f2">\[2\]</a></sup>
- Inspects `.gz` and `.xz` compressed files for parseable log files <sup><a href="#f3">\[3\]</a></sup>
- Tested against "in the wild" log files from varying sources
  (see project path [`./logs/`])
- Prepends datetime and file paths, for easy programmatic parsing or
  visual traversal of varying log messages
- [Comparable speed as GNU `grep` and `sort`](#speed-comparison)
- Processes invalid UTF-8
- Accepts arbitrarily large files <sup><a href="#hacks">see _Hacks_</a></sup>

[`acct`, `pacct`]: https://www.man7.org/linux/man-pages/man5/acct.5.html
[`lastlog`]: https://man.netbsd.org/lastlog.5
[`utmp`, `utmpx`]: https://en.wikipedia.org/w/index.php?title=Utmp&oldid=1143684808#utmpx,_wtmpx_and_btmpx
[Internet Message Format (RFC 2822)]: https://www.rfc-editor.org/rfc/rfc2822#section-3.3
[The BSD syslog Protocol (RFC 3164)]: https://www.rfc-editor.org/rfc/rfc3164#section-4.1.2
[Date and Time on the Internet: Timestamps (RFC 3339)]: https://www.rfc-editor.org/rfc/rfc3339#section-5.8
[The Syslog Protocol (RFC 5424)]: https://www.rfc-editor.org/rfc/rfc5424#section-6.2.3
[ISO 8601]: https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1113067353#General_principles
[Red Hat Audit Log]: https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/security_guide/sec-understanding_audit_log_files
[Windows Event Log]: https://learn.microsoft.com/en-us/windows/win32/wes/windows-event-log
[systemd journal]: https://systemd.io/JOURNAL_FILE_FORMAT/
[`journalctl`]: https://www.man7.org/linux/man-pages/man1/journalctl.1.html
[`./logs/`]: https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/main/logs

### Limitations

- Only processes UTF-8 or ASCII encoded syslog files. ([Issue #16])
- Cannot process multi-file `.gz` files (only processes first stream found).
  ([Issue #8])
- Cannot process multi-file `.xz` files (only processes first stream found).
  ([Issue #11])
- Cannot process `.zip` archives ([Issue #39])
- <span id="f1"><sup>\[1\]</sup></span> ISO 8601
  - ISO 8601 forms recognized
  (using [ISO descriptive format])
    - `YYYY-MM-DDThh:mm:ss`
    - `YYYY-MM-DDThhmmss`
    - `YYYYMMDDThhmmss`
    (may use date-time separator character `'T'` or character blank space `' '`)
  - ISO 8601 forms not recognized:
    - Absent seconds
    - [_Ordinal dates_], i.e. "day of the year", format `YYYY-DDD`, e.g. `"2022-321"`
    - [_Week dates_], i.e. "week-numbering year", format `YYYY-Www-D`, e.g. `"2022-W25-1"`
    - times [without minutes and seconds] (i.e. only `hh`)
- <span id="f2"><sup>\[2\]</sup></span> Cannot process archive files or compressed files within other
  archive files or compressed files ([Issue #14])<br/>
  e.g. cannot process `logs.tar.xz`
- <span id="f3"><sup>\[3\]</sup></span> Can only process compressed text log files; does not process compressed `.journal` files ([Issue #284]), `.evtx` files, or accounting records like `wtmp` ([Issue #285])

[Issue #8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/8
[Issue #11]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/11
[Issue #14]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/14
[Issue #12]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/12
[Issue #39]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/39
[Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86
[Issue #284]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/284
[Issue #285]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/285
[ISO descriptive format]: https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Calendar_dates
[_Ordinal dates_]: https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Ordinal_dates
[_Week dates_]: https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Week_dates
[without minutes and seconds]: https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114310323#Times

### Hacks

- Entire `.lz4` files are read once before processing ([Issue #293])
- Entire `.xz` files are read into memory before printing ([Issue #12])
- Entire `.evtx` files are read into memory before printing ([Issue #86])
- Entire [user accounting record files are read into memory] before printing

[user accounting record files are read into memory]: https://docs.rs/super_speedy_syslog_searcher/0.6.70/s4lib/readers/fixedstructreader/struct.FixedStructReader.html#summary-of-operation
[Issue #293]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/293

<br/>

---

## More

### Comparisons

An overview of features of varying log mergers including GNU tools.

- GNU _`grep`_ piped to GNU _`sort`_
- _Super Speedy Syslog Searcher_; `s4`
- [_logmerger_](https://github.com/ptmcg/logmerger); `logmerger`
- [_Toolong_](https://github.com/Textualize/toolong); `tl`
- [_logdissect_](https://github.com/dogoncouch/logdissect); `logdissect.py`

|Symbol| |
|-     |-|
|✔    |_Yes_  |
|⬤    |_Most_  |
|◒    |_Some_ |
|✗    |_No_   |
|☐    |_with an accompanying GNU program_ |
|!     |_with user input_                 |
|‼     |_with complex user input_         |

---

#### General Features

|Program        |Source|CLI|TUI|Interactive|live tail|merge varying log formats|datetime search range|
|-              |-     |-  |-  |-          |-        |-                        |- |
|`grep \| sort` |C     |✔  |✗ |✗          |☐ `tail`|✗                        |‼|
|`s4`           |Rust  |✔  |✗ |✗          |✗       |✔                        |✔|
|`logmerger`    |Python|✔  |✔ |✔          |✗       |‼                        |✔|
|`tl`           |Python|✔  |✔ |✔          |✔       |✗                        |✗|
|`logdissect.py`|Python|✔  |✗ |✗          |✗       |✗                        |✗|

---

#### Formal Log DateTime Supported

|Program                   |RFC 2822|RFC 3164|RFC 3339|RFC 5424|ISO 8601|
|-                         |-       |-       |-       |-       |-       |
|`grep \| sort`            |✗      |‼        |!       |!       |!       |
|`s4`                      |✔      |✔       |✔       |✔      |<a href="#formal-datetimes">⬤</a>|
|`logmerger`               |✗      |✗       |!       |!       |◒       |
|`tl`                      |✗      |✗       |✔       |✔      |✔       |

<!--
|`logdissect.py`           |!       |!       |!       |!       |!       |

XXX: I could not get `logdissect.py` to work for any "parser" for any standard RFC log file.
```bash
  for logfile in ./logs/standards/*.log ; do
    for lp in ciscoios emerge linejson sojson syslog syslogiso syslognohost tcpdump webaccess windowsrsyslog ; do
      (set -x;
      logdissect -p $lp $logfile) 2>/dev/null
    done
  done
```
-->

---

#### Other Log or File Formats Supported

|Program        |Ad-hoc text formats|Red Hat Audit Log|journal|`acct`/`lastlog`/`utmp`|`.evtx`|`.pcap`/`.pcapng`|`.jsonl`|
|-              |-                  |-                |-      |-                      |-      |-                |-       |
|`grep \| sort` |‼                  |!                |✗      |✗                     |✗      |✗               |✗       |
|`s4`           |✔                  |✔               |✔      |✔                     |✔      |[✗](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/255)|✔ |
|`logmerger`    |‼                  |‼                |✗      |✗                     |✗      |✔               |✗       |
|`tl`           |✗                  |✗               |✗      |✗                     |✗      |✗               |✔       |

---

#### Archive Formats Supported

|Program        |`.gz`     |`.lz4`   |`.bz`/`.bz2` |`.xz`   |`.tar`|`.zip`|
|-              |-         |-        |-            |-       |-     |-     |
|`grep \| sort` |☐ `zgrep`|☐ `lz4`|☐ `bzip2`    |☐ `xz` |✗     |✗     |
|`s4`           |✔        |✔        |[✗](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/40)|✔      |✔     |[✗](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/39)|
|`logmerger`    |✔        |✗        |✗            |✗      |✗     |✗     |
|`tl`           |✔        |✗        |✔            |✗      |✗     |✗     |
|`logdissect.py`|✔        |✗        |✗            |✗      |✗     |✗     |

---

#### Speed Comparison

A comparison of merging three large log files:

- 2000 line log file, 1116357 bytes (≈1.1 MB), with high-plane unicode
- 2500 line log file, 1078842 bytes (≈1.0 MB), with high-plane unicode
- 5000 line log file, 2158138 bytes (≈2.1 MB), with high-plane unicode

|Program       |real|user|sys |
|-             |-   |-   |-   |
|`grep \| sort`|0.04|0.04|0.00|
|`s4`          |0.04|0.03|0.04|
|`logmerger`   |0.78|0.74|0.00|
|`tl`          |0.94|0.55|0.06|

This informal runtime comparison used GNU `time` running on Ubuntu 22 on WSL2.

- `grep` 3.7, `sort` 8.32
- `s4` 0.6.71
- `logmerger` 0.8.0 on Python 3.10.12
- `tl` 1.4.0 on Python 3.10.12

See directory [compare-log-mergers] and results in [`compare-log-mergers.txt`].

[compare-log-mergers]: ./tools/compare-log-mergers/
[`compare-log-mergers.txt`]: ./releases/0.6.71/compare-log-mergers.txt

---

### Building locally

Building on Linux requires:

- `rust` _minimal_ or more
- `gcc` (which should install `cc`, `libc`, and `libc-headers`)

From the git cloned project directory run `cargo build`.

### Parsing `.journal` files

Requires `libsystemd` to be installed to use `libsystemd.so` at runtime.

### Requesting Support For DateTime Formats; your particular log file

If you have found a log file that _Super Speedy Syslog Searcher_ does not parse
then you may create a [new Issue type _Feature request (datetime format)_].

Here is [an example user-submitted Issue].

[new Issue type _Feature request (datetime format)_]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/new/choose
[an example user-submitted Issue]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/81

### "syslog" and other project definitions

#### syslog

In this project, the term "_syslog_" is used generously to refer to any
log message that has a datetime stamp on the first line of log text.

Technically, "_syslog_" is [defined among several RFCs]
proscribing fields, formats, lengths, and other technical constraints.
In this project, the term "_syslog_" is interchanged with "_log_".

The term "_sysline_" refers to a one log message which may comprise
multiple text lines.

See [docs section _Definitions of data_] for more project definitions.

[defined among several RFCs]: https://en.wikipedia.org/w/index.php?title=Syslog&oldid=1219545533#Internet_standard_documents
[docs section _Definitions of data_]: https://docs.rs/super_speedy_syslog_searcher/latest/s4lib/data/index.html

#### log message

A "log message" is a single log entry for any type of logging scheme;
an entry in a utmpx file, an entry in a systemd journal, an entry in a
Windows Event Log, a formal syslog message, or an ad-hoc log message.

---

## logging chaos: the problem `s4` solves

In practice, most log file formats are an ad-hoc format. And among formally
defined log formats, there are many variations. The result is merging varying
log messages by datetime is prohibitively tedious.

The following real-world example log files are available in project directory
`./logs`.

### open-source software examples

#### nginx webserver

For example, the open-source nginx web server
[logs access attempts in an ad-hoc format] in the file `access.log`

```text
192.168.0.115 - - [08/Oct/2022:22:26:35 +0000] "GET /DOES-NOT-EXIST HTTP/1.1" 404 0 "-" "curl/7.76.1" "-"
```

which is an entirely dissimilar log format to the neighboring nginx log file,
`error.log`

```text
2022/10/08 22:26:35 [error] 6068#6068: *3 open() "/usr/share/nginx/html/DOES-NOT-EXIST" failed (2: No such file or directory), client: 192.168.0.115, server: _, request: "GET /DOES-NOT-EXIST HTTP/1.0", host: "192.168.0.100"
```

nginx is following the bad example set by the apache web server.

#### Debian 11

Here are log snippets from a Debian 11 host.

file `/var/log/alternatives.log`

```text
update-alternatives 2022-10-10 23:59:47: run with --quiet --remove rcp /usr/bin/ssh
```

file `/var/log/dpkg.log`

```text
2022-10-10 15:15:02 upgrade gpgv:amd64 2.2.27-2 2.2.27-2+deb11u1
```

file `/var/log/kern.log`

```text
Oct 10 23:07:16 debian11-b kernel: [    0.10034] Linux version 5.10.0-11-amd64
```

file `/var/log/unattended-upgrades/unattended-upgrades-shutdown.log`

```text
2022-10-10 23:07:16,775 WARNING - Unable to monitor PrepareForShutdown() signal, polling instead.
```

#### binary files

And then there are binary files, such as the `wtmp` file on Linux and other
Unix Operating Systems.
Using tool `utmpdump`, a `utmp` record structure is converted to text like:

```text
[7] [12103] [ts/0] [user] [pts/0] [172.1.2.1] [172.1.2.2] [2023-03-05T23:12:36,270185+00:00]
```

And from a _systemd_ `.journal` file, read using `journalctl`

```text
Mar 03 10:26:10 host systemd[1]: Started OpenBSD Secure Shell server.
░░ Subject: A start job for unit ssh.service has finished successfully
░░ Defined-By: systemd
░░ Support: http://www.ubuntu.com/support
░░
░░ A start job for unit ssh.service has finished successfully.
░░
░░ The job identifier is 120.
Mar 03 10:31:23 host sshd[4559]: Accepted login for user1 from 172.1.2.1 port 51730 ssh2
```

Try merging those two log messages by datetime using GNU `grep`, `sort`, `sed`,
or `awk`!

Additionally, if the `wtmp` file is from a different architecture
or Operating System, then the binary record structure is likely not parseable
by the resident `utmpdump` tool. What then!?

### commercial software examples

Commercial software and computer hardware vendors nearly always use
ad-hoc log message formatting that is even more unpredictable among each log
file on the same system.

#### Synology DiskStation

Here are log file snippets from a Synology DiskStation host.

file `DownloadStation.log`

```text
2019/06/23 21:13:34	(system) trigger DownloadStation 3.8.13-3519 Begin start-stop-status start
```

file `sfdisk.log`

```text
2019-04-06T01:07:40-07:00 dsnet sfdisk: Device /dev/sdq change partition.
```

file `synobackup.log`

```text
info	2018/02/24 02:30:04	SYSTEM:	[Local][Backup Task Backup1] Backup task started.
```

(yes, those are tab characters)

#### Mac OS 12

Here are log file snippets from a Mac OS 12.6 host.

file `/var/log/system`

```text
Sep 22 00:01:21 Mac-mini syslogd[108]: ASL Sender Statistics
Sep 22 00:06:22 Mac-mini halt[78568]: SHUTDOWN_TIME: 1695341182 412865
Oct 11 15:04:44 localhost bootlog[0]: BOOT_TIME 1697036684 0
Oct 11 15:04:55 localhost syslogd[110]: Configuration Notice:
	ASL Module "com.apple.cdscheduler" claims selected messages.
	Those messages may not appear in standard system log files or in the ASL database.
```

file `/var/log/wifi`

```text
Thu Sep 21 23:05:35.850 Usb Host Notification NOT activated
Fri Sep 22 15:05:01.994 Usb Host Notification activated!
```

file `/var/log/fsck_hs.log`

```text
/dev/rdisk2s2: fsck_hfs started at Thu Sep 21 21:31:05 2023
/dev/rdisk2s2: /dev/rdisk2s2: ** /dev/rdisk2s2 (NO WRITE)
/dev/rdisk2s2:    Executing fsck_hfs (version hfs-583.100.10).
  QUICKCHECK ONLY; FILESYSTEM CLEAN
/dev/rdisk2s2: fsck_hfs completed at Thu Sep 21 21:31:05 2023
```

file `/var/log/anka.log`

```text
Fri Sep 22 00:06:05 UTC 2023: Checking /Library/Developer/CoreSimulator/Volumes/watchOS_20S75...
Fri Sep 22 00:06:05 UTC 2023: No updates found
```

file `/var/log/displaypolicyd.log`

```text
2023-09-15 04:26:56.329071-0700: Version: V6.5.7 starting [0]
2023-09-15 04:26:56.330256-0700: Started at Fri Sep 15 04:26:56 2023
2023-09-15 04:27:17.163031-0700: exit at user request Fri Sep 15 04:27:17 2023
```

file `/var/log/com.apple.xpc.launchd/launchd.log.1`

```text
2023-10-26 16:56:23.171019 <Notice>: Doing boot task: restore-datapartition
2023-10-26 16:56:23.271137 <Notice>: Doing boot task: enable-swap
2023-10-26 16:56:23.287770 <Notice>: swap enabled
```

file `/var/log/asl/logs/aslmanager.20231026T170200+00`

```text
Oct 26 17:02:00: aslmanager starting
Oct 26 17:02:00: Processing data store /var/log/asl
Oct 26 17:02:00: Data Store Size = 325176
```

Did you also notice how the log file _names_ differ in unexpected ways?

#### Microsoft Windows 10

Here are log snippets from a Windows 10 host.

file `${env:SystemRoot}\debug\mrt.log`

```text
Microsoft Windows Malicious Software Removal Tool v5.83, (build 5.83.13532.1)
Started On Thu Sep 10 10:08:35 2020
```

file `${env:SystemRoot}\comsetup.log`

```text
COM+[12:24:34]: ********************************************************************************
COM+[12:24:34]: Setup started - [DATE:05,27,2020 TIME: 12:24 pm]
COM+[12:24:34]: ********************************************************************************
```

file `${env:SystemRoot}\DirectX.log`

```text
11/01/19 20:03:40: infinst: Installed file C:\WINDOWS\system32\xactengine2_1.dll
```

file `${env:SystemRoot}/Microsoft.NET/Framework/v4.0.30319/ngen.log`

```text
09/15/2022 14:13:22.951 [515]: 1>Warning: System.IO.FileNotFoundException: Could not load file or assembly
```

file `${env:SystemRoot}/Performance/WinSAT/winsat.log`

```text
68902359 (21103) - exe\logging.cpp:0841: --- START 2022\5\17 14:26:09 PM ---
68902359 (21103) - exe\main.cpp:4363: WinSAT registry node is created or present
```

(yes, it reads hour `14`, and `PM`…  🙄)

### Summary

This chaotic logging approach is typical of commercial and open-source software,
__*AND IT'S A MESS!*__
Attempting to merge log messages by their natural sort mechanism,
a datetime stamp, is difficult to impossible.

Hence the need for _Super Speedy Syslog Searcher_! 🦸

[logs access attempts in an ad-hoc format]: https://docs.nginx.com/nginx/admin-guide/monitoring/logging/#setting-up-the-access-log

---

## Further Reading

- [`CHANGELOG.md`]
- [`Extended-Thoughts.md`]

[`CHANGELOG.md`]: ./CHANGELOG.md
[`Extended-Thoughts.md`]: ./Extended-Thoughts.md

## Stargazers

[![Stargazers over time](https://starchart.cc/jtmoon79/super-speedy-syslog-searcher.svg?variant=adaptive)](https://starchart.cc/jtmoon79/super-speedy-syslog-searcher)

---

<a href="https://stackexchange.com/users/216253/">
<img src="https://stackexchange.com/users/flair/216253.png" width="208" height="58" alt="profile for @JamesThomasMoon on Stack Exchange, a network of free, community-driven Q&amp;A sites" title="profile for @JamesThomasMoon on Stack Exchange, a network of free, community-driven Q&amp;A sites" />
</a>
