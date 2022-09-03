# Super Speedy Syslog Searcher! (s4)<!-- omit in TOC -->

Speedily search and sort many syslog files by datetime.

_Super Speedy Syslog Searcher_ (s4) can search within compressed syslog files (`.gz`, `.xz`)
and syslog archives (`.tar`, `.zip`). The first goal of s4 is speedy searching.

---

<!-- TOC generated by Markdown All In One -->

- [About](#about)
  - [Features](#features)
  - [Limitations](#limitations)
  - [Hacks](#hacks)
- [Background](#background)
  - [That is a long name!](#that-is-a-long-name)
  - [Learn Rust](#learn-rust)
  - [The Problem](#the-problem)
    - [The Solution](#the-solution)
      - [Solution Difficulties](#solution-difficulties)
        - [_differing sysline formats, differing datetime formats_](#differing-sysline-formats-differing-datetime-formats)
        - [_Which is the correct datetime stamp?_](#which-is-the-correct-datetime-stamp)
        - [_awareness of multi-line syslines adds some layer of program "state"_](#awareness-of-multi-line-syslines-adds-some-layer-of-program-state)
        - [_What year? What timezone?_](#what-year-what-timezone)
        - [_Unicode decoding is a surprisingly expensive operation._](#unicode-decoding-is-a-surprisingly-expensive-operation)
        - [_Binary-search in a file that has varying character encodings._](#binary-search-in-a-file-that-has-varying-character-encodings)
        - [_What defines the boundaries of a sysline?_](#what-defines-the-boundaries-of-a-sysline)

---

## About

### Features

- Prepends datetime and file paths, for easy programmatic parsing or visual traversal of varying syslog messages
- Supports many varying datetime formats including (WHAT IS THE OFFICIAL ONE?)
- Tested against "in the wild" log files from varying Linux distributions (see `./logs/`)
- Comparable speed as GNU `grep` and `sort` (see `./tools/compare-grep-sort1.sh`)
- Handles invalid UTF-8 (prints whatever is found)

### Limitations

- Only handles UTF-8 or ASCII encoded log files.
- Cannot handle multi-file `.gz` files (multiple "streams") (TODO describe problem)
- Cannot handle multi-file `.xz` files (chooses first file found) (TODO describe problem)
- Cannot process archive or compressed files within other archive or compressed files.<br/>
  e.g. a `.tar` file within another `.tar` file will not be processed, a `.gz` file within a `.tar` file will not be processed.

### Hacks

- Entire `.xz` files are read into memory during the initial `open` (see [607a23c00aff0d9b34fb3d678bdfd5c14290582d](https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/607a23c00aff0d9b34fb3d678bdfd5c14290582d#diff-a23d01b527ccc36fa0336ab1789a2f5d2567f21e93c5708b0e5b46ae9f3a708cR783-R836))

## Background

Why _Super Speedy Syslog Searcher_?

### That is a long name!

I prefer unique, searchable, memorable, and self-explanatory names for all programs.

To explain another way, try an internet search for "syslog finder", or "syslog datetime sort", etc.
Many varying results, not a memorable phrase.

### Learn Rust

Great language! Great ecosystem!

Not perfect but a necessary generational improvement over competitors C and C++.

### The Problem

As a Test Engineer, I'm often investigating problems that affect complex systems.
I'm usually only superficially familiar with the underlying subsystems (Test Engineers
are often system generalists, not specialists).  During an investigation, I may
know _when_ a problem occurred but I have little certainty about what system components
might be involved, and so I don't what system logs ("syslogs") are relevant to review.

For example, the frobulator failed around minute _2022/01/01 14:33_.
I know the frobulator emitted an error but my hunch is this error only first _appeared_ in the frobulator but the source error is somewhere else. I know many system components interact with the frobulator but I'm not certain of that knowledge. So I'd like to search _all_ other system components for the preceding five minutes (i.e. search all other syslog files).

I have several-times written scripts that:

1. parse varying datetime formats in lines
2. sort lines by datetime
3. filter lines by some datetime period ("I only want syslogs during this minute")
4. display syslog lines ("syslines") by datetime order and prepended file name

Doing this in a scripting language like Python is do-able but tedious. It's also very slow for
large amounts of syslogs and/or compressed syslogs.
Additional complexities make scripting difficult or imperfect.

- Handling compressed or archived files adds scripting complexity and runtime.<br/>
- Storing all the syslines in memory may be impossible.<br/>
- Dealing with multi-line syslog lines ("syslines") means typical "line by line" processing is inadequate.
  e.g. this is one sysline

      2022/01/01 14:33:12 Frobulator: failed to initialize the woozybucket
           woozybucket: I/O failure /dev/sdc4

I once attempted to create such a script for a multi-host data storage system. The system would
generate tens of Gigabytes of syslogs per day (it was running in Debug mode). Those syslogs
were automatically offloaded onto a busy central NAS that resided in a remote office once per day, where they were further compressed and archive. I would investigate errors days after they occured.
Searching a large amount of syslogs for "everything that happened between _datetime
A_ and _datetime A plus five minutes_" was incredibly slow (tens of minutes to hours).

I observed that if the Searching Tool was datetime-aware, and did a binary-search instead
of a linear search, then searching could be very fast. Additionally, if the tool
would release memory for syslines that have been processed then it could handle arbitrarily
large syslog files.

#### The Solution

Enter _Super Speedy Syslog Searcher_ (s4).

_s4_ searches syslogs by datetime in a binary-search manner. That is, given datetime A,
a syslog (e.g. a file with lines having datetime-stamps; "syslines") a binary-search for the
sysline at or nearest to datetime A is possible. Only small blocks of the file
need to be read to discover the sysline with datetime A (or nearest to datetime A).

Surprisingly, nothing exists that can do this.

At the same time, I was interested in learning about the new cool thing, the _rust_ programming language. _rust_ seemed ideal
for this situation where speed and efficiency was a defining feature of the solution.

##### Solution Difficulties

Why do no other tools exist for this common need?

When moving from implementing a script for a known system, to implementing a
general purpose tool, here are some difficulties I discovered during design and implemention.

###### _differing sysline formats, differing datetime formats_

In practice there are many reasonably possible sysline formats (datetime format, plus arrangment).
Handling them all can be done but adds complexity. Handling them all speedily adds further complexity.

These are a few system log lines ("syslines") I've found:

    2022/01/01 12:34:56 There was an error
    There was an error January 01 2022 12:34:56PM
    [Error] There was an error Sat Jan 1 12:34:56 2022

###### _Which is the correct datetime stamp?_

e.g.

    Frobulator 2022/01/01 12:34:50 there was an error 2022/01/01 12:34:56

How do you tell your parser that the _second_ datetime stamp is the _real_ datetime stamp of the sysline?

###### _awareness of multi-line syslines adds some layer of program "state"_

    2022/01/01 12:34:56 Frobulator: failed to initialize the woozybucket
         woozybucket: I/O failure /dev/sdc4

Line-by-line parsing won't suffice.

###### _What year? What timezone?_

It is common to have syslog datetime formats the do not include a year or a timezone.

For example, your syslog searching script finds these three syslines from three different syslogs,
what is the correct chronological order?

    Jan. 1 11:00:00 Error
    Jan. 1 12:00:00 PST Error
    Jan. 1 13:00:00 UTC Error

What year and timezone did the first sysline occur? (Answer: you don't know).
What year did the second and third sysline occur? (Answer: you don't know).

There are ways to get a "probably correct" answer. But how to do this very quickly (i.e.
without reading the entire file? without spinning up some fancy Learning AI computer?)?
It's not a straightforward solution.

In a multi-host system that spans different timezones (which I've worked with) then this
problem moves from theory to reality.

###### _Unicode decoding is a surprisingly expensive operation._

See `benches/bench_decode_utf.rs`.

###### _Binary-search in a file that has varying character encodings._

What if the "byte cursor" of your binary search lands on the middle byte that is part of three-byte long unicode character?

###### _What defines the boundaries of a sysline?_

A sysline is a line or lines that come after a sysline (or the beginning of a file),
and before a sysline (or the end of a file).

(I hope you are understanding the implied difficulty here).
