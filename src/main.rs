// main.rs
/* …
Successful `sort`. Passes all tests in run-tests including utf-8 with high-order characters.

(export RUST_BACKTRACE=1; cargo run -- --filepath Cargo.toml)
(cargo build && rust-gdb -ex 'layout split' -ex 'b src/main.rs:2062' -ex 'r' --args target/debug/super_speedy_syslog_searcher --filepath ./logs/other/tests/basic-dt.log 2>/dev/null)
(export RUST_BACKTRACE=1; cargo run -- --filepath ./logs/other/tests/test3-hex.log)

# compare performance to `sort`
/usr/bin/time -v -- ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/gen-*.log -- 0x1000 '20000101T000000'
/usr/bin/time -v -- sort -n -- ./logs/other/tests/gen-*.log

(
 # install:
 #   apt install -y linux-perf linux-tools-generic
 #
 # add to Cargo.toml
 #   [profile.bench]
 #   debug = true
 #   [profile.release]
 #   debug = true
 set -eu
 export CARGO_PROFILE_RELEASE_DEBUG=true;
 export PERF=$(realpath /usr/lib/linux-tools/5*-generic/perf)
 set -x;
 cargo build --release
 flamegraph -o flame-S4.svg ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/gen-*.log '20000101T000100'
)

Test this with shell command: run-test.sh

A good library `fselect` for finding files:
https://docs.rs/crate/fselect/0.7.6

This would be fun: flamegraph
https://github.com/flamegraph-rs/flamegraph

Would this be helpful for datetime_searcher(&String)?
https://lib.rs/crates/strcursor

This looks helpful for searching `Vec[u8]` without requiring conversion to `str`.
https://lib.rs/crates/bstr

Slices and references refresher:
    https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=0fe005a84f341848c491a92615288bad

Stack Offset refresher
    https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2d870ad0b835ffc8499f7a16b1c424ec

"Easy Rust" book
https://erasin.wang/books/easy-rust/

The Rust Programing Book
https://doc.rust-lang.org/book/

DROP: TODO: [2021/09/01] what about mmap? https://stackoverflow.com/questions/45972/mmap-vs-reading-blocks

IDEA: [2021/09/17]
      termcolor each different file. Allow user to constrain colors too (in case some colors display poorly on their terminal)
            CLI options like:
               --color={all,none}
               --colors="black,red,green,yellow"
            Good first step.
            Later could allow user to determine colors for particular files.
            Maybe an "advanced" --path option that allows also passing color for the file:
               --path-color=/var/log/syslog:red

LAST WORKING ON [2021/09/05]
    seems to work as a replacement `cat`! :-)
    Add special debug helper function to `BLockReader` and `LineReader` to print
    current known data but in correct file order (not in the order it was accessed): `fn print_known_data`
    Then do similar test but only print some section of the input file. Like first quarter, then middle, then last quarter.
    Consider hiding all these test functions behind a `--test` option. If `--test` is not passed, then just
    behave like `cat`.
    After all that, I think the `SyslogReader` can be started.

LAST WORKING ON [2021/09/09]
    got run-tests.sh to pass!
    Add to `LineReader`
       pub fn print_line(fileoffset)
       pub fn fileoffsets() -> Vec<FileOffset> { [x for x in self.lines.keys()] }
       pub fn print_lines()
       fn scan_lines(blockoffset)  # this will be used for analyzing the first block
                                   # do not use `next_line`, write from scratch
    Then implement `SyslogReader`.

    I JUST REALIZED!!!
    The best way to write this, is to have a cursor for each file.
    For each file {
      find the datetime to start at according to filters (beginning of file if no filter)
      set a FileCursor
    }
    Wait for all FileCursors
    loop {
        comparing all known current FileCursors
        print earliest FileCursor, advance that cursor
    }
    ... which is sort of what I'm doing.... but in actuality, I did not need
    manually worry about Blocks. I could have limited search length
    arbitrarily, and used built-in line-searching algorithms.
    DAMN...
    Though, it's the repetitive file reads that would cause slowness...
    so grabbing big Block chunks then analyzing in memory *is* the right approach.
    The tricky part will be throwing Blocks away as soon as they are done with.
    HMMM...
    A good next thing to implement would be a "print and throw away" that
    can print a Sysline based on offset, then checks if the Sysline and underlying
    Lines and Blocks can be deleted. `print` is already implemented, just need
    the "throw away" function. Would need a way to mark Sysline, Line, Block
    as "ready for garbage collection".

LAST WORKING ON [2021/09/15]
    Finished Sysline and SyslineReader.
    Now what? See TODO about `get_slice`. That may be next best thing.
    After `get_slice`, compare runtime to prior iteration `try7`, compiled as `block_reader_speedy_try7`
    //       Add `fn get_slice(FileOffset) -> (FileOffset, &[u8], FileOffset)`
    //       gives all relevant Line slices of [u8] directly from underlying Block(s),
    //       no copies or new [u8] or anything else.
    //       Passing value 0 returns
    //           (FileOffset of returned slice, first slice, FileOffset of next slice)
    //       call again with "FileOffset of next slice" to get
    //            (FileOffset of returned slice, next slice, FileOffset of next next slice)
    //       Call until "FileOffset of next next slice" is FO_NULL.
    //       Would need to add `Sysline.get_slice` that calls underlying `Line.get_slice`.
    //       This will allow to create a specialized `struct Printer` that calls
    //       `while Sysline.get_slice` (or should it be a `trait Printer`?)
    //       Then remove all `print` stuff from `Line` and `Sysline`.
    --
    Then need to implement a basic but useful `find_datetime`.
    Just have it handle a few easy patterns `^YYYY-MM-DD HH:MM:SS`, etc.
    Then `find_datetime` needs to store the processed value as a formal datetime thingy.
    Ignore TZ for now, but add a TODO for handling TZs.
    Will need to look into the best rust datetime crate, must be comparable, and handle differeing TZ.
    Then much after that, will need to implement binary search for syslines based on datetime range.
    Then ... multi-threaded file processing? This leads into proper stages of program:
    1. analyze first block, is it syslog? what is encoding? 2. if yes, begin printing syslogs

LAST WORKING ON [2021/09/16]
    Now runs about 3% to 5% faster than prior try7-syslinereader.rs implementation.
    About 110% the time of plain `cat` the file.
    Added more stub code to `find_datetime`.
    Added `get_slices`. Different than above idea and simpler to think about.
    Above `get_slice` idea requires `Iterator` Trait and/or closures, but would be very efficient.
    But hold off for now. (might be helpful https://hermanradtke.com/2015/06/22/effectively-using-iterators-in-rust.html)
    Then resume ideas at "LAST WORKING ON 2021/09/15":
    1. `find_datetime` should also transform string to datetime thingy. return (index, index, datetime_thingy)
    2. add a few more hardcoded patterns to `find_datetime` that parse down to H:M:S.f
    3. implement binary search with datetime filtering.
    Item 3. is a big one, and the last step to complete the proof of concept; to answer the question:
    can this run faster than the Unix script version? `cat`, `sort`, `grep`, etc.
    -
    Big milestones, in recommended order:
    - datetime filtering
    - datetime binary search processing
    - multi-threaded processing of multiple files
      - shared task queue of files to process
      - "datetime cursor" leads printing of syslines
      - "throw away" all printed syslines and related resources (heap measurement crate?)
        (definitely read this https://nnethercote.github.io/perf-book/heap-allocations.html)
    - passing directory paths (directory walks)
    - determine if file is syslog file
    - robust datetime matching
    - gz archived single log file
    - xz archived single log file
    - ssh URLs (and accessed)
    - multi-byte encoded files
      - use of `bstr` (is it faster?)
    - tar archived single log file
    - tar archived multiple log file
    - tar.gz archives
    - datetime pattern matching at variable line index

DONE: TODO: [2021/09/16]
      clean up the confusing use Result. Create your own Result Enum that copies what is necessary
      from built-in code.

LAST WORKING ON [2021/09/17]
    Fixing `find_datetime_in_line`, and then store the `DateTime` instances.
    Then need to think about how to use the `DateTime` instances. Maybe a BTreeMap<&DateTime, SyslineP> ?
    I might want to remove `test_find_datetime_in_line` and just use `test_SyslineReader`.

TODO: [2021/09/17]
    If a function does not need `self` then remove it. Simpler, testable.

TODO: [2021/09/20]
      Better distinguish "byte lengths" and "character lengths".
      i.e. rename functions like `len` to `byte_len` or `char_len`.
      or to `size` (bytes) and `len` (characters).
      Also rename various `*Index` to `ByteIndex` or `CharIndex`.
      Also rename various `Offset` to `ByteOffset` or `CharOffset`.

LAST WORKING ON [2021/09/20]
     Tried out flamegraph for fun.
     Now to convert `BlockReader.read_block` to use it's own typed `ResultS4`.
     Then fix the zero size bug, then resume work on function called by `test_SyslineReader_w_filtering`.

FIXED: BUG: [2021/09/20] file of zero size, or single line causes a crash.

LAST WORKING ON [2021/09/22]
     Revised debug indent printing.
     First implement the `fname` macro (search for it) mentioned, then replace debug prints.
     Then resume implementing `find_sysline_at_datetime_filter`. It's only job is to find one sysline
     closest to passed datetime filter and fileoffset. No need to loop on it.

LAST WORKING ON [2021/09/28 02:00:00]
     Just implemented `test_LineReader_1`. Now to resume implementing `test_SyslineReader_w_filtering_3`
     dealing with multiple files.... which I think I'm done with for now.
     Actually want to move on to basic implementation of multi-threaded file reading. No need to print
     in synchrony, just read in different threads, return something (what data can be returned from a finishing thread?).
     Later, work on synchronized printing based on datetime and filters.
     Oh, create a copy of all "TODO" up in this header comment area so I can precede with "DONE"

LAST WORKING ON [2021/10/01 01:10:00]
     Got `test_threading_3` running. It just prints syslog files among shared threads. No coordination.
     Next is to coordinate among the different threads.
     Each file processing threads must:
         processing thread (many):
           find a sysline datetime
               if no datetime found, send Done to `channel_dt`, exit.
           send datetime to channel `channel_dt`
           Wait on recv channel `print`
         coordinating thread (one):
            has map[???, channel_dt]
            waits to recv on several `channel_dt`
              if receive datetime then associates recieved datetime with a send channel
              if receive Done then removes the associated channel
            compares currently had datetimes, for winning datetime (soonest),
               find send channel, send to channel `print`
         processing thread (many):
           receives signal on channel `print`
           prints sysline
           (loop)
     ... carp, this ain't even all of it... there can be many files but only N processing threads.
     So given limited threads but one worker per file, need share a few threads among the many workers,
     like some sort of work pipeline.
     yet need to coordinate among all workers...
     Next implementation step should create one thread per passed file, then implement the datetime printing
     coordination mechanism.
     After that, work on the limited threads mechanism.

LAST WORKING ON [2021/10/02 02:26:00]
     Simplified `test_threading_3` much. One thread per file.
     Need to implement the main thread loop that reads the Sync_Receiver channels, and then
     chooses to print the soonest datetime.

TODO: [2021/10/02]
      need to add a SyslinePrinter that prints `Sysline`.
      A `Sysline` can print itself, however, one little troublesome detail:
      the last `Sysline` of a file often has no terminating '\n'
      When printing `Sysline` from many different files, it'll result in some Syslines
      getting printed on the same row in the CLI. Looks bad, is unexpected.
      To avoid that, the `SyslinePrinter` must be aware of when it is handling the last `Sysline`
      of a file, and write it's own '\n' to stdout.
      Alternative is to append a '\n' to the last Sysline during processing, but I really don't
      like that solution; breaks presumptions that `Sysline` (and underlying `Line`, `Block`)
      hold exactly what was read from the file.

DONE: TODO: [2021/10/02]
      Need to save the parsed datetime, very efficient without it.

TODO: [2021/10/03]
     Offer CLI option to fallback to some TZ. i.e. `--fallback-TZ -0800`
     Use this when TZ cannot be parsed.
     Consider, are there other TZ hints? Like, TZ of the hosting system?
     Or, is it embedded in any file attributes?
     Inform user of default fallback in `--help`.

TODO: [2021/10/03]
     Offer CLI option to fallback to current year. i.e. `--fallback-year 2018`
     For when year cannot be parsed.
     Consider, are there other Year hints?
     The file modified attribute may be very helpful here.
     Could other information be scraped from the file?
     Inform user of default fallback in `--help`.

LAST WORKING ON [2021/10/03 00:20:00]
     Have a simple multi-threaded reader, one thread per file.
     Next, need to improve coordination main thread to allow limited threads per storage source (i.e. x2 for "C:\", x2 for "D:\", etc.)
     May want to move `basic_threading_3` into a it's own proper function.
     I made a skeleton for SyslogWriter, but I'm not sure what to do with it.
     Perhaps that could be the thing that handles all the threading? Rename `SyslogsPrinter` ?

FIXED: BUG: [2021/10/04 01:05:00]
     Fails to parse datetime in datetime from file `logs/Ubuntu18/vmware-installer`, example sysline
         [2019-05-06 11:24:34,033] Installer running.
     Debug output shows an attempt to parse it, all parameters looks correct.
     Not sure what's happening here.
     Is there some odd character not visually obvious in the file or in the pattern? (i.e. a different "hyphen" character?)

TODO: [2021/10/05]
      Some sysline datetime variations not yet possible:
      - variable preceding string; in addition to a datetime pattern with offsets, add an optional
        preceding regexp pattern to try, then match datetime pattern after matched regexp
      - missing year
      - missing TZ (currently, always presumes `Local` TZ but after fixing that to allow for any TZ...)

BUG: [2021/10/06 00:03:00]
     fails for file `basic-basic-dt20.log`:
          for i in 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21; do
            (set -x;
            ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/basic-basic-dt20.log -- 65536 "2000-01-01 00:00:${i}"
            );
            echo; echo;
          done
     Bug appears to be in `find_sysline_at_datetime_filter`.
     As opposed to manually retesting on one file...

TODO: [2022/03/11]
      The concept of static datetime pattern lengths (beg_i, end_i, actual_beg_i, actual_end_i) won't work for
      variable length datetime patterns, i.e. full month names 'July 1, 2020' and 'December 1, 2020'
      See longer note in `find_datetime_in_line`

LAST WORKING ON [2022/03/18]
      Got many tests `test_find_sysline_at_datetime_filter*` Looks pretty solid.
      Now to review all recent "LAST WORKING ON/HERE" embedded in this.
      Then backtrack other recent TODOs.
      IIRC, the next small milestone was getting basic interleaved logs read.
      Running a smorgasborg of log files failed to sort them correctly:
          ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/ *log $(find ./logs/debian9/ ./logs/OpenSUSE15/ ./logs/synology/ ./logs/Ubuntu18/ -type f -not -name '*.gz')
      :-(
      More unfortunately, this bug was only apparent for large numbers of large files.

TODO: [2022/03/18] before opening a file, attempt to retreive it's attributes.
      The Last Modified Time can be used when the datetime format does not include a year.
      e.g. sysline like
          Feb 23 06:37:40 server1 CRON[1242]: pam_unix(cron:session): session closed for user root
      this Last Modified Time could also be used to determine times in a dmesg or Xorg.log file
      e.g. syslines like
          [    91.203] Build Operating System: Linux 4.4.0-170-generic x86_64 Ubuntu

TODO: [2022/03/18] need summary of files not read, or message to user about files that had
      no syslines found. A simple check would be: if !file.is_empty && file.syslines.count()==0 then print warning.
      May want to add an option to print summary of findings after all files.
            --summary
      This might help me debug right now... 

LAST WORKING ON 2022/03/18 17:20:00 basic run is printing every line twice, i.e.
         ./target/debug/super_speedy_syslog_searcher --path ./logs/other/tests/basic-dt5.log
      it appears there is a mixup of return fileoffset value of `find_sysline`.
      When does returned fileoffset mean last char of sysline (inclusive), when does it mean one beyond that last char (exclusive)?
      this needs to be scrutinized for consistency.
      also, `fn find_sysline` may be out of sync with similar `fn find_line`, and should mimic it.
      I should probably add test case for this. Is there not one already? (test_SyslineReader?)

 */

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// uses
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::io::prelude::Read;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
use std::path::Path;
use std::str;
use std::sync::Arc;

extern crate atty;

extern crate backtrace;

extern crate clap;

extern crate chrono;
use chrono::{DateTime, Local, TimeZone};

extern crate crossbeam_channel;

extern crate debug_print;
#[allow(unused_imports)]
use debug_print::{debug_eprint, debug_eprintln, debug_print, debug_println};

extern crate lru;
use lru::LruCache;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate more_asserts;

extern crate rand;

extern crate rangemap;
use rangemap::RangeMap;

extern crate mut_static;

extern crate tempfile;
use tempfile::NamedTempFile;

extern crate termcolor;
use termcolor::{Color, ColorSpec, WriteColor};

use std::sync::Once;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// misc. globals
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// global constants

/// global test initializer to run once
/// see https://stackoverflow.com/a/58006287/471376
static _Test_Init_Once: Once = Once::new();

/// NewLine as char
#[allow(non_upper_case_globals, dead_code)]
static NLc: char = '\n';
/// Single-byte newLine char as u8
#[allow(non_upper_case_globals)]
static NLu8: u8 = 10;
/// Newline in a byte buffer
#[allow(non_upper_case_globals)]
static NLu8a: [u8; 1] = [NLu8];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// custom Results enums for various *Reader functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#481-495

/// `Result` `Ext`ended
/// sometimes things are not `Ok` but a value needs to be returned
#[derive(Debug)]
pub enum ResultS4<T, E> {
    /// Contains the success data
    Found(T),

    /// Contains the success data and reached End Of File and things are okay
    #[allow(non_camel_case_types)]
    Found_EOF(T),

    /// File is empty, or other condition that means "Done", nothing to return, but no bad errors happened
    #[allow(non_camel_case_types)]
    Done,

    /// Contains the error value, something unexpected happened
    Err(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659
// XXX: how to link to specific version of `result.rs`?

impl<T, E> ResultS4<T, E> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Ok`, `Found_EOF`, 'Done`].
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS4::Found(_) | ResultS4::Found_EOF(_) | ResultS4::Done)
    }

    /// Returns `true` if the result is [`Err`].
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Found_EOF`].
    #[inline]
    pub const fn is_eof(&self) -> bool {
        matches!(*self, ResultS4::Found_EOF(_))
    }

    /// Returns `true` if the result is [`Found_EOF`, `Done`].
    #[inline]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS4::Done)
    }

    /// Returns `true` if the result is an [`Ok`, `Found_EOF`] value containing the given value.
    #[must_use]
    #[inline]
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultS4::Found(y) => x == y,
            ResultS4::Found_EOF(y) => x == y,
            ResultS4::Done => false,
            ResultS4::Err(_) => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given value.
    #[must_use]
    #[inline]
    pub fn contains_err<F>(&self, f: &F) -> bool
    where
        F: PartialEq<E>,
    {
        match self {
            ResultS4::Err(e) => f == e,
            _ => false,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for each variant
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `Result<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: Result<u32, &str> = Err("Nothing here");
    /// assert_eq!(x.ok(), None);
    /// ```
    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultS4::Found(x) => Some(x),
            ResultS4::Found_EOF(x) => Some(x),
            ResultS4::Done => None,
            ResultS4::Err(_) => None,
        }
    }

    /// Converts from `Result<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    #[inline]
    pub fn err(self) -> Option<E> {
        match self {
            ResultS4::Found(_) => None,
            ResultS4::Found_EOF(_) => None,
            ResultS4::Done => None,
            ResultS4::Err(x) => Some(x),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - debug printing indentation (stack depths)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type Map_ThreadId_SD<'a> = HashMap<thread::ThreadId, usize>;

// use `stack_offset_set` to set `_STACK_OFFSET_TABLE` once, use `stack_offset` to get
// XXX: no mutex to guard access; it's rarely written to 🤞
// XXX: a mutable static reference for "complex types" is not allowed in rust
//      use `lazy_static` and `mut_static` to create one
//      see https://github.com/tyleo/mut_static#quickstart
lazy_static! {
    static ref _STACK_OFFSET_TABLE: mut_static::MutStatic<Map_ThreadId_SD<'static>> =
        mut_static::MutStatic::new();
        //Map_ThreadId_SD::new();
}

/// return current stack depth according to `backtrace::trace`, including this function
#[allow(dead_code)]
#[inline(always)]
fn stack_depth() -> usize {
    let mut sd: usize = 0;
    backtrace::trace(|_| {
        sd += 1;
        true
    });
    sd
}


/// return current stack offset compared to "original" stack depth. The "original" stack depth
/// should have been recorded at the beginning of the thread by calling `stack_offset_set`.
#[allow(dead_code)]
#[inline(always)]
fn stack_offset() -> usize {
    let mut sd: usize = stack_depth() - 1;
    let sd2 = sd; // XXX: copy `sd` to avoid borrow error
    let tid = thread::current().id();
    let mut so: &usize;
    // XXX: for tests, just set on first call
    if !_STACK_OFFSET_TABLE.is_set().unwrap() {
        _STACK_OFFSET_TABLE.set(Map_ThreadId_SD::new());
    }
    let so_table = _STACK_OFFSET_TABLE.read().unwrap();
    so = so_table.get(&tid).unwrap_or(&sd2);
    if &sd < so {
        return 0;
    }
    sd -= so;
    return sd;
}

/// set once in each thread, preferably near the beginning of the thread
fn stack_offset_set(correction: Option<isize>) {
    let sd_ = stack_depth();
    let sdi: isize = (sd_ as isize) + correction.unwrap_or(0);
    let so = std::cmp::max(sdi, 0) as usize;
    let tid = thread::current().id();
    if !_STACK_OFFSET_TABLE.is_set().unwrap() {
        _STACK_OFFSET_TABLE.set(Map_ThreadId_SD::new());
    }
    assert!(
        !_STACK_OFFSET_TABLE.read().unwrap().contains_key(&tid),
        "_STACK_OFFSET_TABLE has already been set for this thread {:?}; must only be set once",
        tid
    );
    _STACK_OFFSET_TABLE.write().unwrap().insert(tid, so);
    debug_eprintln!("stack_offset_set({:?}): tid {:?} stack_offset set to {}, stack_depth {}", correction, tid, so, sd_);
}

// TODO: currently requires human visual inspection; use `assert!` for proper automated testing.
#[test]
fn test_stack_offset() {
    debug_eprintln!("{}test_stack_offset", sn());
    debug_eprintln!("{}stack_offset {}", so(), stack_offset());
    debug_eprintln!("{}stack_offset() in test_stack_offset {}", so(), stack_offset());
    fn test1a() {
        debug_eprintln!("{}stack_offset() in test_stack_offset in test1a {}", so(), stack_offset());
    }
    test1a();
    fn test1b() {
        debug_eprintln!("{}stack_offset() in test_stack_offset in test1b {}", so(), stack_offset());
        fn test2a() {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2a {}", so(), stack_offset());
        }
        test2a();
        fn test2b(_a: u128, _b: u128, _c: u128) {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2b {}", so(), stack_offset());
        }
        test2b(1, 2, 3);
        fn test2c() {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2c {}", so(), stack_offset());
        }
        test2c();
        test2b(1, 2, 3);
    }
    test1b();
    debug_eprintln!("{}test_stack_offset", sx());
}

/// return a string of spaces as long as `stack_offset`
/// for use in `print` calls, so short function name and not perfect
#[allow(dead_code)]
fn so() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => " ",
        1 => "     ",
        2 => "         ",
        3 => "             ",
        4 => "                 ",
        5 => "                     ",
        6 => "                         ",
        7 => "                             ",
        8 => "                                 ",
        9 => "                                     ",
        10 => "                                        ",
        11 => "                                            ",
        12 => "                                                ",
        13 => "                                                    ",
        14 => "                                                        ",
        15 => "                                                            ",
        16 => "                                                                ",
        17 => "                                                                    ",
        18 => "                                                                        ",
        19 => "                                                                            ",
        _ => "                                                                                ",
    }
}

/// `print` helper, a `s`tring for e`n`tering a function
#[allow(dead_code)]
fn sn() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => "→",
        1 => "    →",
        2 => "        →",
        3 => "            →",
        4 => "                →",
        5 => "                    →",
        6 => "                        →",
        7 => "                            →",
        8 => "                                →",
        9 => "                                    →",
        10 => "                                       →",
        11 => "                                           →",
        12 => "                                               →",
        13 => "                                                   →",
        14 => "                                                       →",
        15 => "                                                           →",
        16 => "                                                               →",
        17 => "                                                                   →",
        18 => "                                                                       →",
        19 => "                                                                           →",
        _ => "                                                                               →",
    }
}

/// `print` helper, a `s`tring for e`x`iting a function
#[allow(dead_code)]
fn sx() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => "←",
        1 => "    ←",
        2 => "        ←",
        3 => "            ←",
        4 => "                ←",
        5 => "                    ←",
        6 => "                        ←",
        7 => "                            ←",
        8 => "                                ←",
        9 => "                                    ←",
        10 => "                                        ←",
        11 => "                                            ←",
        12 => "                                                ←",
        13 => "                                                    ←",
        14 => "                                                        ←",
        15 => "                                                            ←",
        16 => "                                                                ←",
        17 => "                                                                    ←",
        18 => "                                                                        ←",
        19 => "                                                                            ←",
        _ => "                                                                                ←",
    }
}

/// `print` helper, a `s`tring for e`n`tering and e`x`iting a function
/// (like a small function that only needs a one-liner)
#[allow(dead_code)]
fn snx() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => "↔",
        1 => "    ↔",
        2 => "        ↔",
        3 => "            ↔",
        4 => "                ↔",
        5 => "                    ↔",
        6 => "                        ↔",
        7 => "                            ↔",
        8 => "                                ↔",
        9 => "                                    ↔",
        10 => "                                        ↔",
        11 => "                                            ↔",
        12 => "                                                ↔",
        13 => "                                                    ↔",
        14 => "                                                        ↔",
        15 => "                                                            ↔",
        16 => "                                                                ↔",
        17 => "                                                                    ↔",
        18 => "                                                                        ↔",
        19 => "                                                                            ↔",
        _ => "                                                                                ↔",
    }
}

// TODO: [2021/09/22]
//       create new macro for current function name `fname`
//       macro function_name!() prints all parents `A::B::my_func`, just print `my_func`.
//       can be ripped from https://github.com/popzxc/stdext-rs/blob/2179f94475f925a2eacdc2f2408d7ab352d0052c/src/macros.rs#L44-L74
//       could possibly use `backtrace::trace` and return this as part of `so`, `sn`, `sx` ???
/*
fn fno() -> () {
    let bt = backtrace::Backtrace::new();
    let frames = bt.frames();
    dbg!(frames);
    for f in frames.iter() {
        dbg!(f);
        debug_eprintln!("\n");
        for s in f.symbols() {
            dbg!(s);
        }
        debug_eprintln!("\n\n\n");
    }
    frames[1].symbols()[0];
    debug_eprintln!("\n\n\n");
    panic!();
}
*/

/// quickie test for debug helpers `sn`, `so`, `sx`
#[test]
pub fn test_sn_so_sx() {
    fn depth1() {
        debug_eprintln!("{}depth1 enter", sn());
        fn depth2() {
            debug_eprintln!("{}depth2 enter", sn());
            fn depth3() {
                debug_eprintln!("{}depth3 enter", sn());
                fn depth4() {
                    debug_eprintln!("{}depth4 enter", sn());
                    debug_eprintln!("{}depth4 middle", so());
                    debug_eprintln!("{}depth4 exit", sx());
                }
                debug_eprintln!("{}depth3 middle before", so());
                depth4();
                debug_eprintln!("{}depth3 middle after", so());
                debug_eprintln!("{}depth3 exit", sx());
            }
            debug_eprintln!("{}depth2 middle before", so());
            depth3();
            debug_eprintln!("{}depth2 middle after", so());
            debug_eprintln!("{}depth2 exit", sx());
        }
        debug_eprintln!("{}depth1 middle before", so());
        depth2();
        debug_eprintln!("{}depth1 middle after", so());
        debug_eprintln!("{}depth1 exit", sx());
    }
    depth1();
}


/// global test initializer for test functions, useful for complex tests that debug_print using
/// sn(), so(), sx().
/// from https://stackoverflow.com/a/58006287/471376
#[cfg(test)]
pub fn test_init() {
    //_Test_Init_Once.call_once(|| {
        stack_offset_set(Some(-2));
    //});
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - various print and write
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// turn passed u8 into char, for any char values that are CLI formatting instructions transform
/// them to pictoral representations, e.g. '\n' returns a pictoral unicode representation '␊'
/// only intended for debugging
fn char_to_nonraw_char(c: char) -> char {
    if c.is_ascii_graphic() {
        return c;
    }
    // https://www.fileformat.info/info/unicode/block/control_pictures/images.htm
    // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#C0_controls
    let val: u32 = c as u32;
    match val {
        0 => '␀',
        1 => '␁',
        2 => '␂',
        3 => '␃',
        4 => '␄',
        5 => '␅',
        6 => '␆',
        7 => '␇',  // '\a'
        8 => '␈',  // '\b'
        9 => '␉',  // '\t'
        10 => '␊', // '\n'
        11 => '␋', // '\v'
        12 => '␌', // '\f'
        13 => '␍', // '\r'
        14 => '␎',
        15 => '␏',
        16 => '␐',
        17 => '␑',
        18 => '␒',
        19 => '␓',
        20 => '␔',
        21 => '␕',
        22 => '␖',
        23 => '␗',
        24 => '␘',
        25 => '␙',
        26 => '␚',
        27 => '␛', // '\e'
        28 => '␜',
        29 => '␝',
        30 => '␞',
        31 => '␟',
        127 => '␡',
        _ => ' ',
    }
}

/// tranform utf-8 byte (presumably) to non-raw char
/// only intended for debugging
#[allow(dead_code)]
fn byte_to_nonraw_char(byte: u8) -> char {
    return char_to_nonraw_char(byte as char);
}

/// transform buffer of utf-8 chars (presumably) to a non-raw String
/// inefficient
/// only intended for debugging
#[allow(non_snake_case, dead_code)]
fn buffer_to_nonraw_String(buffer: &[u8]) -> String {
    let s1 = match str::from_utf8(buffer) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: buffer_to_nonraw_String: Invalid UTF-8 sequence during from_utf8: {}", err);
            return String::with_capacity(0);
        }
    };
    let mut s2 = String::with_capacity(s1.len() + 10);
    for c in s1.chars() {
        let c_ = char_to_nonraw_char(c);
        s2.push(c_);
    }
    return s2;
}

/// transform str to non-raw String version
/// only intended for debugging
#[allow(non_snake_case, dead_code)]
fn str_to_nonraw_String(str_buf: &str) -> String {
    let mut s2 = String::with_capacity(str_buf.len() + 1);
    for c in str_buf.chars() {
        let c_ = char_to_nonraw_char(c);
        s2.push(c_);
    }
    return s2;
}

/// return contents of file utf-8 chars (presumably) at `path` as non-raw String
/// inefficient
/// only intended for debugging
#[allow(non_snake_case, dead_code)]
fn file_to_nonraw_String(path: &FPath) -> String {
    let path_ = Path::new(path);
    let mut open_options = OpenOptions::new();
    let mut file_ = match open_options.read(true).open(&path_) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: File::open('{:?}') error {}", path_, err);
            return String::with_capacity(0);
        }
    };
    let filesz = match file_.metadata() {
        Ok(val) => val.len() as usize,
        Err(err) => {
            eprintln!("ERROR: File::metadata() error {}", err);
            return String::with_capacity(0);
        }
    };
    let mut s2 = String::with_capacity(filesz + 1);
    let s2read = match file_.read_to_string(&mut s2) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: File::read_to_string() error {}", err);
            return String::with_capacity(0);
        }
    };
    assert_eq!(
        s2read, filesz,
        "Read {} bytes but expected to read file size count of bytes {} for file {:?}",
        s2read, filesz, path
    );
    let mut s3 = String::with_capacity(filesz + 1);
    for c in s2.chars() {
        let c_ = char_to_nonraw_char(c);
        s3.push(c_);
    }
    return s3;
}

/// print colored output to terminal if possible
/// otherwise, print plain output
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
fn print_colored(color: Color, value: &[u8]) -> Result<()> {
    let mut choice: termcolor::ColorChoice = termcolor::ColorChoice::Never;
    if atty::is(atty::Stream::Stdout) {
        choice = termcolor::ColorChoice::Always;
    } else if cfg!(debug_assertions) {
        choice = termcolor::ColorChoice::Always;
    }
    let mut stdout = termcolor::StandardStream::stdout(choice);
    match stdout.set_color(ColorSpec::new().set_fg(Some(color))) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("print_colored: stdout.set_color({:?}) returned error {}", color, err);
            return Err(err);
        }
    };
    //let mut stderr_lock:Option<io::StderrLock> = None;
    //if cfg!(debug_assertions) {
    //    stderr_lock = Some(io::stderr().lock());
   // }
    match stdout.write(value) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("print_colored: stdout.write(…) returned error {}", err);
            return Err(err);
        }
    }
    match stdout.reset() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("print_colored: stdout.reset() returned error {}", err);
            return Err(err);
        }
    }
    stdout.flush()?;
    if cfg!(debug_assertions) {
        //if stderr_lock.is_some() {
        //    stderr_lock.unwrap().flush()?;
        //}
        io::stderr().flush()?;
    }
    Ok(())
}

/// safely write the `buffer` to stdout with help of `StdoutLock`
pub fn write_stdout(buffer: &[u8]) {
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    match stdout_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: StdoutLock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: stdout flushing error {}", err);
        }
    }
    if cfg!(debug_assertions) {
        match io::stderr().flush() {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

/// flush stdout and stderr
pub fn flush_stdouterr() {
    io::stdout().flush();
    io::stderr().flush();
}

/// write to console, `raw` as `true` means "as-is"
/// else use `char_to_nonraw_char` to replace chars in `buffer` (inefficient)
/// only intended for debugging
pub fn pretty_print(buffer: &[u8], raw: bool) {
    if raw {
        return write_stdout(buffer);
    }
    // is this an expensive command? should `stdout` be cached?
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    // XXX: only handle single-byte encodings
    // XXX: doing this char by char is probably not efficient
    //let s = match str::from_utf8_lossy(buffer) {
    let s = match str::from_utf8(&buffer) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: pretty_print: Invalid UTF-8 sequence during from_utf8: {}", err);
            return;
        }
    };
    let mut dst: [u8; 4] = [0, 0, 0, 0];
    for c in s.chars() {
        let c_ = char_to_nonraw_char(c);
        let _cs = c_.encode_utf8(&mut dst);
        match stdout_lock.write(&dst) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: pretty_print: StdoutLock.write({:?}) error {}", &dst, err);
            }
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: pretty_print: stdout flushing error {}", err);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - misc.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// testing helper to write a `str` to a temporary file
/// The temporary file will be automatically deleted when returned `NamedTempFile` is dropped.
#[allow(dead_code)]
fn create_temp_file(content: &str) -> NamedTempFile {
    let mut ntf1 = match NamedTempFile::new() {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf1.write_all(content.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    return ntf1;
}

static COLORS: [Color; 6] = [
    Color::Yellow,
    Color::Green,
    Color::Cyan,
    Color::Red,
    Color::White,
    Color::Magenta,
];

/// "cached" indexing value for `color_rand`
/// not thread aware
#[allow(non_upper_case_globals)]
static mut _color_at: usize = 0;

/// return a random color from `COLORS`
fn color_rand() -> Color {
    let ci: usize;
    unsafe {
        _color_at += 1;
        if _color_at == COLORS.len() {
            _color_at = 0;
        }
        ci = _color_at;
    }
    return COLORS[ci];
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// main
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// `F`ake `Path` or `F`ile `Path`
/// TODO: use `std::path::Path` class
type FPath = String;

pub fn main() -> std::result::Result<(), chrono::format::ParseError> {
    let dt_filter_pattern1: &str = "%Y%m%dT%H%M%S";
    let dt_filter_pattern2: &str = "%Y-%m-%d %H:%M:%S";
    let filter_patterns = [dt_filter_pattern1, dt_filter_pattern2];
    let dt_example: &str = "20200102T123000";

    let matches = clap::App::new("super speedy syslog searcher")
        .version("0.0.2")
        .author("JTM")
        .about("Reads syslog files and prints the each syslog line sorted by datetime. Optional filtering by Datetime.")
        .arg(
        clap::Arg::with_name("paths")
            .short("f")
            .long("path")
            .value_name("FILE")
            .help("Path of file(s) to read")
            .takes_value(true)
            .required(true)
            .multiple(true)
        )
        .arg(
        clap::Arg::with_name("blocksz")
            .short("z")
            .long("blocksz")
            .help("Block Size")
            .required(false)
            .index(1)
            .takes_value(true)
            .default_value("65536")
            .value_name("BLOCKSZ")
        )
        // how to pass args to `group`?
        //.group(
            //clap::ArgGroup::with_name("filters")
                .arg(
                clap::Arg::with_name("dt-after")
                    .help(
                        &format!("DateTime After filter - print syslog lines with a datetime that is at or after this datetime. For example, {:?}", dt_example)
                    )
                    .short("a")
                    .long("dt-after")
                    .required(false)
                    .index(2)
                    .takes_value(true)
                    .default_value("")
                    .value_name("DT_AFTER")
                )
                .arg(
                clap::Arg::with_name("dt-before")
                    .help(
                        &format!("DateTime Before filter - print syslog lines with a datetime that is at or before this datetime. For example, {:?}", dt_example)
                    )
                    .short("b")
                    .long("dt-before")
                    .required(false)
                    .index(3)
                    .takes_value(true)
                    .default_value("")
                    .value_name("DT_BEFORE")
                )
        //)
        .after_help(
            format!("DateTime Filter patterns may be: {:?}", filter_patterns).as_str()
        )
        .get_matches();

    // set once, use `stackdepth_main` to access `_STACKDEPTH_MAIN`
    if cfg!(debug_assertions) {
        stack_offset_set(None);
    }
    debug_eprintln!("{}main()", sn());

    let blockszs = String::from(matches.value_of("blocksz").unwrap());
    let filter_dt_after_s: &str = matches.value_of("dt-after").unwrap();
    let filter_dt_before_s: &str = matches.value_of("dt-before").unwrap();
    let fpaths_str: Vec<_> = matches.values_of("paths").unwrap().collect();
    let fpaths: Vec<String> = fpaths_str.iter().map(|x: &&str| String::from(*x)).collect();

    // parse input number as either hexadecimal or decimal
    let bsize: BlockSz;
    if blockszs.starts_with("0x") {
        bsize = match BlockSz::from_str_radix(&blockszs.trim_start_matches("0x"), 16) {
            Ok(val) => val,
            Err(_e) => 0,
        };
    } else {
        bsize = match blockszs.parse::<BlockSz>() {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Unable to parse a number for --blocksz {:?} {}", blockszs, err);
                std::process::exit(1);
            }
        };
    }

    // parse datetime filters after
    let mut filter_dt_after: DateTimeL_Opt = None;
    //if matches.is_present("dt-after") {
    if !filter_dt_after_s.is_empty() {
        for patt in filter_patterns.iter() {
            debug_eprintln!("{}datetime_from_str({:?}, {:?})", so(), filter_dt_after_s, patt);
            match Local.datetime_from_str(filter_dt_after_s, patt) {
                Ok(val) => {
                    filter_dt_after = Some(val);
                    break;
                }
                Err(_) => {}
            };
        }
        if filter_dt_after.is_none() {
            eprintln!("Unable to parse a datetime for --dt-after {:?}", filter_dt_after_s);
            std::process::exit(1);
        }
    }

    // parse datetime filters before
    let mut filter_dt_before: DateTimeL_Opt = None;
    //if matches.is_present("dt-before") {
    if !filter_dt_before_s.is_empty() {
        for patt in filter_patterns {
            debug_eprintln!("{}datetime_from_str({:?}, {:?})", so(), filter_dt_before_s, patt);
            match Local.datetime_from_str(filter_dt_before_s, patt) {
                Ok(val) => {
                    filter_dt_before = Some(val);
                    break;
                }
                Err(_) => {}
            };
        }
        if filter_dt_before.is_none() {
            eprintln!("Unable to parse a datetime for --dt-before {:?}", filter_dt_before_s);
            std::process::exit(1);
        }
    }

    if filter_dt_after.is_some() && filter_dt_before.is_some() {
        let dta = filter_dt_after.unwrap();
        let dtb = filter_dt_before.unwrap();
        if dta > dtb {
            eprintln!("ERROR: Datetime --dt-after ({}) is after Datetime --dt-before ({})", dta, dtb);
            std::process::exit(1);
        }
    }

    let fpath = fpaths[0].clone();
    //test_sn_so_sx();
    //test_stack_offset();
    //test_BlockReader_offsets();
    //test_BlockReader(&fpath, bsize);
    //test_find_datetime_in_line(bsize);
    //test_LineReader_1();
    //test_LineReader(&fpath, bsize);
    //test_LineReader_rand(&fpath, bsize);
    //test_sysline_pass_filters();
    //test_dt_after_or_before();
    //test_SyslineReader(&fpath, bsize);
    //test_SyslineReader_rand(&fpath, bsize);
    //test_SyslineReader_w_filtering_1(&fpath, bsize, filter_dt_after, filter_dt_before);
    //test_SyslineReader_w_filtering_2(&fpath, bsize, &filter_dt_after, &filter_dt_before);
    //test_SyslineReader_w_filtering_3(&fpaths, bsize, &filter_dt_after, &filter_dt_before);
    //test_threading_1();
    //test_threading_2();
    //test_datetime_soonest1();
    //basic_threading_3(&fpaths, bsize, &filter_dt_after, &filter_dt_before);
    test_threading_4(&fpaths, bsize, &filter_dt_after, &filter_dt_before);

    debug_eprintln!("{}main()", sx());
    return Ok(());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Blocks and BlockReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// type aliases
/// Block Size in bytes
type BlockSz = u64;
/// Byte offset (Index) into a `Block` from beginning of `Block`
type BlockIndex = usize;
/// Offset into a file in `Block`s, depends on `BlockSz` runtime value
type BlockOffset = u64;
/// Offset into a file in bytes
type FileOffset = u64;
/// Block of bytes data read from some file storage
type Block = Vec<u8>;
/// Sequence of Bytes
type Bytes = Vec<u8>;
/// thread-safe Atomic Reference Counting Pointer to a `Block`
type BlockP = Arc<Block>;

type Slices<'a> = Vec<&'a [u8]>;
// Consider this user library which claims to be faster than std::collections::BTreeMap
// https://docs.rs/cranelift-bforest/0.76.0/cranelift_bforest/
type Blocks = BTreeMap<BlockOffset, BlockP>;
type BlocksLRUCache = LruCache<BlockOffset, BlockP>;
/// for case where reading blocks, lines, or syslines reaches end of file, the value `WriteZero` will
/// be used here ot mean "_end of file reached, nothing new_"
/// XXX: this is a hack
#[allow(non_upper_case_globals)]
static EndOfFile: ErrorKind = ErrorKind::WriteZero;

/// Cached file reader that stores data in `BlockSz` byte-sized blocks.
/// A `BlockReader` corresponds to one file.
/// TODO: make a copy of `path`, no need to hold a reference, it just complicates things by introducing explicit lifetimes
pub struct BlockReader<'blockreader> {
    /// Path to file
    pub path: &'blockreader FPath,
    /// File handle, set in `open`
    file: Option<File>,
    /// File.metadata(), set in `open`
    file_metadata: Option<Metadata>,
    /// File size in bytes, set in `open`
    filesz: u64,
    /// File size in blocks, set in `open`
    blockn: u64,
    /// BlockSz used for read operations
    pub blocksz: BlockSz,
    /// cached storage of blocks
    blocks: Blocks,
    /// internal stats tracking
    stats_read_block_cache_lru_hit: u32,
    /// internal stats tracking
    stats_read_block_cache_lru_miss: u32,
    /// internal stats tracking
    stats_read_block_cache_hit: u32,
    /// internal stats tracking
    stats_read_block_cache_miss: u32,
    /// internal LRU cache for `read_block`
    _read_block_lru_cache: BlocksLRUCache,
}

impl fmt::Debug for BlockReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("BlockReader")
            .field("path", &self.path)
            .field("file", &self.file)
            .field("filesz", &self.filesz)
            .field("blockn", &self.blockn)
            .field("blocksz", &self.blocksz)
            .field("blocks cached", &self.blocks.len())
            .field("cache LRU hit", &self.stats_read_block_cache_lru_hit)
            .field("cache LRU miss", &self.stats_read_block_cache_lru_miss)
            .field("cache hit", &self.stats_read_block_cache_hit)
            .field("cache miss", &self.stats_read_block_cache_miss)
            .finish()
    }
}

/// helper for humans debugging Blocks, very inefficient
#[allow(dead_code)]
fn printblock(buffer: &Block, blockoffset: BlockOffset, fileoffset: FileOffset, blocksz: BlockSz, _mesg: String) {
    const LN: usize = 64;
    println!("╔════════════════════════════════════════════════════════════════════════════╕");
    println!(
        "║File block offset {:4}, byte offset {:4}, block length {:4} (0x{:04X}) (max {:4})",
        blockoffset,
        fileoffset,
        buffer.len(),
        buffer.len(),
        blocksz
    );
    println!("║          ┌────────────────────────────────────────────────────────────────┐");
    let mut done = false;
    let mut i = 0;
    let mut buf = Vec::<char>::with_capacity(LN);
    while i < buffer.len() && !done {
        buf.clear();
        for j in 0..LN {
            if i + j >= buffer.len() {
                done = true;
                break;
            };
            // print line number at beginning of line
            if j == 0 {
                let at: usize = i + j + ((blockoffset * blocksz) as usize);
                print!("║@0x{:06x} ", at);
            };
            let v = buffer[i + j];
            let cp = byte_to_nonraw_char(v);
            buf.push(cp);
        }
        // done reading line, print buf
        i += LN;
        {
            //let s_: String = buf.into_iter().collect();
            let s_ = buf.iter().cloned().collect::<String>();
            println!("│{}│", s_);
        }
    }
    println!("╚══════════╧════════════════════════════════════════════════════════════════╛");
}

/// implement the BlockReader things
impl<'blockreader> BlockReader<'blockreader> {
    /// create a new `BlockReader`
    pub fn new(path_: &'blockreader FPath, blocksz: BlockSz) -> BlockReader<'blockreader> {
        // TODO: why not open the file here? change `open` to a "static class wide" (or equivalent)
        //       that does not take a `self`. This would simplify some things about `BlockReader`
        // TODO: how to make some fields `blockn` `blocksz` `filesz` immutable?
        //       https://stackoverflow.com/questions/23743566/how-can-i-force-a-structs-field-to-always-be-immutable-in-rust
        assert_ne!(0, blocksz, "Block Size cannot be 0");
        return BlockReader {
            path: path_,
            file: None,
            file_metadata: None,
            filesz: 0,
            blockn: 0,
            blocksz,
            blocks: Blocks::new(),
            stats_read_block_cache_lru_hit: 0,
            stats_read_block_cache_lru_miss: 0,
            stats_read_block_cache_hit: 0,
            stats_read_block_cache_miss: 0,
            _read_block_lru_cache: BlocksLRUCache::new(4),
        };
    }

    // TODO: make a `self` version of the following helpers that does not require
    //       passing `BlockSz`. Save the user some trouble.
    //       Can also `assert` that passed `FileOffset` is not larger than filesz, greater than zero.
    //       But keep the public static version available for testing.
    //       Change the LineReader calls to call `self.blockreader....`

    /// return preceding block offset at given file byte offset
    pub fn block_offset_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockOffset {
        return (file_offset / blocksz) as BlockOffset;
    }

    /// return file_offset (byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(block_offset: BlockOffset, blocksz: BlockSz) -> FileOffset {
        return (block_offset * blocksz) as BlockOffset;
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(
        blockoffset: BlockOffset, blocksz: BlockSz, blockindex: BlockIndex,
    ) -> FileOffset {
        assert_lt!(
            (blockindex as BlockSz),
            blocksz,
            "BlockIndex {} should not be greater or equal to BlockSz {}",
            blockindex,
            blocksz
        );
        BlockReader::file_offset_at_block_offset(blockoffset, blocksz) + (blockindex as FileOffset)
    }

    /// return block_index (byte offset into a `Block`) for `Block` that corresponds to `FileOffset`
    pub fn block_index_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockIndex {
        return (file_offset
            - BlockReader::file_offset_at_block_offset(
                BlockReader::block_offset_at_file_offset(file_offset, blocksz),
                blocksz,
            )) as BlockIndex;
    }

    /// return count of blocks in a file
    pub fn file_blocks_count(filesz: FileOffset, blocksz: BlockSz) -> u64 {
        return (filesz / blocksz + (if filesz % blocksz > 0 { 1 } else { 0 })) as u64;
    }

    /// return last valid BlockOffset
    pub fn blockoffset_last(&self) -> BlockOffset {
        if self.filesz == 0 {
            return 0;
        }
        (BlockReader::file_blocks_count(self.filesz, self.blocksz) as BlockOffset) - 1
    }

    /// open the `self.path` file, set other field values after opening.
    /// propagates any `Err`, success returns `Ok(())`
    pub fn open(&mut self) -> Result<()> {
        assert!(self.file.is_none(), "ERROR: the file is already open");
        let mut open_options = OpenOptions::new();
        match open_options.read(true).open(&self.path) {
            Ok(val) => self.file = Some(val),
            Err(err) => {
                eprintln!("ERROR: File::open('{:?}') error {}", &self.path, err);
                return Err(err);
            }
        };
        let file_ = self.file.as_ref().unwrap();
        match file_.metadata() {
            Ok(val) => {
                self.filesz = val.len();
                self.file_metadata = Some(val);
            }
            Err(err) => {
                eprintln!("ERROR: File::metadata() error {}", err);
                return Err(err);
            }
        };
        self.blockn = BlockReader::file_blocks_count(self.filesz, self.blocksz);
        self.blocks = Blocks::new();
        Ok(())
    }

    /// read a `Block` of data of max size `self.blocksz` from a prior `open`ed data source
    /// when successfully read returns `Ok(BlockP)`
    /// when reached the end of the file, and no data was read returns `Err(EndOfFile)`
    /// all other `File` and `std::io` errors are propagated to the caller
    /// TODO: create custom `ResultS4` for this too, get rid of hack `EndOfFile`
    pub fn read_block(&mut self, blockoffset: BlockOffset) -> Result<BlockP> {
        debug_eprintln!("{}read_block: @{:p}.read_block({})", sn(), self, blockoffset);
        assert!(self.file.is_some(), "File has not been opened '{:?}'", self.path);
        // check LRU cache
        match self._read_block_lru_cache.get(&blockoffset) {
            Some(bp) => {
                self.stats_read_block_cache_lru_hit += 1;
                debug_eprintln!(
                    "{}read_block: return Ok(BlockP@{:p}); hit LRU cache Block[{}] len {}",
                    sx(),
                    &*bp,
                    &blockoffset,
                    (*bp).len()
                );
                return Ok(bp.clone());
            }
            None => {
                debug_eprintln!("{}read_block: blockoffset {} not found LRU cache", so(), blockoffset);
                self.stats_read_block_cache_lru_miss += 1;
            }
        }
        // check hash map cache
        if self.blocks.contains_key(&blockoffset) {
            debug_eprintln!("{}read_block: blocks.contains_key({})", so(), blockoffset);
            self.stats_read_block_cache_hit += 1;
            let bp: &BlockP = &self.blocks[&blockoffset];
            debug_eprintln!("{}read_block: LRU cache put({}, BlockP@{:p})", so(), blockoffset, bp);
            self._read_block_lru_cache.put(blockoffset, bp.clone());
            debug_eprintln!(
                "{}read_block: return Ok(BlockP@{:p}); cached Block[{}] len {}",
                sx(),
                &*self.blocks[&blockoffset],
                &blockoffset,
                self.blocks[&blockoffset].len()
            );
            return Ok(bp.clone());
        }
        self.stats_read_block_cache_miss += 1;
        let seek = (self.blocksz * blockoffset) as u64;
        let mut file_ = self.file.as_ref().unwrap();
        match file_.seek(SeekFrom::Start(seek)) {
            Ok(_) => {}
            Err(err) => {
                debug_eprintln!("{}read_block: return Err({})", sx(), err);
                eprintln!("ERROR: file.SeekFrom({}) Error {}", seek, err);
                return Err(err);
            }
        };
        let mut reader = file_.take(self.blocksz as u64);
        // here is where the `Block` is created then set with data.
        // It should never change after this. Is there a way to mark it as "frozen"?
        // I guess just never use `mut`.
        // XXX: currently does not handle a partial read. From the docs (https://doc.rust-lang.org/std/io/trait.Read.html#method.read_to_end)
        //      > If any other read error is encountered then this function immediately returns. Any
        //      > bytes which have already been read will be appended to buf.
        //
        let mut buffer = Block::with_capacity(self.blocksz as usize);
        debug_eprintln!("{}read_block: reader.read_to_end(@{:p})", so(), &buffer);
        match reader.read_to_end(&mut buffer) {
            Ok(val) => {
                if val == 0 {
                    // special case of `Err` that caller should handle
                    debug_eprintln!(
                        "{}read_block: return Err(EndOfFile) EndOfFile blockoffset {} {:?}",
                        sx(),
                        blockoffset,
                        self.path
                    );
                    return Err(Error::new(EndOfFile, "End Of File"));
                }
            }
            Err(err) => {
                eprintln!("ERROR: reader.read_to_end(buffer) error {}", err);
                debug_eprintln!("{}read_block: return Err({})", sx(), err);
                return Err(err);
            }
        };
        let bp = BlockP::new(buffer);
        // store block
        debug_eprintln!("{}read_block: blocks.insert({}, BlockP@{:p})", so(), blockoffset, bp);
        self.blocks.insert(blockoffset, bp.clone());
        // store in LRU cache
        debug_eprintln!("{}read_block: LRU cache put({}, BlockP@{:p})", so(), blockoffset, bp);
        self._read_block_lru_cache.put(blockoffset, bp.clone());
        debug_eprintln!(
            "{}read_block: return Ok(BlockP@{:p}); new Block[{}] len {}",
            sx(),
            &*self.blocks[&blockoffset],
            &blockoffset,
            (*self.blocks[&blockoffset]).len()
        );
        Ok(bp)
    }

    /// get byte at FileOffset
    /// `None` means the data at `FileOffset` was not available
    /// Does not request any `read_block`! Only copies from what is currently available from prior
    /// calls to `read_block`.
    /// debug helper only
    fn _get_byte(&self, fo: FileOffset) -> Option<u8> {
        let bo = BlockReader::block_offset_at_file_offset(fo, self.blocksz);
        let bi = BlockReader::block_index_at_file_offset(fo, self.blocksz);
        if self.blocks.contains_key(&bo) {
            return Some((*self.blocks[&bo])[bi]);
        }
        return None;
    }

    /// return `Bytes` at `[fo_a, fo_b)`.
    /// uses `self._get_byte` which does not request any reads!
    /// debug helper only
    fn _vec_from(&self, fo_a: FileOffset, fo_b: FileOffset) -> Bytes {
        assert_le!(fo_a, fo_b, "bad fo_a {} fo_b {}", fo_a, fo_b);
        assert_le!(fo_b, self.filesz, "bad fo_b {} but filesz {}", fo_b, self.filesz);
        if fo_a == fo_b {
            return Bytes::with_capacity(0);
        }
        let bo_a = BlockReader::block_offset_at_file_offset(fo_a, self.blocksz);
        let bo_b = BlockReader::block_offset_at_file_offset(fo_b, self.blocksz);
        let bo_a_i = BlockReader::block_index_at_file_offset(fo_a, self.blocksz);
        let bo_b_i = BlockReader::block_index_at_file_offset(fo_b, self.blocksz);
        if bo_a == bo_b {
            return Bytes::from(&(*self.blocks[&bo_a])[bo_a_i..bo_b_i]);
        }
        let mut fo_at = fo_a;
        let sz = (fo_b - fo_a) as usize;
        // XXX: inefficient!
        let mut vec_ = Bytes::with_capacity(sz);
        while fo_at < fo_b {
            let b = match self._get_byte(fo_at) {
                Some(val) => val,
                None => {
                    break;
                }
            };
            vec_.push(b);
            fo_at += 1;
        }
        return vec_;
    }
}

#[test]
fn test_BlockReader1() {
    test_BlockReader(&FPath::from("./logs/other/tests/basic-basic-dt10-repeats.log"), 2);
}

/// basic test of BlockReader things
#[allow(non_snake_case, dead_code)]
#[cfg(test)]
fn test_BlockReader(path_: &FPath, blocksz: BlockSz) {
    debug_println!("test_BlockReader()");

    // testing BlockReader basics

    let mut br1 = BlockReader::new(&path_, blocksz);
    debug_println!("new {:?}", &br1);
    match br1.open() {
        Ok(_) => {
            debug_eprintln!("opened {:?}", path_);
        }
        Err(err) => {
            eprintln!("ERROR: BlockReader.open('{:?}') {}", path_, err);
            return;
        }
    }
    debug_println!("opened {:?}", &br1);
    let last_blk = BlockReader::block_offset_at_file_offset(br1.filesz, blocksz);
    for offset in [0, 1, 5, 1, 99, 1, last_blk].iter() {
        {
            let rbp = br1.read_block(*offset);
            match rbp {
                Ok(val) => {
                    let boff: FileOffset = BlockReader::file_offset_at_block_offset(*offset, blocksz);
                    printblock(val.as_ref(), *offset, boff, blocksz, format!(""));
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        continue;
                    } else {
                        eprintln!("ERROR: blockreader.read({}) error {}", offset, err);
                    }
                }
            };
        }
    }
    debug_println!("after reads {:?}", &br1);
}

/// quick self-test
#[allow(dead_code)]
#[test]
fn test_file_blocks_count() {
    debug_eprintln!("test_file_blocks_count()");
    assert_eq!(1, BlockReader::file_blocks_count(1, 1));
    assert_eq!(2, BlockReader::file_blocks_count(2, 1));
    assert_eq!(3, BlockReader::file_blocks_count(3, 1));
    assert_eq!(4, BlockReader::file_blocks_count(4, 1));
    assert_eq!(1, BlockReader::file_blocks_count(1, 2));
    assert_eq!(1, BlockReader::file_blocks_count(2, 2));
    assert_eq!(2, BlockReader::file_blocks_count(3, 2));
    assert_eq!(2, BlockReader::file_blocks_count(4, 2));
    assert_eq!(3, BlockReader::file_blocks_count(5, 2));
    assert_eq!(1, BlockReader::file_blocks_count(1, 3));
    assert_eq!(1, BlockReader::file_blocks_count(2, 3));
    assert_eq!(1, BlockReader::file_blocks_count(3, 3));
    assert_eq!(2, BlockReader::file_blocks_count(4, 3));
    assert_eq!(1, BlockReader::file_blocks_count(1, 4));
    assert_eq!(1, BlockReader::file_blocks_count(4, 4));
    assert_eq!(2, BlockReader::file_blocks_count(5, 4));
    assert_eq!(1, BlockReader::file_blocks_count(4, 5));
    assert_eq!(1, BlockReader::file_blocks_count(5, 5));
    assert_eq!(2, BlockReader::file_blocks_count(6, 5));
    assert_eq!(2, BlockReader::file_blocks_count(10, 5));
    assert_eq!(3, BlockReader::file_blocks_count(11, 5));
    assert_eq!(3, BlockReader::file_blocks_count(15, 5));
    assert_eq!(4, BlockReader::file_blocks_count(16, 5));
}

/// quick self-test
#[allow(dead_code)]
#[test]
fn test_file_offset_at_block_offset() {
    debug_eprintln!("test_file_offset_at_block_offset()");
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 1));
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 2));
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 4));
    assert_eq!(1, BlockReader::file_offset_at_block_offset(1, 1));
    assert_eq!(2, BlockReader::file_offset_at_block_offset(1, 2));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(1, 4));
    assert_eq!(2, BlockReader::file_offset_at_block_offset(2, 1));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(2, 2));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(2, 4));
    assert_eq!(3, BlockReader::file_offset_at_block_offset(3, 1));
    assert_eq!(6, BlockReader::file_offset_at_block_offset(3, 2));
    assert_eq!(12, BlockReader::file_offset_at_block_offset(3, 4));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(4, 1));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(4, 2));
    assert_eq!(16, BlockReader::file_offset_at_block_offset(4, 4));
    assert_eq!(5, BlockReader::file_offset_at_block_offset(5, 1));
    assert_eq!(10, BlockReader::file_offset_at_block_offset(5, 2));
    assert_eq!(20, BlockReader::file_offset_at_block_offset(5, 4));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(8, 1));
    assert_eq!(16, BlockReader::file_offset_at_block_offset(8, 2));
    assert_eq!(32, BlockReader::file_offset_at_block_offset(8, 4));
}

/// quick self-test
#[allow(dead_code)]
#[test]
fn test_block_offset_at_file_offset() {
    debug_eprintln!("test_block_offset_at_file_offset()");
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 1));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(1, 1));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(2, 1));
    assert_eq!(3, BlockReader::block_offset_at_file_offset(3, 1));
    assert_eq!(4, BlockReader::block_offset_at_file_offset(4, 1));
    assert_eq!(5, BlockReader::block_offset_at_file_offset(5, 1));
    assert_eq!(8, BlockReader::block_offset_at_file_offset(8, 1));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 2));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 2));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(2, 2));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(3, 2));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(4, 2));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(5, 2));
    assert_eq!(4, BlockReader::block_offset_at_file_offset(8, 2));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(2, 3));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(3, 3));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(4, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(6, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(7, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(8, 3));
    assert_eq!(3, BlockReader::block_offset_at_file_offset(9, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(2, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(3, 4));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(4, 4));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(5, 4));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(8, 4));
}

/// quick self-test
#[allow(dead_code)]
#[test]
fn test_block_index_at_file_offset() {
    debug_eprintln!("test_block_index_at_file_offset()");
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(1, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(2, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(3, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 2));
    assert_eq!(1, BlockReader::block_index_at_file_offset(1, 2));
    assert_eq!(0, BlockReader::block_index_at_file_offset(2, 2));
    assert_eq!(1, BlockReader::block_index_at_file_offset(3, 2));
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(1, 3));
    assert_eq!(2, BlockReader::block_index_at_file_offset(2, 3));
    assert_eq!(0, BlockReader::block_index_at_file_offset(3, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(4, 3));
    assert_eq!(2, BlockReader::block_index_at_file_offset(5, 3));
    assert_eq!(0, BlockReader::block_index_at_file_offset(6, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(7, 3));
}

/// quick self-test
#[allow(dead_code)]
#[test]
fn test_file_offset_at_block_offset_index() {
    debug_eprintln!("test_file_offset_at_block_offset_index()");
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 1, 0));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(1, 1, 0));
    assert_eq!(2, BlockReader::file_offset_at_block_offset_index(2, 1, 0));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(3, 1, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(4, 1, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 2, 0));
    assert_eq!(2, BlockReader::file_offset_at_block_offset_index(1, 2, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(2, 2, 0));
    assert_eq!(6, BlockReader::file_offset_at_block_offset_index(3, 2, 0));
    assert_eq!(8, BlockReader::file_offset_at_block_offset_index(4, 2, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 3, 0));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(1, 3, 0));
    assert_eq!(6, BlockReader::file_offset_at_block_offset_index(2, 3, 0));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(3, 3, 0));
    assert_eq!(12, BlockReader::file_offset_at_block_offset_index(4, 3, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 4, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(1, 4, 0));
    assert_eq!(8, BlockReader::file_offset_at_block_offset_index(2, 4, 0));
    assert_eq!(12, BlockReader::file_offset_at_block_offset_index(3, 4, 0));
    assert_eq!(16, BlockReader::file_offset_at_block_offset_index(4, 4, 0));

    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 2, 1));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(1, 2, 1));
    assert_eq!(5, BlockReader::file_offset_at_block_offset_index(2, 2, 1));
    assert_eq!(7, BlockReader::file_offset_at_block_offset_index(3, 2, 1));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(4, 2, 1));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 3, 1));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(1, 3, 1));
    assert_eq!(7, BlockReader::file_offset_at_block_offset_index(2, 3, 1));
    assert_eq!(10, BlockReader::file_offset_at_block_offset_index(3, 3, 1));
    assert_eq!(13, BlockReader::file_offset_at_block_offset_index(4, 3, 1));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 4, 1));
    assert_eq!(5, BlockReader::file_offset_at_block_offset_index(1, 4, 1));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(2, 4, 1));
    assert_eq!(13, BlockReader::file_offset_at_block_offset_index(3, 4, 1));
    assert_eq!(17, BlockReader::file_offset_at_block_offset_index(4, 4, 1));
}

#[allow(non_snake_case, dead_code)]
#[cfg(test)]
fn test_BlockReader_offsets() {
    test_file_blocks_count();
    test_file_offset_at_block_offset();
    test_block_offset_at_file_offset();
    test_block_index_at_file_offset();
    test_file_offset_at_block_offset_index();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart, Line, and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Struct describing a part or all of a line within a `Block`
/// A "line" can span more than one `Block`. This tracks part or all of a line within
/// one `Block`. One `LinePart` to one `Block`.
/// But one or more `LinePart` are necessary to represent an entire "line".
pub struct LinePart {
    /// index into the `blockp`, index at beginning
    pub blocki_beg: BlockIndex,
    /// index into the `blockp`, index at one after ending '\n' (may refer to one past end of `Block`)
    pub blocki_end: BlockIndex,
    /// the `Block` pointer
    pub blockp: BlockP,
    /// the byte offset into the file where this `LinePart` begins
    pub fileoffset: FileOffset,
    /// debug helper, might be good to get rid of this?
    pub blockoffset: BlockOffset,
    /// debug helper, might be good to get rid of this?
    pub blocksz: BlockSz,
    // TODO: add size of *this* block
}

impl fmt::Debug for LinePart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LinePart")
            .field("LinePart @", &format_args!("{:p}", &self))
            .field("blocki_beg", &self.blocki_beg)
            .field("blocki_end", &self.blocki_end)
            .field("len", &self.len())
            .field("blockp @", &format_args!("{:p}", &(*self.blockp)))
            .field("fileoffset", &self.fileoffset)
            .field("blockoffset", &self.blockoffset)
            .finish()
    }
}

impl LinePart {
    pub fn new(
        blocki_beg: BlockIndex, blocki_end: BlockIndex, blockp: BlockP, fileoffset: FileOffset,
        blockoffset: BlockOffset, blocksz: BlockSz,
    ) -> LinePart {
        debug_eprintln!(
            "{}LinePart::new(blocki_beg {}, blocki_end {}, Block @{:p}, fileoffset {}, blockoffset {}, blocksz {})",
            so(),
            blocki_beg,
            blocki_end,
            &*blockp,
            fileoffset,
            blockoffset,
            blocksz
        );
        // some sanity checks
        assert_ne!(fileoffset, FileOffset::MAX, "Bad fileoffset MAX");
        assert_ne!(blockoffset, BlockOffset::MAX, "Bad blockoffset MAX");
        let fo1 = BlockReader::file_offset_at_block_offset(blockoffset, blocksz);
        assert_le!(fo1, fileoffset, "Bad FileOffset {}, must ≥ {}", fileoffset, fo1);
        let fo2 = BlockReader::file_offset_at_block_offset(blockoffset + 1, blocksz);
        assert_le!(fileoffset, fo2, "Bad FileOffset {}, must ≤ {}", fileoffset, fo2);
        let bo = BlockReader::block_offset_at_file_offset(fileoffset, blocksz);
        assert_eq!(blockoffset, bo, "Bad BlockOffset {}, expected {}", blockoffset, bo);
        let bi = BlockReader::block_index_at_file_offset(fileoffset, blocksz);
        assert_eq!(
            blocki_beg, bi,
            "blocki_beg {} ≠ {} block_index_at_file_offset({}, {})",
            blocki_beg, bi, fileoffset, blocksz
        );
        assert_ne!(blocki_end, 0, "Bad blocki_end 0, expected > 0");
        assert_lt!(blocki_beg, blocki_end, "blocki_beg {} should be < blocki_end {}", blocki_beg, blocki_end);
        assert_lt!((blocki_beg as BlockSz), blocksz, "blocki_beg {} should be < blocksz {}", blocki_beg, blocksz);
        assert_le!((blocki_end as BlockSz), blocksz, "blocki_end {} should be ≤ blocksz {}", blocki_end, blocksz);
        LinePart {
            blocki_beg,
            blocki_end,
            blockp,
            fileoffset,
            blockoffset,
            blocksz,
        }
    }

    /// length of line starting at index `blocki_beg`
    pub fn len(&self) -> usize {
        (self.blocki_end - self.blocki_beg) as usize
    }

    pub fn is_empty(&self) -> bool {
        return self.len() == 0;
    }

    // TODO: [2022/03/19] add function to return some kind of pointer to underlying
    //       block bytes, that also iterates
    //       this would be used to write underlying block bytes to console.
    //       is this alrady implemented elsewhere?
}

/// A sequence to track a `Line`.
/// A "line" may span multiple `Block`s. One `LinePart` is needed for each `Block`.
type LineParts = Vec<LinePart>;

/// A `Line` has information about a "line" that may or may not span more than one `Block`
pub struct Line {
    lineparts: LineParts,
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for li in self.lineparts.iter() {
            li_s.push_str(&format!(
                " @{:p} (blocki_beg {}, blocki_end {}, len() {}, BlockP.len() {}, fileoffset {}, blockoffset {})",
                &li,
                &li.blocki_beg,
                &li.blocki_end,
                &li.len(),
                &li.blockp.len(),
                &li.fileoffset,
                &li.blockoffset
            ));
        }
        let mut fo_b = 0;
        if !self.lineparts.is_empty() {
            fo_b = self.lineparts[0].fileoffset;
        }
        let mut fo_e = 0;
        if !self.lineparts.is_empty() {
            let last_li = self.lineparts.len() - 1;
            fo_e = self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1;
        }
        f.debug_struct("Line")
            .field("line.fileoffset_begin()", &fo_b)
            .field("line.fileoffset_end()", &fo_e)
            .field("lineparts @", &format_args!("{:p}", &self))
            .field("lineparts.len", &self.lineparts.len())
            .field("lineparts", &li_s)
            .finish()
    }
}

impl Line {
    /// default `with_capacity` for a `LineParts`, most often will only need 1 capacity
    /// as the found "line" will likely reside within one `Block`
    const LINE_PARTS_WITH_CAPACITY: usize = 1;

    pub fn new() -> Line {
        return Line {
            lineparts: LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY),
        };
    }

    pub fn new_from_linepart(info: LinePart) -> Line {
        let mut v = LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY);
        v.push(info);
        return Line { lineparts: v };
    }

    pub fn push(&mut self, linepart: LinePart) {
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li = &self.lineparts[l_ - 1];
            assert_le!(
                li.blockoffset,
                linepart.blockoffset,
                "Prior stored LinePart at blockoffset {} is after passed LinePart at blockoffset {}",
                li.blockoffset,
                linepart.blockoffset,
            );
            assert_lt!(
                li.fileoffset,
                linepart.fileoffset,
                "Prior stored LinePart at fileoffset {} is at or after passed LinePart at fileoffset {}",
                li.fileoffset,
                linepart.fileoffset,
            );
        }
        // TODO: add sanity checks of all prior `linepart` that all `blocki_end` match `*blockp.len()`
        self.lineparts.push(linepart);
    }

    /// the byte offset into the file where this `Line` begins
    /// "points" to first character of `Line`
    pub fn fileoffset_begin(self: &Line) -> FileOffset {
        assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        self.lineparts[0].fileoffset
    }

    /// the byte offset into the file where this `Line` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Line) -> FileOffset {
        assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        let last_li = self.lineparts.len() - 1;
        self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1
    }

    /// XXX: is this correct?
    pub fn len(self: &Line) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `LinePart` in `self.lineparts.len()`
    pub fn count(self: &Line) -> usize {
        self.lineparts.len()
    }

    /// return all slices that make up this `Line`
    pub fn get_slices(self: &Line) -> Slices {
        // short-circuit this case
        let sz = self.lineparts.len();
        let mut slices = Slices::with_capacity(sz);
        for linepart in self.lineparts.iter() {
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            slices.push(slice);
        }
        return slices;
    }

    /// return a count of slices that would be returned by `get_slices`
    pub fn get_slices_count(self: &Line) -> usize {
        return self.lineparts.len();
    }

    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each bute to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    pub fn print(self: &Line, raw: bool) {
        // is this an expensive command? should `stdout` be cached?
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for linepart in &self.lineparts {
            // TODO: I'm somewhat sure this is not creating anything new but I should verify with `gdb-rust`.
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            if raw {
                match stdout_lock.write(slice) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!(
                            "ERROR: StdoutLock.write(@{:p}[{}‥{}]) error {}",
                            &*linepart.blockp, linepart.blocki_beg, linepart.blocki_end, err
                        );
                    }
                }
            } else {
                // XXX: only handle single-byte encodings
                // XXX: this is not efficient
                //let s = match str::from_utf8_lossy(slice) {
                let s = match str::from_utf8(slice) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("ERROR: Invalid UTF-8 sequence during from_utf8_lossy: {:?}", err);
                        continue;
                    }
                };
                let mut dst: [u8; 4] = [0, 0, 0, 0];
                for c in s.chars() {
                    let c_ = char_to_nonraw_char(c);
                    let _cs = c_.encode_utf8(&mut dst);
                    match stdout_lock.write(&dst) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("ERROR: StdoutLock.write({:?}) error {}", &dst, err);
                        }
                    }
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: stdout flushing error {}", err);
            }
        }
    }

    /// create `String` from known bytes referenced by `self.lineparts`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_nonraw_char`)
    /// XXX: not efficient!
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    fn _to_String_raw(self: &Line, raw: bool) -> String {
        let mut sz: usize = 0;
        for linepart in &self.lineparts {
            sz += linepart.len();
        }
        let mut s1 = String::with_capacity(sz);

        for linepart in &self.lineparts {
            if raw {
                // transform slices to `str`, can this be done more efficiently?
                // XXX: here is a good place to use `bstr`
                let s2 = &(&*linepart.blockp)[linepart.blocki_beg..linepart.blocki_end];
                let s3 = match str::from_utf8(s2) {
                    Ok(val) => val,
                    Err(err) => {
                        let fo1 = self.fileoffset_begin() + (linepart.blocki_beg as FileOffset);
                        let fo2 = self.fileoffset_begin() + (linepart.blocki_end as FileOffset);
                        eprintln!("ERROR: failed to convert [u8] at FileOffset[{}‥{}] to utf8 str; {}", fo1, fo2, err);
                        continue;
                    }
                };
                s1.push_str(s3);
            } else {
                // copy u8 as char to `s1`
                let stop = linepart.len();
                let block_iter = (&*linepart.blockp).iter();
                for (bi, b) in block_iter.skip(linepart.blocki_beg).enumerate() {
                    if bi >= stop {
                        break;
                    }
                    let c = byte_to_nonraw_char(*b);
                    s1.push(c);
                }
            }
        }
        return s1;
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    pub fn to_String(self: &Line) -> String {
        return self._to_String_raw(true);
    }

    #[allow(non_snake_case)]
    pub fn to_String_from(self: &Line, _from: usize) -> String {
        unimplemented!("to_String_from");
    }

    #[allow(non_snake_case)]
    pub fn to_String_from_to(self: &Line, _from: usize, _to: usize) -> String {
        unimplemented!("to_String_from_to");
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    pub fn to_String_noraw(self: &Line) -> String {
        return self._to_String_raw(false);
    }

    /// slice that represents the entire `Line`
    /// if `Line` does not cross a Block then this returns slice into the `Block`,
    /// otherwise it requires a copy of `Block`s data
    /// TODO: should use `&[char]`?
    /// XXX: cannot return slice because 1. size not known at compile time so cannot
    ///      place on stack 2. slice is an array which is not an "owned type"
    pub fn as_slice(self: &Line) -> Bytes {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            let bi_beg = self.lineparts[0].blocki_beg;
            let bi_end = self.lineparts[0].blocki_end;
            assert_eq!(bi_end - bi_beg, self.len(), "bi_end-bi_beg != line.len()");
            return Bytes::from(&(*(self.lineparts[0].blockp))[bi_beg..bi_end]);
        }
        // not efficient case, Line crosses stored Blocks so have to create a new array
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data = Bytes::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_slice: data.len() != self.len()");
        return data;
    }
}

type CharSz = usize;
/// thread-safe Atomic Reference Counting pointer to a `Line`
type LineP = Arc<Line>;
/// storage for Lines found from the underlying `BlockReader`
/// FileOffset key is the first byte/offset that begins the `Line`
type FoToLine = BTreeMap<FileOffset, LineP>;
/// Line Searching error
#[allow(non_camel_case_types)]
type ResultS4_LineFind = ResultS4<(FileOffset, LineP), Error>;
type LinesLRUCache = LruCache<FileOffset, ResultS4_LineFind>;
/// range map where key is Line begin to end `[Line.fileoffset_begin(), Line.fileoffset_end()]`
/// and where value is Line begin (`Line.fileoffset_begin()`). Use the value to lookup associated `Line` map
type LinesRangeMap = RangeMap<FileOffset, FileOffset>;

/// Specialized Reader that uses BlockReader to find FoToLine
pub struct LineReader<'linereader> {
    blockreader: BlockReader<'linereader>,
    /// track `Line` found among blocks in `blockreader`, tracked by line beginning `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_begin()`
    pub lines: FoToLine,
    /// track `Line` found among blocks in `blockreader`, tracked by line ending `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_end()`
    lines_end: FoToLine,
    /// char size in bytes
    /// TODO: handle char sizes > 1 byte
    /// TODO: handle multi-byte encodings
    _charsz: CharSz,
    /// `Line` offsets stored as Range `[fileoffset_begin..fileoffset_end+1)`. to `fileoffset_begin`.
    ///  the stored value can be used to lookup `Line` in `self.lines`
    lines_by_range: LinesRangeMap,
    /// internal LRU cache for `find_line`
    _find_line_lru_cache: LinesLRUCache,
    // TODO: [2021/09/21] add efficiency stats
}

impl fmt::Debug for LineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("LineReader")
            //.field("@", format!("{:p}", &self))
            .field("blockreader", &self.blockreader)
            .field("_charsz", &self._charsz)
            .field("lines", &self.lines)
            .finish()
    }
}

// XXX: cannot place these within `impl LineReader`?
/// minimum char storage size in bytes
static CHARSZ_MIN: CharSz = 1;
/// maximum char storage size in bytes
static CHARSZ_MAX: CharSz = 4;
/// default char storage size in bytes
/// XXX: does not handle multi-byte encodings (e.g. UTF-8) or multi-byte character storage (e.g. UTF-32)
static CHARSZ: CharSz = CHARSZ_MIN;

/// implement the LineReader things
impl<'linereader> LineReader<'linereader> {
    pub fn new(path: &'linereader FPath, blocksz: BlockSz) -> Result<LineReader<'linereader>> {
        // XXX: multi-byte
        assert_ge!(
            blocksz,
            (CHARSZ_MIN as BlockSz),
            "BlockSz {} is too small, must be greater than or equal {}",
            blocksz,
            CHARSZ_MAX
        );
        assert_ne!(blocksz, 0, "BlockSz is zero");
        let mut br = BlockReader::new(path, blocksz);
        match br.open() {
            Err(err) => {
                return Err(err);
            }
            Ok(_) => {}
        };
        Ok(LineReader {
            blockreader: br,
            lines: FoToLine::new(),
            lines_end: FoToLine::new(),
            _charsz: CHARSZ,
            // give impossible value to start with
            //_next_line_blockoffset: FileOffset::MAX,
            //_next_line_blockp_opt: None,
            //_next_line_blocki: 0,
            lines_by_range: LinesRangeMap::new(),
            _find_line_lru_cache: LinesLRUCache::new(8),
        })
    }

    /// smallest size character in bytes
    pub fn charsz(&self) -> usize {
        self._charsz
    }

    pub fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz
    }

    pub fn filesz(&self) -> BlockSz {
        self.blockreader.filesz
    }

    pub fn path(&self) -> &str {
        return self.blockreader.path.as_str();
    }

    /// print `Line` at `fileoffset`
    /// return `false` if `fileoffset` not found
    pub fn print(&self, fileoffset: &FileOffset) -> bool {
        if !self.lines.contains_key(fileoffset) {
            return false;
        }
        let lp = &self.lines[fileoffset];
        lp.print(true);
        return true;
    }

    /// Testing helper only
    /// print all known `Line`s
    pub fn print_all(&self) {
        for fo in self.lines.keys() {
            self.print(fo);
        }
    }

    /// count of lines held by this LineReader
    pub fn count(&self) -> usize {
        self.lines.len()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        BlockReader::file_blocks_count(self.filesz(), self.blocksz())
    }

    pub fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader.blockoffset_last()
    }

    /// find next `Line` starting from `fileoffset`
    /// in the process of finding, creates and stores the `Line` from underlying `Block` data
    /// returns `Found`(`FileOffset` of beginning of the _next_ line, found `LineP`)
    /// reaching end of file (and no new line) returns `Found_EOF`
    /// reaching end of file returns `FileOffset` value that is one byte past the actual end of file (and should not be used)
    /// otherwise `Err`, all other `Result::Err` errors are propagated
    /// 
    /// similar to `find_sysline`, `read_block`
    ///
    /// XXX: presumes single-byte to one '\n', does not handle UTF-16 or UTF-32 or other (`charsz` hardcoded to 1)
    /// TODO: [2021/08/30] handle different encodings
    /// XXX: this function is fragile and cumbersome, any tweaks require extensive retesting
    pub fn find_line(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);


        // some helpful constants
        let charsz_fo = self._charsz as FileOffset;
        let charsz_bi = self._charsz as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();

        // check LRU cache
        match self._find_line_lru_cache.get(&fileoffset) {
            Some(rlp) => {
                // self.stats_read_block_cache_lru_hit += 1;
                debug_eprint!("{}find_line: found LRU cached for offset {}", sx(), fileoffset);
                match rlp {
                    ResultS4_LineFind::Found(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found(({}, …)) @[{}, {}]", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_LineFind::Found((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Found_EOF(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found_EOF(({}, …)) @[{}, {}]", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_LineFind::Found_EOF((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Done => {
                        debug_eprintln!(" return ResultS4_LineFind::Done");
                        return ResultS4_LineFind::Done;
                    }
                    _ => {
                        debug_eprintln!(" Err");
                        eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {}", fileoffset);
                    }
                }
            }
            None => {
                //self.stats_read_block_cache_lru_miss += 1;
                debug_eprintln!("{}find_line: fileoffset {} not found in LRU cache", so(), fileoffset);
            }
        }

        // handle special cases
        if filesz == 0 {
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; file is empty", sx());
            return ResultS4_LineFind::Done;
        } else if fileoffset > filesz {
            // TODO: need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            let err = Error::new(
                ErrorKind::AddrNotAvailable,
                format!("Passed fileoffset {} past file size {}", fileoffset, filesz),
            );
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Err({}); fileoffset was too big!", sx(), err);
            return ResultS4_LineFind::Err(err);
        } else if fileoffset == filesz {
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done(); fileoffset is at end of file!", sx());
            return ResultS4_LineFind::Done;
        }

        match self.lines_by_range.get(&fileoffset) {
            Some(fo_range) => {
                debug_eprintln!(
                    "{}find_line: fileoffset {} refers to self.lines_by_range Range {:?}",
                    so(),
                    fileoffset,
                    fo_range
                );
                let lp = self.lines[fo_range].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                // TODO: add stats like BlockReader._stats*
                debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_next);
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p}) @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                return ResultS4_LineFind::Found((fo_next, lp));
            }
            None => {
                //self.stats_read_block_cache_lru_miss += 1;
                debug_eprintln!("{}find_line: fileoffset {} not found in rangemap", so(), fileoffset);
            }
        }

        // first check if there is a line already known at this fileoffset
        if self.lines.contains_key(&fileoffset) {
            debug_eprintln!("{}find_line: hit cache for FileOffset {}", so(), fileoffset);
            debug_eprintln!(
                "{}find_line: XXX: IS IT AN ERROR GETTING HERE BUT NOT FINDING IN self.lines_by_range????",
                so()
            );
            let lp = self.lines[&fileoffset].clone();
            let fo_next = (*lp).fileoffset_end() + charsz_fo;
            // TODO: add stats like BlockReader._stats*
            debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_next);
            self._find_line_lru_cache
                .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p})  @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
            return ResultS4_LineFind::Found((fo_next, lp));
        }
        debug_eprintln!("{}find_line: fileoffset {} not found in self.lines", so(), fileoffset);
        debug_eprintln!("{}find_line: searching for first newline newline A …", so());

        //
        // walk through blocks and bytes looking for beginning of a line (a newline character; part A)
        //

        // block pointer to the current block of interest
        let mut bp: BlockP;
        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // should point to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = 0;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            debug_eprintln!("{}find_line: newline A is {} because at beginning of file!", so(), fo_nl_a);
        }
        // if prior char at fileoffset-1 has newline then use that
        // caller's commonly call this function `find_line` in a sequence so it's an easy check
        // with likely success
        if !found_nl_a {
            // XXX: single-byte encoding
            let fo1 = fileoffset - charsz_fo;
            if self.lines_end.contains_key(&fo1) {
                found_nl_a = true;
                debug_eprintln!(
                    "{}find_line: found newline A {} from lookup of passed fileoffset-1 {}",
                    so(),
                    fo1,
                    fileoffset - 1
                );
                // `fo_nl_a` should refer to first char past newline A
                // XXX: single-byte encoding
                fo_nl_a = fo1 + charsz_fo;
            }
        }

        let mut eof = false;
        let mut bo = self.block_offset_at_file_offset(fileoffset);
        let mut bin_beg_init_a = self.block_index_at_file_offset(fileoffset);
        while !found_nl_a && bo <= blockoffset_last {
            debug_eprintln!("{}find_line: self.blockreader.read_block({})", so(), bo);
            match self.blockreader.read_block(bo) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_line: read_block returned Block @{:p} len {} while searching for newline A",
                        so(),
                        &(*val),
                        (*val).len()
                    );
                    bp = val;
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        debug_eprintln!("{}find_line: read_block returned EndOfFile {:?} searching for found_nl_a failed (IS THIS AN ERROR???????)", so(), self.path());
                        // reached end of file, no beginning newlines found
                        // TODO: Is this an error state? should this be handled differently?
                        debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; EOF from read_block; NOT SURE IF THIS IS CORRECT", sx());
                        return ResultS4_LineFind::Done;
                    }
                    debug_eprintln!("{}find_line: LRU cache put({}, Done)", so(), fileoffset);
                    self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; NOT SURE IF THIS IS CORRECT!!!!", sx());
                    return ResultS4_LineFind::Done;
                }
            }
            let blen = (*bp).len() as BlockIndex;
            let mut bin_beg = bin_beg_init_a;
            while bin_beg < blen {
                // XXX: single-byte encoding
                if (*bp)[bin_beg] == NLu8 {
                    found_nl_a = true;
                    fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                    debug_eprintln!(
                        "{}find_line: found newline A from byte search at fileoffset {} ≟ blockoffset {} blockindex {}",
                        so(),
                        fo_nl_a,
                        bo,
                        bin_beg
                    );
                    // `fo_nl_a` should refer to first char past newline A
                    // XXX: single-byte encoding
                    fo_nl_a += charsz_fo;
                    break;
                }
                // XXX: single-byte encoding
                bin_beg += charsz_bi;
            }
            if found_nl_a {
                break;
            }
            bin_beg_init_a = 0;
            bo += 1;
            if bo > blockoffset_last {
                debug_eprintln!("{}find_line: EOF blockoffset {} > {} blockoffset_last", so(), bo, blockoffset_last);
                eof = true;
                break;
            }
            if fo_nl_a >= filesz {
                debug_eprintln!("{}find_line: EOF newline A fileoffset {} > {} file size", so(), fo_nl_a, filesz);
                eof = true;
                break;
            }
        } // ! found_nl_a

        assert_lt!(fo_nl_a, filesz + 1, "ERROR: newline A {} is past end of file {}", fo_nl_a, filesz + 1);
        if eof {
            debug_eprintln!("{}find_line: LRU Cache put({}, Done)", so(), fileoffset);
            self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
            // the last character in the file is a newline
            // XXX: is this correct?
            debug_eprintln!(
                "{}find_line: return ResultS4_LineFind::Done; newline A is at last char in file {}, not a line IS THIS CORRECT?",
                sx(),
                filesz - 1
            );
            return ResultS4_LineFind::Done;
        }

        //
        // walk through blocks and bytes looking for ending of line (a newline character; part B)
        //
        debug_eprintln!(
            "{}find_line: found first newline A, searching for second B newline starting at {} …",
            so(),
            fo_nl_a
        );

        // found newline part B? Line ends at this
        let mut found_nl_b: bool = false;
        // this is effectively the cursor that is being analyzed
        let mut fo_nl_b: FileOffset = fo_nl_a;
        // set for the first loop (first block), then is zero
        let mut bin_beg_init_b: BlockIndex = self.block_index_at_file_offset(fo_nl_b);
        // append LinePart to this `Line`
        let mut line: Line = Line::new();
        bo = self.block_offset_at_file_offset(fo_nl_b);
        while !found_nl_b && bo <= blockoffset_last {
            debug_eprintln!("{}find_line: self.blockreader.read_block({})", so(), bo);
            match self.blockreader.read_block(bo) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_line: read_block returned Block @{:p} len {} while searching for newline B",
                        so(),
                        &(*val),
                        (*val).len()
                    );
                    bp = val;
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        debug_eprintln!(
                            "{}find_line: read_block returned EndOfFile {:?} while searching for newline B",
                            so(),
                            self.path()
                        );
                        let rl = self.insert_line(line);
                        let fo_ = (*rl).fileoffset_end() + charsz_fo;
                        debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_);
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_, rl.clone())));
                        debug_eprintln!(
                            "{}find_line: return ResultS4_LineFind::Found_EOF(({}, {:p})) @[{} , {}]; {:?}",
                            sx(),
                            fo_,
                            &*rl,
                            (*rl).fileoffset_begin(),
                            (*rl).fileoffset_end(),
                            (*rl).to_String_noraw()
                        );
                        return ResultS4_LineFind::Found_EOF((fo_, rl));
                    }
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Err({:?});", sx(), err);
                    return ResultS4_LineFind::Err(err);
                }
            }
            let blen = (*bp).len() as BlockIndex;
            let bin_beg = bin_beg_init_b;
            let mut bin_end = bin_beg;
            while bin_end < blen {
                // XXX: single-byte encoding
                if (*bp)[bin_end] == NLu8 {
                    found_nl_b = true;
                    fo_nl_b = self.file_offset_at_block_offset_index(bo, bin_end);
                    bin_end += charsz_bi; // refer to one past end
                    debug_eprintln!(
                        "{}find_line: newline B found by byte search fileoffset {} ≟ blockoffset {} blockindex {}",
                        so(),
                        fo_nl_b,
                        bo,
                        bin_end
                    );
                    break;
                }
                // XXX: single-byte encoding
                bin_end += charsz_bi;
            }
            let fo_beg = self.file_offset_at_block_offset_index(bo, bin_beg);
            // sanity check
            if fo_beg == filesz {
                assert_eq!(bin_end - bin_beg, 0, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
            }
            // sanity check
            if bin_end - bin_beg == 0 {
                assert_eq!(fo_beg, filesz, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
            }
            // at end of file, "zero length" LinePart, skip creating a `LinePart`
            if bin_end - bin_beg == 0 && fo_beg == filesz {
                debug_eprintln!("{}find_line: no newline B, at end of file", so());
                break;
            }
            let li = LinePart::new(bin_beg, bin_end, bp.clone(), fo_beg, bo, self.blocksz());
            debug_eprintln!("{}find_line: Line.push({:?})", so(), &li);
            line.push(li);
            if found_nl_b {
                break;
            }
            bin_beg_init_b = 0;
            bo += 1;
            if bo > blockoffset_last {
                break;
            }
        } // ! found_nl_b

        // may occur in files ending on a single newline
        if line.count() == 0 {
            debug_eprintln!("{}find_line: LRU Cache put({}, Done)", so(), fileoffset);
            self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done;", sx());
            return ResultS4_LineFind::Done;
        }

        // sanity check
        debug_eprintln!("{}find_line: return {:?};", so(), line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        //assert_eq!(fo_beg, fo_nl_a, "line.fileoffset_begin() {} ≠ {} searched fo_nl_a", fo_beg, fo_nl_a);
        //assert_eq!(fo_end, fo_nl_b, "line.fileoffset_end() {} ≠ {} searched fo_nl_b", fo_end, fo_nl_b);
        if fo_beg != fo_nl_a {
            debug_eprintln!("WARNING: line.fileoffset_begin() {} ≠ {} searched fo_nl_a", fo_beg, fo_nl_a);
        }
        if fo_end != fo_nl_b {
            debug_eprintln!("WARNING: line.fileoffset_end() {} ≠ {} searched fo_nl_b", fo_end, fo_nl_b);
        }
        assert_lt!(fo_end, filesz, "line.fileoffset_end() {} is past file size {}", fo_end, filesz);

        let rl = self.insert_line(line);
        debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_end + 1);
        self._find_line_lru_cache
            .put(fileoffset, ResultS4_LineFind::Found((fo_end + 1, rl.clone())));
        debug_eprintln!(
            "{}find_line: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}]; {:?}",
            sx(),
            fo_end + 1,
            &*rl,
            (*rl).fileoffset_begin(),
            (*rl).fileoffset_end(),
            (*rl).to_String_noraw()
        );
        return ResultS4_LineFind::Found((fo_end + 1, rl));
    }

    fn insert_line(&mut self, line: Line) -> LineP {
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let rl = LineP::new(line);
        debug_eprintln!("{}LineReader.insert_line: lines.insert({}, Line @{:p})", so(), fo_beg, &(*rl));
        self.lines.insert(fo_beg, rl.clone());
        debug_eprintln!("{}LineReader.insert_line: lines_end.insert({}, Line @{:p})", so(), fo_end, &(*rl));
        self.lines_end.insert(fo_end, rl.clone());
        // XXX: multi-byte character encoding
        let fo_end1 = fo_end + (self.charsz() as FileOffset);
        debug_eprintln!("{}LineReader.insert_line: lines_by_range.insert({}‥{}, {})", so(), fo_beg, fo_end1, fo_beg);
        self.lines_by_range.insert(fo_beg..fo_end1, fo_beg);
        return rl;
    }
}

/// loop on `LineReader.find_line` until it is done
/// prints to stdout
/// testing helper
#[allow(dead_code)]
fn process_LineReader(lr1: &mut LineReader) {
    debug_eprintln!("{}process_LineReader()", sn());
    let mut fo1: FileOffset = 0;
    loop {
        debug_eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = lr1.count();
                debug_eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                fo1 = fo;
                if cfg!(debug_assertions) {
                    match print_colored(Color::Green, &(*lp).as_slice()) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("ERROR: print_colored returned error {}", err);
                        }
                    }
                } else {
                    (*lp).print(true);
                }
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = lr1.count();
                debug_eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                fo1 = fo;
                (*lp).print(true);
            }
            ResultS4_LineFind::Done => {
                debug_eprintln!("{}ResultS4_LineFind::Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                debug_eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }
    debug_eprintln!("{}process_LineReader()", sx());
}

/// basic test of LineReader things with premade tests
/// simple read of file offsets in order, should print to stdout an identical file
#[allow(non_snake_case, dead_code)]
#[test]
fn test_LineReader_1() {
    debug_eprintln!("{}test_LineReader_1()", sn());

    for (content, line_count) in [
        ("", 0),
        (" ", 1),
        ("  ", 1),
        (" \n", 1),
        (" \n ", 2),
        ("  \n  ", 2),
        (" \n \n", 2),
        ("  \n  \n", 2),
        (" \n \n ", 3),
        ("  \n  \n  ", 3),
        ("  \n  \n  \n", 3),
        ("  \n  \n  \n  ", 4),
        ("  \n  \n  \n  \n", 4),
        ("two unicode points é\n  \n  \n  \n", 4),
    ] {
        let ntf = create_temp_file(content);
        let blocksz: BlockSz = 64;
        let path = String::from(ntf.path().to_str().unwrap());
        let mut lr1 = match LineReader::new(&path, blocksz) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("ERROR: LineReader::new({:?}, {}) failed {}", path, blocksz, err);
                return;
            }
        };
        let bufnonraw = buffer_to_nonraw_String(content.as_bytes());
        debug_eprintln!("{}File {:?}", so(), bufnonraw);
        process_LineReader(&mut lr1);
        let lc = lr1.count();
        assert_eq!(line_count, lc, "Expected {} count of lines, found {}", line_count, lc);
        #[allow(unused_must_use)]
        print_colored(
            Color::Green,
            format!("{}PASS Found {} Lines as expected from {:?}\n", so(), lc, bufnonraw).as_bytes(),
        );
        debug_eprintln!("{}{:?}", so(), content.as_bytes());
    }
    debug_eprintln!("{}test_LineReader_1()", sx());
}

/// basic test of LineReader things using user passed file
/// simple read of file offsets in order, should print to stdout an identical file
#[allow(non_snake_case, dead_code)]
#[cfg(test)]
fn test_LineReader(path_: &FPath, blocksz: BlockSz) {
    debug_eprintln!("{}test_LineReader({:?}, {})", sn(), &path_, blocksz);
    let mut lr1 = match LineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: LineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}LineReader {:?}", so(), lr1);

    process_LineReader(&mut lr1);
    //debug_eprintln!("\n{}{:?}", so(), lr1);

    if cfg!(debug_assertions) {
        debug_eprintln!("{}Found {} Lines", so(), lr1.count())
    }
    debug_eprintln!("{}test_LineReader({:?}, {})", sx(), &path_, blocksz);
}

/// testing helper
fn randomize(v_: &mut Vec<FileOffset>) {
    // XXX: can also use `rand::shuffle` https://docs.rs/rand/0.8.4/rand/seq/trait.SliceRandom.html#tymethod.shuffle
    let sz = v_.len();
    let mut i = 0;
    while i < sz {
        let r = rand::random::<usize>() % sz;
        v_.swap(r, i);
        i += 1;
    }
}

/// testing helper
fn fill(v_: &mut Vec<FileOffset>) {
    let sz = v_.capacity();
    let mut i = 0;
    while i < sz {
        v_.push(i as FileOffset);
        i += 1;
    }
}

/// basic test of LineReader things using user passed file
/// read all file offsets but randomly
#[allow(non_snake_case, dead_code)]
#[cfg(test)]
fn test_LineReader_rand(path_: &FPath, blocksz: BlockSz) {
    debug_eprintln!("{}test_LineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let mut lr1 = match LineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: LineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}LineReader {:?}", so(), lr1);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(lr1.filesz() as usize);
    fill(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    for fo1 in offsets_rand {
        debug_eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = lr1.count();
                debug_eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                //fo1 = fo;
                //(*lp).print(true);
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = lr1.count();
                debug_eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                //fo1 = fo;
                //(*lp).print(true);
            }
            ResultS4_LineFind::Done => {
                debug_eprintln!("{}ResultS4_LineFind::Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                debug_eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }
    // should print the file as-is and not be affected by random reads
    lr1.print_all();
    debug_eprintln!("\n{}{:?}", so(), lr1);
    debug_eprintln!("{}test_LineReader_rand({:?}, {})", sx(), &path_, blocksz);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sysline and SyslogReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A sequence to track one or more `LineP` that make up a `Sysline` 
type Lines = Vec<LineP>;
/// An offset into a `Line`
type LineIndex = usize;
/// typical DateTime with TZ type
type DateTimeL = DateTime<Local>;
#[allow(non_camel_case_types)]
type DateTimeL_Opt = Option<DateTimeL>;
/// Sysline Searching error
/// TODO: does SyslineFind need an `Found_EOF` state? Is it an unnecessary overlap of `Ok` and `Done`?
#[allow(non_camel_case_types)]
type ResultS4_SyslineFind = ResultS4<(FileOffset, SyslineP), Error>;

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s
/// a "syslog line" is one or more lines, where the first line starts with a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslines.
pub struct Sysline {
    /// the one or more `Line` that make up a Sysline
    lines: Lines,
    /// index into `Line` where datetime string starts
    /// byte-based count
    /// datetime is presumed to be on first Line
    dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    /// byte-based count
    /// datetime is presumed to be on first Line
    dt_end: LineIndex,
    /// parsed DateTime instance
    /// TODO: assumes `Local` TZ, how to create an "any TZ" chrono DateTime instance?
    dt: DateTimeL_Opt,
}

/// a signifier value for "not set" or "null" - because sometimes Option is a PitA
const LI_NULL: LineIndex = LineIndex::MAX;

impl fmt::Debug for Sysline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.lines.iter() {
            li_s.push_str(&format!(
                "Line @{:p} (fileoffset_beg {}, fileoffset_end {}, len() {}, count() {}",
                &*lp,
                (*lp).fileoffset_begin(),
                (*lp).fileoffset_end(),
                (*lp).len(),
                (*lp).count()
            ));
        }
        f.debug_struct("Sysline")
            .field("fileoffset_begin()", &self.fileoffset_begin())
            .field("fileoffset_end()", &self.fileoffset_end())
            .field("lines @", &format_args!("{:p}", &self.lines))
            .field("lines.len", &self.lines.len())
            .field("dt_beg", &self.dt_beg)
            .field("dt_end", &self.dt_end)
            .field("dt", &self.dt)
            .field("lines", &li_s)
            .finish()
    }
}

impl Sysline {
    /// default `with_capacity` for a `Lines`, most often will only need 1 capacity
    /// as the found "sysline" will likely be one `Line`
    const SYSLINE_PARTS_WITH_CAPACITY: usize = 1;
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new() -> Sysline {
        return Sysline {
            lines: Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY),
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        };
    }

    pub fn new_from_line(linep: LineP) -> Sysline {
        let mut v = Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY);
        v.push(linep);
        return Sysline {
            lines: v,
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        };
    }

    pub fn charsz(self: &Sysline) -> usize {
        Sysline::CHARSZ
    }

    pub fn push(&mut self, linep: LineP) {
        if !self.lines.is_empty() {
            // TODO: sanity check lines are in sequence
        }
        debug_eprintln!(
            "{}SyslineReader.push(@{:p}), self.lines.len() is now {}",
            so(),
            &*linep,
            self.lines.len() + 1
        );
        self.lines.push(linep);
    }

    /// the byte offset into the file where this `Sysline` begins
    /// "points" to first character of `Sysline` (and underlying `Line`)
    pub fn fileoffset_begin(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        (*self.lines[0]).fileoffset_begin()
    }

    /// the byte offset into the file where this `Sysline` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        let last_ = self.lines.len() - 1;
        (*self.lines[last_]).fileoffset_end()
    }

    /// the byte offset into the next sysline
    /// however, this Sysline does not know if it is at the end of a file
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

    /// length in bytes of the Sysline
    pub fn len(self: &Sysline) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `Line` in `self.lines`
    pub fn count(self: &Sysline) -> usize {
        self.lines.len()
    }

    /// print approach #2, print by slices
    /// prints raw data from underlying `Block`
    /// testing helper
    fn print2(&self) {
        let slices = self.get_slices();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for slice in slices.iter() {
            match stdout_lock.write(slice) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: write: StdoutLock.write(slice@{:p} (len {})) error {}", slice, slice.len(), err);
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: write: stdout flushing error {}", err);
            }
        }
    }

    /// a `String` copy of the demarcating datetime string found in the Sysline
    /// TODO: does not handle datetime spanning multiple lines... is that even possible? No...
    #[allow(non_snake_case)]
    pub fn datetime_String(self: &Sysline) -> String {
        assert_ne!(self.dt_beg, LI_NULL, "dt_beg has not been set");
        assert_ne!(self.dt_end, LI_NULL, "dt_end has not been set");
        assert_lt!(self.dt_beg, self.dt_end, "bad values dt_end {} dt_beg {}", self.dt_end, self.dt_beg);
        let slice_ = self.lines[0].as_slice();
        assert_lt!(
            self.dt_beg,
            slice_.len(),
            "dt_beg {} past end of slice[{}‥{}]?",
            self.dt_beg,
            self.dt_beg,
            self.dt_end
        );
        assert_le!(
            self.dt_end,
            slice_.len(),
            "dt_end {} past end of slice[{}‥{}]?",
            self.dt_end,
            self.dt_beg,
            self.dt_end
        );
        // TODO: here is a place to use `bstr`
        let buf: &[u8] = &slice_[self.dt_beg..self.dt_end];
        let s_ = match str::from_utf8(buf) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Error in datetime_String() during str::from_utf8 {} buffer {:?}", err, buf);
                ""
            }
        };
        String::from(s_)
    }

    /// return all the slices that make up this `Sysline`
    pub fn get_slices(self: &Sysline) -> Slices {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += lp.get_slices_count();
        }
        let mut slices = Slices::with_capacity(sz);
        for lp in &self.lines {
            slices.extend(lp.get_slices().iter());
        }
        return slices;
    }

    /// print approach #1, use underlying `Line` to `print`
    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    pub fn print1(self: &Sysline, raw: bool) {
        for lp in &self.lines {
            (*lp).print(raw);
        }
    }

    /// create `String` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_nonraw_char`)
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    fn _to_String_raw(self: &Sysline, raw: bool) -> String {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += (*lp).len();
        }
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let mut s_ = String::with_capacity(sz + 1);
        for lp in &self.lines {
            s_ += (*lp)._to_String_raw(raw).as_str();
        }
        return s_;
    }

    /*
    /// create `str` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_nonraw_char`)
    /// TODO: can this be more efficient? specialized for `str`?
    #[allow(non_snake_case)]
    fn _to_str_raw(self: &Sysline, raw: bool) -> &str {
        return (&self._to_String_raw(raw)).as_str();
    }
     */

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    pub fn to_String(self: &Sysline) -> String {
        return self._to_String_raw(true);
    }

    #[allow(non_snake_case)]
    pub fn to_String_from(self: &Sysline, _from: usize) -> String {
        unimplemented!("yep");
    }

    #[allow(non_snake_case)]
    pub fn to_String_from_to(self: &Sysline, _from: usize, _to: usize) -> String {
        unimplemented!("yep");
    }

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    pub fn to_String_noraw(self: &Sysline) -> String {
        return self._to_String_raw(false);
    }
}

/// thread-safe Atomic Reference Counting Pointer to a `Sysline`
type SyslineP = Arc<Sysline>;
/// storage for `Sysline`
type Syslines = BTreeMap<FileOffset, SyslineP>;
/// range map where key is sysline begin to end `[ Sysline.fileoffset_begin(), Sysline.fileoffset_end()]`
/// and where value is sysline begin (`Sysline.fileoffset_begin()`). Use the value to lookup associated `Syslines` map
type SyslinesRangeMap = RangeMap<FileOffset, FileOffset>;
// DateTime typing

/// DateTime formatting pattern, passed to `chrono::datetime_from_str`
type DateTimePattern_str = str;
type DateTimePattern = String;
/// DateTimePattern for searching a line (not the results)
/// slice index begin, slice index end of entire datetime pattern
/// slice index begin just the datetime, slice index end just the datetime
/// TODO: why not define as a `struct` instead of a tuple?
type DateTime_Parse_Data<'a> = (&'a DateTimePattern_str, LineIndex, LineIndex, LineIndex, LineIndex);
type DateTime_Parse_Datas_ar<'a> = [DateTime_Parse_Data<'a>];
type DateTime_Parse_Datas_vec<'a> = Vec<DateTime_Parse_Data<'a>>;
/// return type for `SyslineReader::find_datetime_in_line`
type Result_FindDateTime<'a> = Result<(DateTime_Parse_Data<'a>, DateTimeL)>;
type Result_ParseDateTime = Result<(LineIndex, LineIndex, DateTimeL)>;
/// count of datetime format strings used
type DateTime_Pattern_Counts<'a> = HashMap<&'a DateTimePattern_str, u64>;
type SyslinesLRUCache = LruCache<FileOffset, ResultS4_SyslineFind>;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime1 {
    Pass,
    OccursAtOrAfter,
    OccursBefore,
}

impl Result_Filter_DateTime1 {
    /// Returns `true` if the result is [`OccursAfter`].
    #[inline]
    pub const fn is_after(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursAtOrAfter)
    }

    /// Returns `true` if the result is [`OccursBefore`].
    #[inline]
    pub const fn is_before(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursBefore)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime2 {
    /// PASS
    OccursInRange,
    /// FAIL
    OccursBeforeRange,
    /// FAIL
    OccursAfterRange,
}

impl Result_Filter_DateTime2 {
    #[inline]
    pub const fn is_pass(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursInRange)
    }

    #[inline]
    pub const fn is_fail(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursAfterRange | Result_Filter_DateTime2::OccursBeforeRange)
    }
}

/// Specialized Reader that uses `LineReader` to find syslog lines
pub struct SyslineReader<'syslinereader> {
    linereader: LineReader<'syslinereader>,
    /// Syslines by fileoffset_begin
    syslines: Syslines,
    // TODO: has `syslines_by_range` ever found a sysline?
    //       would be good to add a test for it.
    /// Syslines fileoffset by sysline fileoffset range, i.e. `[Sysline.fileoffset_begin(), Sysline.fileoffset_end()+1)`
    /// the stored value can be used as a key for `self.syslines`
    syslines_by_range: SyslinesRangeMap,
    /// datetime formatting data, for extracting datetime strings from Lines
    dt_patterns: DateTime_Parse_Datas_vec<'syslinereader>,
    dt_patterns_counts: DateTime_Pattern_Counts<'syslinereader>,
    // TODO: [2021/09/21] add efficiency stats
    // TODO: get rid of LRU cache
    /// internal LRU cache for `find_sysline`
    _find_sysline_lru_cache: SyslinesLRUCache,
}

// TODO: [2021/09/19]
//       put all filter data into one struct `SyslineFilter`, simpler to pass around

impl fmt::Debug for SyslineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyslineReader")
            .field("linereader", &self.linereader)
            .field("syslines", &self.syslines)
            .finish()
    }
}

/// quick debug helper
#[allow(non_snake_case, dead_code)]
fn debug_eprint_LRU_cache<K, V>(cache: &LruCache<K, V>)
where
    K: std::fmt::Debug,
    K: std::hash::Hash,
    K: Eq,
    V: std::fmt::Debug,
{
    if !cfg!(debug_assertions) {
        return;
    }
    debug_eprint!("[");
    for (key, val) in cache.iter() {
        debug_eprint!(" Key: {:?}, Value: {:?};", key, val);
    }
    debug_eprint!("]");
}

/// built-in datetime parsing patterns, these are all known patterns attempted on processed files
/// first string is a chrono strftime pattern
/// https://docs.rs/chrono/latest/chrono/format/strftime/
/// first two numbers are total string slice offsets
/// last two numbers are string slice offsets constrained to *only* the datetime portion
/// offset values are [X, Y) (beginning offset is inclusive, ending offset is exclusive or "one past")
/// i.e. string `"[2000-01-01 00:00:00]"`, the pattern may begin at `"["`, the datetime begins at `"2"`
///      same rule for the endings.
const DATETIME_PARSE_DATAS: [DateTime_Parse_Data; 17] = [
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/samba/log.10.7.190.134` (multi-line)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2020/03/05 12:17:59.631000,  3] ../source3/smbd/oplock.c:1340(init_oplocks)
    //        init_oplocks: initializing messages.
    //
    ("[%Y/%m/%d %H:%M:%S%.6f,", 0, 28, 1, 27),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware/hostd-62.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     2019-07-26T10:40:29.682-07:00 info hostd[03210] [Originator@6876 sub=Default] Current working directory: /usr/bin
    //
    // TODO: [2021/10/03] no support of differing TZ
    //("%Y-%m-%dT%H:%M:%S%.3f%z ", 0, 30, 0, 29),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/kernel.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     Mar  9 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode
    //
    // TODO: [2021/10/03] no support of inferring the year
    //("%b %e %H:%M:%S ", 0, 25, 0, 25),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/synology/synobackup.log` (has horizontal alignment tabs)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     info	2017/02/21 21:50:48	SYSTEM:	[Local][Backup Task LocalBackup1] Backup task started.
    //     err	2017/02/23 02:55:58	SYSTEM:	[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    // example escaped:
    //     info␉2017/02/21 21:50:48␉SYSTEM:␉[Local][Backup Task LocalBackup1] Backup task started.
    //     err␉2017/02/23 02:55:58␉SYSTEM:␉[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    //
    // TODO: [2021/10/03] no support of variable offset datetime
    //       this could be done by trying random offsets into something
    //       better is to search for a preceding regexp pattern
    //("\t%Y/%m/%d %H:%M:%S\t", 5, 24, 0, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/xrdp.log`
    // example with offset:
    //
    //               1
    //     01234567890123456789
    //     [20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
    //
    ("[%Y%m%d-%H:%M:%S]", 0, 19, 1, 18),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware-installer.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2019-05-06 11:24:34,074] Successfully loaded GTK libraries.
    //
    ("[%Y-%m-%d %H:%M:%S,%3f] ", 0, 26, 1, 24),
    // repeat prior but no trailing space
    ("[%Y-%m-%d %H:%M:%S,%3f]", 0, 25, 1, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/other/archives/proftpd/xferlog`
    // example with offset:
    //
    //               1         2
    //     0123456789012345678901234
    //     Sat Oct 03 11:26:12 2020 0 192.168.1.12 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //
    ("%a %b %d %H:%M:%S %Y ", 0, 25, 0, 24),
    // repeat prior but no trailing space
    ("%a %b %d %H:%M:%S %Y", 0, 24, 0, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/OpenSUSE15/zypper.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2019-05-23 16:53:43 <1> trenker(24689) [zypper] main.cc(main):74 ===== Hi, me zypper 1.14.27
    //
    ("%Y-%m-%d %H:%M:%S ", 0, 20, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/debian9/apport.log.1`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     ERROR: apport (pid 9) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 93) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 935) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9359) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //
    (" %a %b %d %H:%M:%S %Y:", 22, 47, 22, 46),
    (" %a %b %d %H:%M:%S %Y:", 23, 48, 23, 47),
    (" %a %b %d %H:%M:%S %Y:", 24, 49, 24, 48),
    (" %a %b %d %H:%M:%S %Y:", 25, 50, 25, 49),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01 00:00:01xyz
    //
    ("%Y-%m-%d %H:%M:%S", 0, 19, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01 xyz
    //
    ("%Y-%m-%dT%H:%M:%S ", 0, 20, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01xyz
    //
    ("%Y-%m-%dT%H:%M:%S", 0, 19, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101 000001 xyz
    //
    ("%Y%m%d %H%M%S ", 0, 16, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001 xyz
    //
    ("%Y%m%dT%H%M%S ", 0, 16, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001xyz
    //
    ("%Y%m%dT%H%M%S", 0, 15, 0, 15),
    // ---------------------------------------------------------------------------------------------
];

lazy_static! {
    static ref DATETIME_PARSE_DATAS_VEC: DateTime_Parse_Datas_vec<'static> =
        DateTime_Parse_Datas_vec::from(DATETIME_PARSE_DATAS);
}

/// implement SyslineReader things
impl<'syslinereader> SyslineReader<'syslinereader> {
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;
    const DT_PATTERN_MAX: usize = 2;

    pub fn new(path: &'syslinereader FPath, blocksz: BlockSz) -> Result<SyslineReader<'syslinereader>> {
        let lr = match LineReader::new(&path, blocksz) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("ERROR: LineReader::new({}, {}) failed {}", path, blocksz, err);
                return Err(err);
            }
        };
        Ok(SyslineReader {
            linereader: lr,
            syslines: Syslines::new(),
            syslines_by_range: SyslinesRangeMap::new(),
            _find_sysline_lru_cache: SyslinesLRUCache::new(4),
            dt_patterns: DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX),
            dt_patterns_counts: DateTime_Pattern_Counts::with_capacity(SyslineReader::DT_PATTERN_MAX),
        })
    }

    pub fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    pub fn filesz(&self) -> BlockSz {
        self.linereader.filesz()
    }

    pub fn path(&self) -> &str {
        self.linereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.linereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.linereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.linereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        self.linereader.file_blocks_count()
    }

    pub fn blockoffset_last(&self) -> BlockOffset {
        self.linereader.blockoffset_last()
    }

    /// smallest size character
    pub fn charsz(&self) -> usize {
        self.linereader._charsz
    }

    /// Testing helper only
    pub fn print(&self, fileoffset: FileOffset, raw: bool) {
        let syslinep: &SyslineP = match self.syslines.get(&fileoffset) {
            Some(val) => val,
            None => {
                eprintln!("ERROR: in print, self.syslines.get({}) returned None", fileoffset);
                return;
            }
        };
        for linep in &(*syslinep).lines {
            (*linep).print(raw);
        }
    }

    /// Testing helper only
    /// print all known `Sysline`s
    pub fn print_all(&self, raw: bool) {
        debug_eprintln!("{}print_all(true)", sn());
        for fo in self.syslines.keys() {
            self.print(*fo, raw);
        }
        debug_eprintln!("\n{}print_all(true)", sx());
    }

    /// is given `SyslineP` last in the file?
    fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        let filesz = self.filesz();
        let fo_end = (*syslinep).fileoffset_end();
        if (fo_end + 1) == filesz {
            return true;
        }
        assert_lt!(fo_end + 1, filesz, "fileoffset_end() {} is at or after filesz() fileoffset {}", fo_end, filesz);
        return false;
    }

    /// store passed `Sysline` in `self.syslines`
    fn insert_sysline(&mut self, line: Sysline) -> SyslineP {
        let fo_beg: FileOffset = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let slp = SyslineP::new(line);
        debug_eprintln!("{}SyslineReader.insert_sysline: syslines.insert({}, Sysline @{:p})", so(), fo_beg, &*slp);
        self.syslines.insert(fo_beg, slp.clone());
        // XXX: multi-byte character
        let fo_end1 = fo_end + (self.charsz() as FileOffset);
        debug_eprintln!(
            "{}SyslineReader.insert_sysline: syslines_by_range.insert(({}‥{}], {})",
            so(),
            fo_beg,
            fo_end1,
            fo_beg
        );
        self.syslines_by_range.insert(fo_beg..fo_end1, fo_beg);
        return slp;
    }

    /// if datetime found in `Line` returns `Ok` around
    /// indexes into `line` of found datetime string `(start of string, end of string)`
    /// else returns `Err`
    /// TODO: assumes Local TZ
    /// TODO: 2022/03/11 14:30:00
    ///      The concept of static pattern lengths (beg_i, end_i, actual_beg_i, actual_end_i) won't work for
    ///      variable length datetime patterns, i.e. full month names 'July 1, 2020' and 'December 1, 2020'.
    ///      Instead of fixing the current problem of unexpected datetime matches,
    ///      fix the concept problem of passing around fixed-length datetime strings. Then redo this.
    pub fn find_datetime_in_line(
        line: &Line, parse_data: &'syslinereader DateTime_Parse_Datas_vec,
    ) -> Result_FindDateTime<'syslinereader> {
        debug_eprintln!("{}find_datetime_in_line:(Line@{:p}) {:?}", sn(), &line, line.to_String_noraw());
        // skip easy case; no possible datetime
        if line.len() < 4 {
            debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::InvalidInput);", sx());
            return Result_FindDateTime::Err(Error::new(ErrorKind::InvalidInput, "Line is too short"));
        }
        // TODO: create `pub fnas_slice_first_X` that return slice of first X bytes,
        //       most cases only need first 30 or so bytes of line, so this less often crosses block boundaries
        let line_as_slice = line.as_slice();

        let mut i = 0;
        // `end_i` and `actual_end_i` is one past last char; exclusive.
        // `actual` are more confined slice offsets of the datetime,
        // XXX: it might be faster to skip the special formatting and look directly for the datetime stamp.
        //      calls to chrono are long according to the flamegraph.
        //      however, using the demarcating characters ("[", "]") does give better assurance.
        for (pattern, beg_i, end_i, actual_beg_i, actual_end_i) in parse_data.iter() {
            i += 1;
            debug_eprintln!("{}find_datetime_in_line: pattern tuple {} ({:?}, {}, {}, {}, {})", so(), i, pattern, beg_i, end_i, actual_beg_i, actual_end_i);
            debug_assert_lt!(beg_i, end_i, "Bad values beg_i end_i");
            debug_assert_ge!(actual_beg_i, beg_i, "Bad values actual_beg_i beg_i");
            debug_assert_le!(actual_end_i, end_i, "Bad values actual_end_i end_i");
            //debug_eprintln!("{}find_datetime_in_line searching for pattern {} {:?}", so(), i, pattern);
            let len_ = (end_i - beg_i) as LineIndex;
            // XXX: does not support multi-byte string; assumes single-byte
            if line_as_slice.len() < (*end_i) {
                debug_eprintln!(
                    "{}find_datetime_in_line: line len {} is too short for pattern {} len {} @({}, {}] {:?}",
                    so(),
                    line_as_slice.len(),
                    i,
                    len_,
                    beg_i,
                    end_i,
                    pattern,
                );
                continue;
            }
            // take a slice of the `line_as_slice` then convert to `str`
            // TODO: here is a place to use `bstr` that will handle failed encoding attempts
            debug_eprintln!("{}find_datetime_in_line: &line_as_slice[(*{})‥(*{})]", so(), beg_i, end_i);
            let dts = match str::from_utf8(&line_as_slice[(*beg_i)..(*end_i)]) {
                Ok(val) => val,
                Err(err) => {
                    debug_eprintln!("{}ERROR: find_datetime_in_line str::from_utf8 failed during pattern {} Utf8Error {}", so(), i, err);
                    continue;
                }
            };
            debug_eprintln!(
                "{}find_datetime_in_line: searching for pattern {} {:?} in {:?} (slice [{}‥{}] from Line {:?})",
                so(),
                i,
                pattern,
                str_to_nonraw_String(dts),
                beg_i,
                end_i,
                line.to_String_noraw(),
            );
            // TODO: [2021/10/03]
            //       according to flamegraph, this function `Local::datetime_from_str` takes a very large amount of
            //       runtime, around 20% to 25% of entire process runtime. How to improve that?
            //       Could I create my own hardcoded version for a few common patterns?
            let dt = match Local.datetime_from_str(dts, pattern) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_datetime_in_line matched pattern {} {:?} to String {:?} extrapolated datetime {:?}",
                        so(),
                        i,
                        pattern,
                        str_to_nonraw_String(dts),
                        val
                    );
                    val
                }
                Err(err) => {
                    debug_eprintln!("{}find_datetime_in_line: failed to match pattern {} ParseError {}", so(), i, err);
                    continue;
                }
            }; // end for(pattern, ...)
            debug_eprintln!("{}find_datetime_in_line: return Ok({}, {}, {});", sx(), beg_i, end_i, &dt);
            return Result_FindDateTime::Ok((
                (
                    pattern,
                    *beg_i as LineIndex,
                    *end_i as LineIndex,
                    *actual_beg_i as LineIndex,
                    *actual_end_i as LineIndex,
                ),
                dt,
            ));
        }

        debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::NotFound);", sx());
        return Result_FindDateTime::Err(Error::new(ErrorKind::NotFound, "No datetime found!"));
    }

    fn dt_patterns_update(&mut self, datetime_parse_data: DateTime_Parse_Data<'syslinereader>) {
        debug_eprintln!("{}dt_patterns_update(SyslineReader@{:p}, {:?})", sn(), self, datetime_parse_data);
        if &self.dt_patterns.len() >= &SyslineReader::DT_PATTERN_MAX {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns already DT_PATTERN_MAX length {:?}",
                sx(),
                self,
                &self.dt_patterns.len()
            );
            return;
        }
        for datetime_parse_data_ in &self.dt_patterns {
            if datetime_parse_data_.eq(&datetime_parse_data) {
                debug_eprintln!(
                    "{}dt_patterns_update(SyslineReader@{:p}) found eq DateTime_Parse_Data; skip self.dt_patterns.push",
                    sx(),
                    self
                );
                return;
            }
        }
        debug_eprintln!(
            "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns.push({:?})",
            sx(),
            self,
            datetime_parse_data
        );
        self.dt_patterns.push(datetime_parse_data);
    }

    fn dt_patterns_counts_update(&mut self, dt_pattern: &'syslinereader DateTimePattern_str) {
        debug_eprintln!("{}dt_pattern_count_update(SyslineReader@{:p}, {:?})", snx(), self, dt_pattern);
        // unnecessary but possibly useful sanity check
        //if self.dt_patterns.is_empty() {
        //    eprintln!("ERROR: self.dt_patterns.is_empty() which is unexpected");
        //    return;
        //}
        let counter = self.dt_patterns_counts.entry(dt_pattern).or_insert(1);
        *counter += 1;
    }

    /// attempt to parse the `DateTime`
    /// wraps call to `find_datetime_in_line` according to status of `self.dt_patterns`
    /// if `self.dt_patterns` is `None`, will set `self.dt_patterns`
    fn parse_datetime_in_line(&mut self, line: &Line) -> Result_ParseDateTime {
        // 2021/10/09 21:00:00 where does dt_patterns come from?
        // LAST WORKING HERE 2022/03/11 01:25:00
        //      the current problem is datetime_f used is "%Y-%m-%d %H:%M:%S " which for unknown reason matches
        //      "2020-01-01 00:00:00␊"
        //      1. why!?
        //      2. should not this first try pattern "%Y-%m-%d %H:%M:%S" ? or should there be a set of fallback patterns?
        //      3. what happens if the first datetime pattern matching searches chooses a pattern that works for
        //         some lines but not others? maybe this problem should be ignored for now?
        // LAST WORKING HERE 2022/03/15 01:40:00
        // fixed the test test_find_sysline_at_datetime_filter1. Can anything else be added to it?
        // afterward, review prior LAST WORKING HERE and then resume with top-of-file TODO and notes
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}); {:?}", sn(), self, line.to_String_noraw());
        if self.dt_patterns.is_empty() {
            debug_eprintln!("{}parse_datetime_in_line self.dt_patterns is empty", sn());
            // this `SyslineReader` has not determined it's own DateTime formatting data `self.dt_patterns`
            // so pass the built-in `DATETIME_PARSE_DATAS`.
            // Use the extra data returned by `find_datetime_in_line` to set `self.dt_patterns` once.
            // This will only happen once per `SyslineReader` (assuming a valid Syslog file)
            let result = SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC);
            let (datetime_parse_data, dt) = match result {
                Ok(val) => val,
                Err(err) => {
                    debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};", sx(), self, err);
                    return Err(err);
                }
            };
            self.dt_patterns_counts_update(datetime_parse_data.0);
            self.dt_patterns_update(datetime_parse_data.clone());
            debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Ok;", sx(), self);
            return Result_ParseDateTime::Ok((datetime_parse_data.3, datetime_parse_data.4, dt));
        }
        debug_eprintln!("{}parse_datetime_in_line self.dt_patterns has {} entries", so(), &self.dt_patterns.len());
        // have already determined DateTime formatting for this file, so
        // no need to try *all* built-in DateTime formats, just try the known good formats.
        let result = SyslineReader::find_datetime_in_line(line, &self.dt_patterns);
        let (datetime_parse_data, dt) = match result {
            Ok(val) => val,
            Err(err) => {
                // the known good format failed, so now try other formats.
                // TODO: [2022/03/15]
                //       if the "known good" formats failed, and there are 2 of them, then do not attempt to try
                //       more formats, risk getting an errant match. Instead, presume that at acuumulated x2 dt_patterns
                //       there should be no more guesses.
                //       in other words, consider one more value of `Result_ParseDateTime`:
                //       `TryMore` - this would effectively mean call `find_datetime_in_line(DATETIME_PARSE_DATAS_VEC)`
                //       hmm... but should such a determination eminate from `find_datetime_in_line` ? Probably not.
                //       Have to think about this.
                debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {}; try again using default DATETIME_PARSE_DATAS_VEC", so(), self, err);
                //return Err(err);
                match SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC) {
                    Ok((datetime_parse_data_, dt_)) => {
                        self.dt_patterns_counts_update(datetime_parse_data_.0);
                        self.dt_patterns_update(datetime_parse_data_.clone());
                        (datetime_parse_data_, dt_)
                    }
                    Err(err_) => {
                        debug_eprintln!(
                            "{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};",
                            sx(),
                            self,
                            err_
                        );
                        return Err(err_);
                    }
                }
            }
        };
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Ok;", sx(), self);
        return Result_ParseDateTime::Ok((datetime_parse_data.1, datetime_parse_data.2, dt));
    }

    /// find first sysline at or after `fileoffset`
    /// return (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`)
    /// similar to `find_line`, `read_block`
    /// XXX: this function is large and cumbersome
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        debug_eprintln!("{}find_sysline(SyslineReader@{:p}, {})", sn(), self, fileoffset);

        // check LRU cache
        match self._find_sysline_lru_cache.get(&fileoffset) {
            Some(rlp) => {
                // self.stats_read_block_cache_lru_hit += 1;
                debug_eprintln!("{}find_sysline: found LRU cached for fileoffset {}", so(), fileoffset);
                match rlp {
                    ResultS4_SyslineFind::Found(val) => {
                        debug_eprintln!("{}return ResultS4_SyslineFind::Found(({}, …)) @[{}, {}]", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_SyslineFind::Found((val.0, val.1.clone()));
                    }
                    ResultS4_SyslineFind::Found_EOF(val) => {
                        debug_eprintln!("{}return ResultS4_SyslineFind::Found_EOF(({}, …)) @[{}, {}]", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_SyslineFind::Found_EOF((val.0, val.1.clone()));
                    }
                    ResultS4_SyslineFind::Done => {
                        debug_eprintln!("{}return ResultS4_SyslineFind::Done", sx());
                        return ResultS4_SyslineFind::Done;
                    }
                    _ => {
                        debug_eprintln!(" Err");
                        eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {}", fileoffset);
                    }
                }
            }
            None => {
                //self.stats_read_block_cache_lru_miss += 1;
                debug_eprintln!("{}find_sysline: fileoffset {} not found in LRU cache", so(), fileoffset);
            }
        }

        // check if there is a Sysline already known at this fileoffset
        if self.syslines.contains_key(&fileoffset) {
            debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fileoffset);
            let slp = self.syslines[&fileoffset].clone();
            // XXX: multi-byte character encoding
            let fo_next = (*slp).fileoffset_end() + (self.charsz() as FileOffset);
            // TODO: determine if `fileoffset` is the last sysline of the file
            //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] {:?}",
                sx(),
                fo_next,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
            self._find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
            return ResultS4_SyslineFind::Found((fo_next, slp));
        } else {
            debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines", so(), fileoffset);
        }

        // TODO: test that retrieving by cache always returns the same ResultS4 enum value as without a cache

        // check if the offset is already in a known range
        match self.syslines_by_range.get_key_value(&fileoffset) {
            Some(range_fo) => {
                let range = range_fo.0;
                debug_eprintln!(
                    "{}find_sysline: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                    so(),
                    fileoffset,
                    range
                );
                let fo = range_fo.1;
                let slp = self.syslines[fo].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_next() + (self.charsz() as FileOffset);
                if self.is_sysline_last(&slp) {
                    debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] {:?}",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).fileoffset_begin(),
                        (*slp).fileoffset_end(),
                        (*slp).to_String_noraw()
                    );
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, slp.clone())));
                    return ResultS4_SyslineFind::Found_EOF((fo_next, slp));
                }
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                debug_eprintln!(
                    "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] {:?}",
                    sx(),
                    fo_next,
                    &*slp,
                    (*slp).fileoffset_begin(),
                    (*slp).fileoffset_end(),
                    (*slp).to_String_noraw()
                );
                return ResultS4_SyslineFind::Found((fo_next, slp));
            }
            None => {
                debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
            }
        }
        debug_eprintln!("{}find_sysline: searching for first sysline datetime A …", so());

        //
        // find line with datetime A
        //

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sl = Sysline::new();
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result: ResultS4_LineFind = self.linereader.find_line(fo1);
            let eof = result.is_eof();
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) | ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    debug_eprintln!("{}find_sysline: LRU cache put({}, Done)", so(), fileoffset);
                    self._find_sysline_lru_cache.put(fileoffset, ResultS4_SyslineFind::Done);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; A", sx());
                    return ResultS4_SyslineFind::Done;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); A", sx(), err);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result = self.parse_datetime_in_line(&*lp);
            debug_eprintln!("{}find_sysline: A find_datetime_in_line returned {:?}", so(), result);
            match result {
                Err(_) => {}
                Ok((dt_beg, dt_end, dt)) => {
                    // a datetime was found! beginning of a sysline
                    fo_a = fo1;
                    sl.dt_beg = dt_beg;
                    sl.dt_end = dt_end;
                    sl.dt = Some(dt);
                    debug_eprintln!("{}find_sysline: A sl.push({:?})", so(), (*lp).to_String_noraw());
                    sl.push(lp);
                    fo1 = sl.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}", dt_beg, dt_end);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}", dt_end, fo1 as usize);
                    if eof {
                        let slp = SyslineP::new(sl);
                        debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo1);
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo1, slp.clone())));
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF({}, {:p}) @[{}, {}]; A",
                            sx(),
                            fo1,
                            &(*slp),
                            (*slp).fileoffset_begin(),
                            (*slp).fileoffset_end(),
                        );
                        return ResultS4_SyslineFind::Found_EOF((fo1, slp));
                    }
                    break;
                }
            }
            debug_eprintln!("{}find_sysline: A skip push Line {:?}", so(), (*lp).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!(
            "{}find_sysline: found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            so(),
            fo_a,
            fo1
        );

        //
        // find line with datetime B
        //

        // TODO: can do an easy quick check if the beginning of next line is already known
        //       if self.syslines_by_range.contains_key(fo1) { return Ok(self.syslines_by_range.get(fo1).fileoffset_begin(), ...); }

        let mut fo_b: FileOffset = fo1;
        let mut eof = false;
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result = self.linereader.find_line(fo1);
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found_EOF(FileOffset {} Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    eof = true;
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found_EOF()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    //debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; B", sx());
                    debug_eprintln!("{}find_sysline: break; B", sx());
                    eof = true;
                    break;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); B", sx(), err);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result = self.parse_datetime_in_line(&*lp);
            debug_eprintln!("{}find_sysline: B find_datetime_in_line returned {:?}", so(), result);
            match result {
                Err(_) => {
                    debug_eprintln!(
                        "{}find_sysline: B append found Line to this Sysline sl.push({:?})",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    sl.push(lp);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    debug_eprintln!(
                        "{}find_sysline: B found datetime; end of this Sysline. Do not append found Line {:?}",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline: found line with datetime B at FileOffset {}", so(), fo_b);

        let slp = self.insert_sysline(sl);
        if eof {
            debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_b);
            self._find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_b, slp.clone())));
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, SyslineP@{:p}) @[{}, {}] {:?}",
                sx(),
                fo_b,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found_EOF((fo_b, slp));
        }
        debug_eprintln!("{}find_sysline: LRU cache put({}, Found({}, …))", so(), fileoffset, fo_b);
        self._find_sysline_lru_cache
            .put(fileoffset, ResultS4_SyslineFind::Found((fo_b, slp.clone())));
        debug_eprintln!(
            "{}find_sysline: return ResultS4_SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] {:?}",
            sx(),
            fo_b,
            &*slp,
            (*slp).fileoffset_begin(),
            (*slp).fileoffset_end(),
            (*slp).to_String_noraw()
        );
        return ResultS4_SyslineFind::Found((fo_b, slp));
    }

    /// find first sysline at or after `fileoffset` that is at or after `dt_filter`
    ///
    /// for example, given syslog file with datetimes:
    ///     20010101
    ///     20010102
    ///     20010103
    /// where the newline ending the first line is the ninth byte (fileoffset 9)
    ///
    /// calling
    ///     syslinereader.find_sysline_at_datetime_filter(0, Some(20010102 00:00:00-0000))
    /// will return
    ///     ResultS4::Found(19, SyslineP(data='20010102␊'))
    ///
    /// TODO: add more of these examples
    ///
    /// XXX: this function is large, cumbersome, and messy
    pub fn find_sysline_at_datetime_filter(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_at_datetime_filter";
        debug_eprintln!("{}{}(SyslingReader@{:p}, {}, {:?})", sn(), _fname, self, fileoffset, dt_filter,);
        let filesz = self.filesz();
        let fo_end: FileOffset = filesz as FileOffset;
        let mut try_fo: FileOffset = fileoffset;
        let mut try_fo_last: FileOffset = try_fo;
        let mut fo_last: FileOffset = fileoffset;
        let mut slp_opt: Option<SyslineP> = None;
        let mut slp_opt_last: Option<SyslineP> = None;
        let mut fo_a: FileOffset = fileoffset; // begin "range cursor" marker
        let mut fo_b: FileOffset = fo_end; // end "range cursor" marker
        loop {
            // TODO: [2021/09/26]
            //       this could be faster.
            //       currently it narrowing down to byte offset
            //       but it only needs to narrow down to range of a sysline
            //       so if `fo_a` and `fo_b` are in same sysline range, then this can return that sysline.
            //       Also, add stats for this function and debug print those stats before exiting.
            //       i.e. count of loops, count of calls to sysline_dt_before_after, etc.
            //       do this before tweaking function so can be compared
            debug_eprintln!("{}{}: loop(…)!", so(), _fname);
            let result = self.find_sysline(try_fo);
            let eof = result.is_eof();
            let done = result.is_done();
            match result {
                ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                    if !eof {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found({}, …) A",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    } else {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found_EOF({}, …) B",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    }
                    debug_eprintln!(
                        "{}{}: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?} C",
                        so(),
                        _fname,
                        fo,
                        &(*slp),
                        slp.lines.len(),
                        (*slp).len(),
                        (*slp).to_String_noraw(),
                    );
                    // here is the binary search algorithm in action
                    debug_eprintln!(
                        "{}{}: sysline_dt_after_or_before(@{:p} ({:?}), {:?})",
                        so(),
                        _fname,
                        &*slp,
                        (*slp).dt,
                        dt_filter
                    );
                    match SyslineReader::sysline_dt_after_or_before(&slp, dt_filter) {
                        Result_Filter_DateTime1::Pass => {
                            debug_eprintln!(
                                "{}{}: Pass => fo {} fo_last {} try_fo {} try_fo_last {} (fo_end {})",
                                so(),
                                _fname,
                                fo,
                                fo_last,
                                try_fo,
                                try_fo_last,
                                fo_end
                            );
                            debug_eprintln!(
                                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); A",
                                sx(),
                                _fname,
                                fo,
                                &*slp
                            );
                            return ResultS4_SyslineFind::Found((fo, slp));
                        } // end Pass
                        Result_Filter_DateTime1::OccursAtOrAfter => {
                            // the Sysline found by `find_sysline(try_fo)` occurs at or after filter `dt_filter`, so search backward
                            // i.e. move end marker `fo_b` backward
                            debug_eprintln!("{}{}: OccursAtOrAfter => fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, fo_end);
                            // short-circuit a common case, passed fileoffset is past the `dt_filter`, can immediately return
                            // XXX: does this mean my algorithm sucks?
                            if try_fo == fileoffset {
                                // first loop iteration
                                debug_eprintln!(
                                    "{}{}:                    try_fo {} == {} try_fo_last; early return",
                                    so(),
                                    _fname,
                                    try_fo,
                                    try_fo_last
                                );
                                debug_eprintln!(
                                    "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); B fileoffset {} {:?}",
                                    sx(),
                                    _fname,
                                    fo,
                                    &*slp,
                                    (*slp).fileoffset_begin(),
                                    (*slp).to_String_noraw()
                                );
                                return ResultS4_SyslineFind::Found((fo, slp));
                            }
                            try_fo_last = try_fo;
                            fo_b = std::cmp::min((*slp).fileoffset_begin(), try_fo_last);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursAtOrAfter
                        Result_Filter_DateTime1::OccursBefore => {
                            // the Sysline found by `find_sysline(try_fo)` occurs before filter `dt_filter`, so search forthward
                            // i.e. move begin marker `fo_a` forthward
                            debug_eprintln!("{}{}: OccursBefore =>    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, fo_end);
                            let slp_foe = (*slp).fileoffset_end();
                            assert_le!(slp_foe, fo, "unexpected values (*SyslineP).fileoffset_end() {}, fileoffset returned by find_sysline {}", slp_foe, fo);
                            try_fo_last = try_fo;
                            assert_le!(try_fo_last, slp_foe, "Unexpected values try_fo_last {} slp_foe {}, last tried offset (passed to self.find_sysline) is beyond returned sysline.fileoffset_end()!?", try_fo_last, slp_foe);
                            debug_eprintln!(
                                "{}{}:                    ∴ fo_a = min(slp_foe {}, fo_b {});",
                                so(),
                                _fname,
                                slp_foe,
                                fo_b
                            );
                            // LAST WORKING HERE [2021/10/06 00:05:00]
                            // LAST WORKING HERE [2022/03/16 01:15:00]
                            // this code passes all tests, but runs strangely. I think the problem is the first found sysline
                            // (that may or may not satisfy the passed filter) is placed into a queue and then printed by the waiting main thread.
                            fo_a = std::cmp::min(slp_foe, fo_b);
                            //fo_a = std::cmp::max(slp_foe, fo_b);
                            //fo_a = slp_foe;
                            //assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursBefore
                    } // end SyslineReader::sysline_dt_after_or_before()
                    debug_eprintln!("{}{}:                    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, fo_end);
                    fo_last = fo;
                    slp_opt_last = slp_opt;
                    slp_opt = Some(slp);
                    // TODO: [2021/09/26]
                    //       I think could do an early check and skip a few loops:
                    //       if `fo_a` and `fo_b` are offsets into the same Sysline
                    //       then that Sysline is the candidate, so return Ok(...)
                    //       unless `fo_a` and `fo_b` are past last Sysline.fileoffset_begin of the file then return Done
                    //       However, before implemetning that, implement the stats tracking of this function mentioned above,
                    //       be sure some improvement really occurs.
                } // end Found | Found_EOF
                ResultS4_SyslineFind::Done => {
                    debug_eprintln!("{}{}: SyslineReader.find_sysline(try_fo: {}) returned Done", so(), _fname, try_fo);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        fo_end
                    );
                    try_fo_last = try_fo;
                    debug_eprintln!(
                        "{}{}:                 ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                        so(),
                        _fname,
                        fo_a,
                        fo_b,
                        fo_a
                    );
                    try_fo = fo_a + ((fo_b - fo_a) / 2);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        fo_end
                    );
                } // end Done
                ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!(
                        "{}{}: SyslineReader.find_sysline(try_fo: {}) returned Err({})",
                        so(),
                        _fname,
                        try_fo,
                        err
                    );
                    eprintln!("ERROR: {}", err);
                    break;
                } // end Err
            } // match result
            debug_eprintln!("{}{}: next loop will try offset {} (fo_end {})", so(), _fname, try_fo, fo_end);

            // TODO: 2022/03/18 this latter part hints at a check that could be done sooner,
            //       before `try_fo==try_fo_last`, that would result in a bit less loops.
            //       A simpler and faster check is to do
            //           fo_next, slp = find_sysline(fileoffset)
            //           _, slp_next = find_sysline(fo_next)
            //       do this at the top of the loop. Then call `dt_after_or_before` for each
            //       `.dt` among `slp`, `slp_next`.

            // `try_fo == try_fo_last` means binary search loop is deciding on the same fileoffset upon each loop.
            // the searching is exhausted.
            if done && try_fo == try_fo_last {
                // reached a dead-end of searching the same fileoffset `find_sysline(try_fo)` and receiving Done
                // so this function is exhausted too.
                debug_eprintln!("{}{}: Done && try_fo {} == {} try_fo_last; break!", so(), _fname, try_fo, try_fo_last);
                break;
            } else if try_fo == try_fo_last {
                debug_eprintln!("{}{}: try_fo {} == {} try_fo_last;", so(), _fname, try_fo, try_fo_last);
                let mut slp = slp_opt.unwrap();
                let fo_beg = slp.fileoffset_begin();
                if self.is_sysline_last(&slp) && fo_beg < try_fo {
                    // binary search stopped at fileoffset past start of last Sysline in file
                    // so entirely past all acceptable syslines
                    debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; C binary searched ended after beginning of last sysline in the file", sx(), _fname,);
                    return ResultS4_SyslineFind::Done;
                }
                // binary search loop is deciding on the same fileoffset upon each loop. That fileoffset must refer to
                // an acceptable sysline. However, if that fileoffset is past `slp.fileoffset_begin` than the threshold
                // change of datetime for the `dt_filter` is the *next* Sysline.
                let fo_next = slp.fileoffset_next();
                // XXX: sanity check
                //debug_assert_eq!(fo_last, fo_next, "fo {} != {} slp.fileoffset_next()", fo_last, fo_next);
                if fo_beg < try_fo {
                    debug_eprintln!("{}{}: slp.fileoffset_begin() {} < {} try_fo;", so(), _fname, fo_beg, try_fo);
                    let slp_next = match self.find_sysline(fo_next) {
                        ResultS4_SyslineFind::Found_EOF((_, slp_)) => {
                            debug_eprintln!(
                                "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found_EOF(…, {:?})",
                                so(),
                                _fname,
                                fo_next,
                                slp_
                            );
                            slp_
                        }
                        ResultS4_SyslineFind::Found((_, slp_)) => {
                            debug_eprintln!(
                                "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found(…, {:?})",
                                so(),
                                _fname,
                                fo_next,
                                slp_
                            );
                            slp_
                        }
                        ResultS4_SyslineFind::Done => {
                            debug_eprintln!(
                                "{}{}: SyslineReader.find_sysline(fo_next1: {}) unexpectedly returned Done",
                                so(),
                                _fname,
                                fo_next
                            );
                            break;
                        }
                        ResultS4_SyslineFind::Err(err) => {
                            debug_eprintln!(
                                "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Err({})",
                                so(),
                                _fname,
                                fo_next,
                                err
                            );
                            eprintln!("ERROR: {}", err);
                            break;
                        }
                    };
                    debug_eprintln!("{}{}: dt_filter: {:?}", so(), _fname, dt_filter);
                    debug_eprintln!(
                        "{}{}: slp      : fo_beg {}, {:?} {:?}",
                        so(),
                        _fname,
                        fo_beg,
                        (*slp).dt.unwrap(),
                        (*slp).to_String_noraw()
                    );
                    debug_eprintln!(
                        "{}{}: slp_next : fo_beg {}, {:?} {:?}",
                        so(),
                        _fname,
                        (*slp_next).fileoffset_begin(),
                        (*slp_next).dt.unwrap(),
                        (*slp_next).to_String_noraw()
                    );
                    let slp_compare = Self::dt_after_or_before(&(*slp).dt.unwrap(), dt_filter);
                    let slp_next_compare = Self::dt_after_or_before(&(*slp_next).dt.unwrap(), dt_filter);
                    debug_eprintln!("{}{}: match({:?}, {:?})", so(), _fname, slp_compare, slp_next_compare);
                    slp = match (slp_compare, slp_next_compare) {
                        (_, Result_Filter_DateTime1::Pass) | (Result_Filter_DateTime1::Pass, _) => {
                            debug_eprintln!("{}{}: unexpected Result_Filter_DateTime1::Pass", so(), _fname);
                            eprintln!("ERROR: unexpected Result_Filter_DateTime1::Pass result");
                            break;
                        }
                        (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursBefore) => {
                            debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                            slp_next
                        }
                        (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursAtOrAfter) => {
                            debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                            slp_next
                        }
                        (Result_Filter_DateTime1::OccursAtOrAfter, Result_Filter_DateTime1::OccursAtOrAfter) => {
                            debug_eprintln!("{}{}: choosing slp", so(), _fname);
                            slp
                        }
                        _ => {
                            debug_eprintln!(
                                "{}{}: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple",
                                so(),
                                _fname
                            );
                            eprintln!("ERROR: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple");
                            break;
                        }
                    };
                } else {
                    debug_eprintln!(
                        "{}{}: slp.fileoffset_begin() {} >= {} try_fo; use slp",
                        so(),
                        _fname,
                        fo_beg,
                        try_fo
                    );
                    /*
                    debug_eprintln!("{}{}: slp.fileoffset_begin() {} >= {} try_fo; get next sysline at {}", so(), _fname, fo_beg, try_fo, fo_next);
                    slp = match self.find_sysline(fo_next) {
                        ResultS4_SyslineFind::Found_EOF((__, slp_)) |
                        ResultS4_SyslineFind::Found((__, slp_)) => {
                            debug_eprintln!("{}{}: SyslineReader.find_sysline(fo_next2: {}) returned Found|Found_EOF(…, {:?}); choose sysline at {}", so(), _fname, fo_next, slp_, fo_next);
                            slp_
                        },
                        ResultS4_SyslineFind::Done => {
                            debug_eprintln!("{}{}: SyslineReader.find_sysline(fo_next2: {}) unexpectedly returned Done; choose sysline at {}", so(), _fname, fo_next, fo_beg);
                            slp
                        }
                        ResultS4_SyslineFind::Err(err) => {
                            debug_eprintln!("{}{}: SyslineReader.find_sysline(fo_next2: {}) returned Err({})", so(), _fname, fo_next, err);
                            eprintln!("ERROR: {}", err);
                            break;
                        }
                    }
                    */
                }
                // XXX: sanity check
                //debug_assert_eq!(fo_last, slp.fileoffset_next(), "fo_last {} != {} slp.fileoffset_next()", fo_last, slp.fileoffset_next());
                if fo_last != slp.fileoffset_next() {
                    eprintln!(
                        "WARNING: fo_last {} != {} slp.fileoffset_next() (fo_end is {})",
                        fo_last,
                        slp.fileoffset_next(),
                        fo_end
                    );
                }
                debug_eprintln!(
                    "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); D fileoffset {} {:?}",
                    sx(),
                    _fname,
                    fo_last,
                    &*slp,
                    (*slp).fileoffset_begin(),
                    (*slp).to_String_noraw()
                );
                return ResultS4_SyslineFind::Found((fo_last, slp));
            }
        } // end loop

        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; E", sx(), _fname);
        return ResultS4_SyslineFind::Done;
    }

    /// if `dt` is at or after `dt_filter` then return `OccursAtOrAfter`
    /// if `dt` is before `dt_filter` then return `OccursBefore`
    /// else return `Pass` (including if `dt_filter` is `None`)
    pub fn dt_after_or_before(dt: &DateTimeL, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        if dt_filter.is_none() {
            debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::Pass; (no dt filters)", snx(),);
            return Result_Filter_DateTime1::Pass;
        }

        let dt_a = &dt_filter.unwrap();
        debug_eprintln!("{}dt_after_or_before comparing dt datetime {:?} to filter datetime {:?}", sn(), dt, dt_a);
        if dt < dt_a {
            debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursBefore; (dt {:?} is before dt_filter {:?})", sx(), dt, dt_a);
            return Result_Filter_DateTime1::OccursBefore;
        }
        debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursAtOrAfter; (dt {:?} is at or after dt_filter {:?})", sx(), dt, dt_a);
        return Result_Filter_DateTime1::OccursAtOrAfter;
    }

    /// convenience wrapper for `dt_after_or_before`
    pub fn sysline_dt_after_or_before(syslinep: &SyslineP, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        debug_eprintln!("{}sysline_dt_after_or_before(SyslineP@{:p}, {:?})", snx(), &*syslinep, dt_filter,);
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &*syslinep);

        let dt = (*syslinep).dt.unwrap();
        return Self::dt_after_or_before(&dt, dt_filter);
    }

    /// If both filters are `Some` and `syslinep.dt` is "between" the filters then return `Pass`
    /// comparison is "inclusive" i.e. `dt` == `dt_filter_after` will return `Pass`
    /// TODO: finish this docstring
    /// If both filters are `None` then return `Pass`
    pub fn dt_pass_filters(
        dt: &DateTimeL, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> Result_Filter_DateTime2 {
        debug_eprintln!("{}dt_pass_filters({:?}, {:?}, {:?})", sn(), dt, dt_filter_after, dt_filter_before,);
        if dt_filter_after.is_none() && dt_filter_before.is_none() {
            debug_eprintln!(
                "{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange; (no dt filters)",
                sx(),
            );
            return Result_Filter_DateTime2::OccursInRange;
        }
        if dt_filter_after.is_some() && dt_filter_before.is_some() {
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt < {:?} dt_fiter_before ???",
                so(),
                &dt_filter_after.unwrap(),
                dt,
                &dt_filter_before.unwrap()
            );
            let da = &dt_filter_after.unwrap();
            let db = &dt_filter_before.unwrap();
            assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
            if dt < da {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursBeforeRange;", sx());
                return Result_Filter_DateTime2::OccursBeforeRange;
            }
            if db < dt {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursAfterRange;", sx());
                return Result_Filter_DateTime2::OccursAfterRange;
            }
            // assert da < dt && dt < db
            assert_le!(da, dt, "Unexpected range values da dt");
            assert_le!(dt, db, "Unexpected range values dt db");
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        } else if dt_filter_after.is_some() {
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt ???",
                so(),
                &dt_filter_after.unwrap(),
                dt
            );
            let da = &dt_filter_after.unwrap();
            if dt < da {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursBeforeRange;", sx());
                return Result_Filter_DateTime2::OccursBeforeRange;
            }
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        } else {
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt {:?} < {:?} dt_filter_before ???",
                so(),
                dt,
                &dt_filter_before.unwrap()
            );
            let db = &dt_filter_before.unwrap();
            if db < dt {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursAfterRange;", sx());
                return Result_Filter_DateTime2::OccursAfterRange;
            }
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        }
    }

    /// wrapper for call to `dt_pass_filters`
    pub fn sysline_pass_filters(
        syslinep: &SyslineP, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> Result_Filter_DateTime2 {
        debug_eprintln!(
            "{}sysline_pass_filters(SyslineP@{:p}, {:?}, {:?})",
            sn(),
            &*syslinep,
            dt_filter_after,
            dt_filter_before,
        );
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &*syslinep);
        let dt = (*syslinep).dt.unwrap();
        let result = SyslineReader::dt_pass_filters(&dt, dt_filter_after, dt_filter_before);
        debug_eprintln!("{}sysline_pass_filters(…) return {:?};", sx(), result);
        return result;
    }
}

/// basic test of `SyslineReader.find_datetime_in_line`
#[allow(non_snake_case, dead_code)]
#[cfg(test)]
fn test_find_datetime_in_line(blocksz: BlockSz) {
    debug_eprintln!("{}test_find_datetime_in_line()", sn());

    let ntf1 = create_temp_file(
        "\
[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.
[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!
[20200113-11:03:08] [INFO ] starting xrdp with pid 23198
[20200113-11:03:08] [INFO ] listening to port 3389 on 0.0.0.0
[20200113-11:13:59] [INFO ] Socket 12: AF_INET6 connection received from ::ffff:127.0.0.1 port 55426
[20200113-11:13:59] [DEBUG] Closed socket 12 (AF_INET6 ::ffff:127.0.0.1 port 3389)
[20200113-11:13:59] [DEBUG] Closed socket 11 (AF_INET6 :: port 3389)
[20200113-11:13:59] [INFO ] Using default X.509 certificate: /etc/xrdp/cert.pem
[20200113-11:13:59] [INFO ] Using default X.509 key file: /etc/xrdp/key.pem
[20200113-11:13:59] [ERROR] Cannot read private key file /etc/xrdp/key.pem: Permission denied
[20200113-11:13:59] [ERROR] Certification error:
    UNABLE TO READ CERTIFICATE!
[20200113-11:13:59] [ERROR] Certification failed.
",
    );
    let path = String::from(ntf1.path().to_str().unwrap());

    let mut slr = match SyslineReader::new(&path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", &path, blocksz, err);
            return;
        }
    };

    let mut fo1: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Found|Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}test_find_datetime_in_line: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                fo1 = fo;
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        if done {
            break;
        }
    }

    debug_eprintln!("{}test_find_datetime_in_line()", sx());
}

#[cfg(test)]
type _test_find_sysline_at_datetime_filter_Checks<'a> = Vec<(FileOffset, &'a str, &'a str)>;

/// underlying test code for `SyslineReader.find_datetime_in_line`
/// called by other functions `test_find_sysline_at_datetime_filterX`
#[cfg(test)]
fn __test_find_sysline_at_datetime_filter(
    file_content: String, dt_pattern: DateTimePattern, blocksz: BlockSz,
    checks: _test_find_sysline_at_datetime_filter_Checks,
) {
    debug_eprintln!("{}__test_find_sysline_at_datetime_filter(…, {:?}, {}, …)", sn(), dt_pattern, blocksz);

    let ntf1 = create_temp_file(&file_content.as_str());
    let path = String::from(ntf1.path().to_str().unwrap());
    let mut slr = match SyslineReader::new(&path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {}) failed {}", &path, blocksz, err);
        }
    };
    for (fo1, dts, sline_expect) in checks.iter() {
        let dt = match Local.datetime_from_str(dts, dt_pattern.as_str()) {
            chrono::ParseResult::Ok(val) => val,
            chrono::ParseResult::Err(err) => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned {}", dts, dt_pattern, err);
            }
        };
        debug_eprintln!("{}Ok: dts {:?}", so(), str_to_nonraw_String(dts));
        let sline_expect_noraw = str_to_nonraw_String(sline_expect);
        debug_eprintln!("{}find_sysline_at_datetime_filter({}, {:?})", so(), fo1, dt);
        let result = slr.find_sysline_at_datetime_filter(*fo1, &Some(dt));
        match result {
            ResultS4_SyslineFind::Found(val) | ResultS4_SyslineFind::Found_EOF(val) => {
                let sline = val.1.to_String();
                let sline_noraw = str_to_nonraw_String(sline.as_str());
                debug_eprintln!("\nexpected: {:?}", sline_expect_noraw);
                debug_eprintln!("returned: {:?}\n", sline_noraw);
                //print_colored(Color::Yellow, format!("expected: {}\n", sline_expect_noraw).as_bytes());
                //print_colored(Color::Yellow, format!("returned: {}\n", sline_noraw).as_bytes());
                assert_eq!(
                    sline,
                    String::from(*sline_expect),
                    "Expected {:?} == {:?} but it is not!",
                    sline_noraw,
                    sline_expect_noraw
                );
                //debug_eprintln!("{}Check PASSED {:?}", so(), sline_noraw);
                print_colored(
                    Color::Green,
                    format!(
                        "Check PASSED SyslineReader().find_sysline_at_datetime_filter({} {:?}) == {:?}\n",
                        fo1, dts, sline_noraw
                    )
                    .as_bytes(),
                );
            }
            ResultS4_SyslineFind::Done => {
                panic!("During test unexpected result Done");
            }
            ResultS4_SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }

    debug_eprintln!("{}_test_find_sysline_at_datetime_filter(…)", sx());
}

// TODO: [2022/03/16] create test cases with varying sets of Checks passed-in, current setup is always
//       clean, sequential series of checks from file_offset 0.
// TODO: BUG: [2022/03/15] why are these checks done in random order? The tests pass but run
//       in a confusing manner. Run `cargo test` to see.
/// basic test of `SyslineReader.find_datetime_in_line`
#[cfg(test)]
fn _test_find_sysline_at_datetime_filter(
    blocksz: BlockSz, checks: Option<_test_find_sysline_at_datetime_filter_Checks>,
) {
    stack_offset_set(None);
    debug_eprintln!("{}_test_find_sysline_at_datetime_filter()", sn());
    let dt_fmt1: DateTimePattern = String::from("%Y-%m-%d %H:%M:%S");
    let file_content1 = String::from(
        "\
2020-01-01 00:00:00
2020-01-01 00:00:01a
2020-01-01 00:00:02ab
2020-01-01 00:00:03abc
2020-01-01 00:00:04abcd
2020-01-01 00:00:05abcde
2020-01-01 00:00:06abcdef
2020-01-01 00:00:07abcdefg
2020-01-01 00:00:08abcdefgh
2020-01-01 00:00:09abcdefghi
2020-01-01 00:00:10abcdefghij
2020-01-01 00:00:11abcdefghijk
2020-01-01 00:00:12abcdefghijkl
2020-01-01 00:00:13abcdefghijklm
2020-01-01 00:00:14abcdefghijklmn
2020-01-01 00:00:15abcdefghijklmno
2020-01-01 00:00:16abcdefghijklmnop
2020-01-01 00:00:17abcdefghijklmnopq
2020-01-01 00:00:18abcdefghijklmnopqr
2020-01-01 00:00:19abcdefghijklmnopqrs
2020-01-01 00:00:20abcdefghijklmnopqrst
2020-01-01 00:00:21abcdefghijklmnopqrstu
2020-01-01 00:00:22abcdefghijklmnopqrstuv
2020-01-01 00:00:23abcdefghijklmnopqrstuvw
2020-01-01 00:00:24abcdefghijklmnopqrstuvwx
2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy
2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz
",
    );
    let checks0: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        (0, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
        (0, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
        (0, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
        (0, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
        (0, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
        (0, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
        (0, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
        (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
        (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        (0, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
        (0, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
        (0, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
        (0, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
        (0, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
        (0, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
        (0, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
        (0, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
        (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let checksx: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (19, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (40, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (62, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (85, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        (109, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
        (134, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
        (162, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
        (187, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
        (215, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
        (244, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
        (274, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
        (305, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        (337, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
        (370, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        (404, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        (439, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
        (475, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
        (512, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
        (550, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
        (589, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
        (629, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
        (670, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
        (712, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
        (755, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (799, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (844, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let mut checks_ = checks0;
    if checks.is_some() {
        checks_ = checks.unwrap();
    }
    __test_find_sysline_at_datetime_filter(file_content1, dt_fmt1, blocksz, checks_);
    debug_eprintln!("{}_test_find_sysline_at_datetime_filter()", sx());
}

// XXX: are these different BlockSz tests necessary? are not these adequately tested by
//      other lower-level tests?

#[test]
fn test_find_sysline_at_datetime_filter_4() {
    _test_find_sysline_at_datetime_filter(4, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_8() {
    _test_find_sysline_at_datetime_filter(8, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_16() {
    _test_find_sysline_at_datetime_filter(16, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_32() {
    _test_find_sysline_at_datetime_filter(32, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_64() {
    _test_find_sysline_at_datetime_filter(64, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_128() {
    _test_find_sysline_at_datetime_filter(128, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_256() {
    _test_find_sysline_at_datetime_filter(256, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_512() {
    _test_find_sysline_at_datetime_filter(512, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_1024() {
    _test_find_sysline_at_datetime_filter(1024, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_2056() {
    _test_find_sysline_at_datetime_filter(2056, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_() {
    _test_find_sysline_at_datetime_filter(64,Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:00",
            "2020-01-01 00:00:00\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            19,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            40,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            62,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            85,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            109,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            134,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            160,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            187,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            215,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            244,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            274,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            305,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            337,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            370,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            404,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            439,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            475,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            512,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            550,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            589,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            629,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            670,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            712,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            755,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            799,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            844,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_z_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_y_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_x_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_za() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ya() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_xa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ma() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3____() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ab() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__az() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__bd() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ml() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__my() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__mz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_byo() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

// TODO: [2022/03/18] create one wrapper test test_find_sysline_at_datetime_checks_ that takes some
//        vec of test-input-output, and does all possible permutations.

/// basic test of `SyslineReader.sysline_pass_filters`
#[allow(non_snake_case, dead_code)]
#[test]
fn test_sysline_pass_filters() {
    debug_eprintln!("{}test_sysline_pass_filters()", sn());

    fn DTL(s: &str) -> DateTimeL {
        return Local.datetime_from_str(s, &"%Y%m%dT%H%M%S").unwrap();
    }

    for (da, dt, db, exp_result) in [
        (
            Some(DTL(&"20000101T010105")),
            DTL(&"20000101T010106"),
            Some(DTL(&"20000101T010107")),
            Result_Filter_DateTime2::OccursInRange,
        ),
        (
            Some(DTL(&"20000101T010107")),
            DTL(&"20000101T010106"),
            Some(DTL(&"20000101T010108")),
            Result_Filter_DateTime2::OccursBeforeRange,
        ),
        (
            Some(DTL(&"20000101T010101")),
            DTL(&"20000101T010106"),
            Some(DTL(&"20000101T010102")),
            Result_Filter_DateTime2::OccursAfterRange,
        ),
        (Some(DTL(&"20000101T010101")), DTL(&"20000101T010106"), None, Result_Filter_DateTime2::OccursInRange),
        (
            Some(DTL(&"20000101T010102")),
            DTL(&"20000101T010101"),
            None,
            Result_Filter_DateTime2::OccursBeforeRange,
        ),
        (Some(DTL(&"20000101T010101")), DTL(&"20000101T010101"), None, Result_Filter_DateTime2::OccursInRange),
        (None, DTL(&"20000101T010101"), Some(DTL(&"20000101T010106")), Result_Filter_DateTime2::OccursInRange),
        (
            None,
            DTL(&"20000101T010101"),
            Some(DTL(&"20000101T010100")),
            Result_Filter_DateTime2::OccursAfterRange,
        ),
        (None, DTL(&"20000101T010101"), Some(DTL(&"20000101T010101")), Result_Filter_DateTime2::OccursInRange),
    ] {
        let result = SyslineReader::dt_pass_filters(&dt, &da, &db);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?}, {:?})", exp_result, result, dt, da, db);
        #[allow(unused_must_use)]
        print_colored(
            Color::Green,
            format!("{}({:?}, {:?}, {:?}) returned expected {:?}\n", so(), dt, da, db, result).as_bytes(),
        );
    }
    debug_eprintln!("{}test_sysline_pass_filters()", sx());
}

/// basic test of `SyslineReader.dt_after_or_before`
#[allow(non_snake_case)]
#[test]
fn test_dt_after_or_before() {
    debug_eprintln!("{}test_dt_after_or_before()", sn());

    fn DTL(s: &str) -> DateTimeL {
        return Local.datetime_from_str(s, &"%Y%m%dT%H%M%S").unwrap();
    }

    for (dt, da, exp_result) in [
        (DTL(&"20000101T010106"), None, Result_Filter_DateTime1::Pass),
        (DTL(&"20000101T010101"), Some(DTL(&"20000101T010103")), Result_Filter_DateTime1::OccursBefore),
        (DTL(&"20000101T010100"), Some(DTL(&"20000101T010100")), Result_Filter_DateTime1::OccursAtOrAfter),
        (DTL(&"20000101T010109"), Some(DTL(&"20000101T010108")), Result_Filter_DateTime1::OccursAtOrAfter),
    ] {
        let result = SyslineReader::dt_after_or_before(&dt, &da);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?})", exp_result, result, dt, da);
        #[allow(unused_must_use)]
        print_colored(
            Color::Green,
            format!("{}({:?}, {:?}) returned expected {:?}\n", so(), dt, da, result).as_bytes(),
        );
    }
    debug_eprintln!("{}test_dt_after_or_before()", sx());
}

// LAST WORKING HERE 2022/03/19 01:56:12 this is panicking for `test_SyslineReader_128_0`
/// testing helper
/// if debug then print with color
/// else print efficiently
/// XXX: does not handle multi-byte
/// BUG: if `(*slp).dt_beg` or `(*slp).dt_end` are within multi-byte encoded character
///      then this will panic. e.g. Sysline with underlying "2000-01-01 00:00:00\n".to_String_noraw()
///      will return "2000-01-01 00:00:00␊". Which will panic:
///          panicked at 'byte index 20 is not a char boundary; it is inside '␊' (bytes 19..22) of `2000-01-01 00:00:00␊`'
///      However, this function is only an intermediary development helper. Can this problem have a
///      brute-force workaround. 
fn print_slp(slp: &SyslineP) {
    if cfg!(debug_assertions) {
        let out = (*slp).to_String_noraw();
        // XXX: presumes single-byte character encoding, does not handle multi-byte encoding
        /*
        debug_eprintln!("{}print_slp: to_String_noraw() {:?} dt_beg {} dt_end {} len {}", so(), out, split_ab, (*slp).dt_end, (*slp).len());
        debug_eprintln!("{}print_slp: out.chars():", so());
        for (c_n, c_) in out.chars().enumerate() {
            debug_eprintln!("{}print_slp:              char {} {:?}", so(), c_n, c_);
        }
        debug_eprintln!("{}print_slp: out.bytes():", so());
        for (b_n, b_) in out.bytes().enumerate() {
            debug_eprintln!("{}print_slp:              byte {} {:?}", so(), b_n, b_);
        }
        */
        let a = &out[..(*slp).dt_beg];
        match print_colored(Color::Green, &a.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored a returned error {}", err);
            }
        };
        let b = &out[(*slp).dt_beg..(*slp).dt_end];
        match print_colored(Color::Yellow, &b.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored b returned error {}", err);
            }
        };
        let c = &out[(*slp).dt_end..];
        match print_colored(Color::Green, &c.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored c returned error {}", err);
            }
        };
        println!();
    } else {
        //(*slp_).print(true);
        let slices = (*slp).get_slices();
        for slice in slices.iter() {
            write_stdout(slice);
        }
    }
}

#[cfg(test)]
type _test_SyslineReader_check<'a> = (FileOffset, &'a str);
#[cfg(test)]
type _test_SyslineReader_checks<'a> = Vec::<(FileOffset, &'a str)>;

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader(path: &Path, blocksz: BlockSz, fileoffset: FileOffset, checks: &_test_SyslineReader_checks) {
    test_init();
    debug_eprintln!("{}test_SyslineReader({:?}, {})", sn(), &path, blocksz);
    let fpath: FPath = path.to_str().unwrap_or("").to_string();
    let mut slr = match SyslineReader::new(&fpath, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", fpath, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}test_SyslineReader: {:?}", so(), slr);

    let mut fo1: FileOffset = fileoffset;
    let mut check_i: usize = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last! Should have returned Found_EOF or is this Sysline not last?");
                fo1 = fo;

                // check fileoffset
                let check_fo = checks[check_i].0;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
                // check slp.String
                let check_String = checks[check_i].1.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value {:?}\nfind_sysline returned {:?}", check_String, actual_String);
            }
            ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!");
                fo1 = fo;

                // check fileoffset
                let check_fo = checks[check_i].0;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
                // check slp.String
                let check_String = checks[check_i].1.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value {:?}\nfind_sysline returned {:?}", check_String, actual_String);
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        check_i += 1;
        if done {
            break;
        }
    }
    assert_eq!(checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", checks.len(), check_i);

    debug_eprintln!("{}test_SyslineReader: Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    debug_eprintln!("{}test_SyslineReader({:?}, {})", sx(), &path, blocksz);
}

#[cfg(test)]
static test_data_file_basicdt5: &str = &"\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

#[cfg(test)]
static test_data_file_basicdt5_checks: [_test_SyslineReader_check; 6] = [
    (20, "2000-01-01 00:00:00\n"),
    (41, "2000-01-01 00:00:01a\n"),
    (63, "2000-01-01 00:00:02ab\n"),
    (86, "2000-01-01 00:00:03abc\n"),
    (110, "2000-01-01 00:00:04abcd\n"),
    (134, "2000-01-01 00:00:05abcde"),
];

#[test]
fn test_SyslineReader_basicdt5_128_0_()
{
    let checks = _test_SyslineReader_checks::from(test_data_file_basicdt5_checks);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 0, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128_1_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_basicdt5_checks[1..]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 1, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128_2_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_basicdt5_checks[2..]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 40, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128_3_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_basicdt5_checks[3..]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 84, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128_4_a()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_basicdt5_checks[3..]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 85, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128_4_b()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_basicdt5_checks[4..]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 86, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128_5_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_basicdt5_checks[5..]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 132, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128__()
{
    let checks = _test_SyslineReader_checks::from([]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 135, &checks);
}

#[test]
fn test_SyslineReader_basicdt5_128__9999()
{
    let checks = _test_SyslineReader_checks::from([]);
    let ntf = create_temp_file(test_data_file_basicdt5);
    test_SyslineReader(ntf.path(), 128, 9999, &checks);
}

// LAST WORKING HERE 2022/03/19 21:11:23 getting these tests test_SyslineReader_basicdt5* to work.
// After that, add *at least* one more data set.
//  see test_data_file_dt5
// then extraploate more tests for test_SyslineReader_w_filtering*

#[cfg(test)]
static test_data_file_dt5: &str = &"\
[ERROR] 2000-01-01 00:00:00
[ERROR] 2000-01-01 00:00:01a
[ERROR] 2000-01-01 00:00:02ab
[ERROR] 2000-01-01 00:00:03abc
[ERROR] 2000-01-01 00:00:04abcd
[ERROR] 2000-01-01 00:00:05abcde";

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_1(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    if cfg!(debug_assertions) {
        let s1 = file_to_nonraw_String(path);
        #[allow(unused_must_use)]
        print_colored(Color::Yellow, s1.as_bytes());
        println!();
    }

    let mut slr = match SyslineReader::new(path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}{:?}", so(), slr);

    let mut fo1: FileOffset = 0;
    let filesz = slr.filesz();
    while fo1 < filesz {
        debug_eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
        let result = slr.find_sysline_at_datetime_filter(fo1, &filter_dt_after_opt);
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found({}, @{:p})",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    fo,
                    &*slp
                );
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print!("FileOffset {:3} {:?} '", fo1, filter_dt_after_opt);
                let snippet = slr
                    .linereader
                    .blockreader
                    ._vec_from(fo1, std::cmp::min(fo1 + 40, filesz));
                #[allow(unused_must_use)]
                print_colored(Color::Yellow, buffer_to_nonraw_String(snippet.as_slice()).as_bytes());
                print!("' ");
                //print_slp(&slp);
                let slices = (*slp).get_slices();
                for slice in slices.iter() {
                    #[allow(unused_must_use)]
                    print_colored(Color::Green, slice);
                }
                println!();
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt
                );
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    err
                );
                eprintln!("ERROR: {}", err);
            }
        }
        fo1 += 1;
        debug_eprintln!("\n");
    }

    debug_eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?}, {:?})",
        sx(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
}

/// print the filtered syslines for a SyslineReader
/// quick debug helper
fn process_SyslineReader(
    slr: &mut SyslineReader, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!("{}process_SyslineReader({:?}, {:?}, {:?})", sn(), slr, filter_dt_after_opt, filter_dt_before_opt,);
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    debug_eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
    let result = slr.find_sysline_at_datetime_filter(fo1, &filter_dt_after_opt);
    match result {
        ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found|Found_EOF({}, @{:p})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                fo,
                &*slp
            );
            debug_eprintln!(
                "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                so(),
                fo,
                &(*slp),
                slp.lines.len(),
                (*slp).len(),
                (*slp).to_String_noraw(),
            );
            fo1 = fo;
            print_slp(&slp);
        }
        ResultS4_SyslineFind::Done => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt
            );
            search_more = false;
        }
        ResultS4_SyslineFind::Err(err) => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                err
            );
            eprintln!("ERROR: {}", err);
            search_more = false;
        }
    }
    if !search_more {
        debug_eprintln!("{}! search_more", so());
        debug_eprintln!("{}process_SyslineReader(…)", sx());
        return;
    }
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                if eof {
                    debug_eprintln!("{}slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo2, fo, &*slp);
                } else {
                    debug_eprintln!("{}slr.find_sysline({}) returned Found({}, @{:p})", so(), fo2, fo, &*slp);
                }
                fo2 = fo;
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                debug_eprintln!(
                    "{}sysline_pass_filters({:?}, {:?}, {:?})",
                    so(),
                    (*slp).dt,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                );
                match SyslineReader::sysline_pass_filters(&slp, filter_dt_after_opt, filter_dt_before_opt) {
                    Result_Filter_DateTime2::OccursBeforeRange | Result_Filter_DateTime2::OccursAfterRange => {
                        debug_eprintln!(
                            "{}sysline_pass_filters returned not Result_Filter_DateTime2::OccursInRange; continue!",
                            so()
                        );
                        continue;
                    }
                    Result_Filter_DateTime2::OccursInRange => {
                        print_slp(&slp);
                        if eof {
                            assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!?");
                        } else {
                            assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last!? Should have returned Found_EOF or this Sysline is really not last.");
                        }
                    }
                }
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!("{}slr.find_sysline({}) returned Done", so(), fo2);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo2, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }
    debug_eprintln!("{}process_SyslineReader({:?}, …)", sx(), slr.path());
}

/// quick debug helper
fn process_file<'a>(
    path: &'a FPath, blocksz: BlockSz, filter_dt_after_opt: &'a DateTimeL_Opt, filter_dt_before_opt: &'a DateTimeL_Opt,
) -> Option<Box<SyslineReader<'a>>> {
    debug_eprintln!(
        "{}process_file({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let slr = match SyslineReader::new(path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
            return None;
        }
    };
    debug_eprintln!("{}{:?}", so(), slr);
    debug_eprintln!("{}process_file(…)", sx());
    return Some(Box::new(slr));
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_2(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_2({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let slr_opt = process_file(path, blocksz, filter_dt_after_opt, filter_dt_before_opt);
    if slr_opt.is_some() {
        let slr = &slr_opt.unwrap();
        debug_eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    }
    debug_eprintln!("{}test_SyslineReader_w_filtering_2(…)", sx());
}

/// basic test of SyslineReader things
/// process multiple files
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_3(
    paths: &Vec<String>, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_3({:?}, {}, {:?}, {:?})",
        sn(),
        &paths,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    let mut slrs = Vec::<SyslineReader>::with_capacity(paths.len());
    for path in paths.iter() {
        debug_eprintln!("{}SyslineReader::new({:?}, {})", so(), path, blocksz);
        let slr = match SyslineReader::new(path, blocksz) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", path, blocksz, err);
                return;
            }
        };
        debug_eprintln!("{}{:?}", so(), slr);
        slrs.push(slr)
    }
    for slr in slrs.iter_mut() {
        process_SyslineReader(slr, filter_dt_after_opt, filter_dt_before_opt);
        println!();
    }
    debug_eprintln!("{}test_SyslineReader_w_filtering_3(…)", sx());
}

/// basic test of SyslineReader things
/// read all file offsets but randomly
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_rand(path_: &FPath, blocksz: BlockSz) {
    debug_eprintln!("{}test_SyslineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let mut slr1 = match SyslineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}SyslineReader {:?}", so(), slr1);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(slr1.filesz() as usize);
    fill(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    for fo1 in offsets_rand {
        let result = slr1.find_sysline(fo1);
        match result {
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}slr1.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
            }
            _ => {}
        }
    }
    // should print the file as-is and not be affected by random reads
    slr1.print_all(true);
    debug_eprintln!("\n{}{:?}", so(), slr1);
    debug_eprintln!("{}test_SyslineReader_rand(…)", sx());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogWriter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type SyslineReaders<'syslogwriter> = Vec<SyslineReader<'syslogwriter>>;

/// Specialized Writer that coordinates writing multiple SyslineReaders
pub struct SyslogWriter<'syslogwriter> {
    syslinereaders: SyslineReaders<'syslogwriter>,
}

impl<'syslogwriter> SyslogWriter<'syslogwriter> {
    pub fn new(syslinereaders: SyslineReaders<'syslogwriter>) -> SyslogWriter<'syslogwriter> {
        assert_gt!(syslinereaders.len(), 0, "Passed zero SyslineReaders");
        SyslogWriter { syslinereaders }
    }

    pub fn push(&mut self, syslinereader: SyslineReader<'syslogwriter>) {
        self.syslinereaders.push(syslinereader);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// multi-threaded
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

// would using the premade ThreadPool be easier?
// https://docs.rs/threadpool/1.8.1/threadpool/
// or Rayon threading
// https://crates.io/crates/rayon
//
// these might be good to read:
// https://pkolaczk.github.io/multiple-threadpools-rust/
//
// https://doc.rust-lang.org/book/ch16-03-shared-state.html#atomic-reference-counting-with-arct
// https://doc.rust-lang.org/book/ch20-02-multithreaded.html

// -------------------------------------------------------------------------------------------------
// threading try #1
// -------------------------------------------------------------------------------------------------

/// Thread Handle
type ThreadHandle = thread::JoinHandle<()>;
/// Thread Handle Arc
//type ThreadHandleA<'a> = Arc<&'a ThreadHandle>;
/// Thread Handle Mutex
//type ThreadHandleM = Mutex<ThreadHandle>;
/// Thread Handle Arc Mutex
//type ThreadHandleAM = Arc<ThreadHandleM>;

struct Worker {
    id: usize,
    //thread: ThreadHandle
    //threadam: ThreadHandleAM,
    //threadam_opt: Option<ThreadHandleAM>,
    //threada: ThreadHandleA<'a>,
    thread_opt: Option<ThreadHandle>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing!", id);
            job();
        });
        //let threadam = ThreadHandleAM::new(
        //    ThreadHandleM::new(thread)
        //);
        //let threada = ThreadHandleA::new(thread);

        Worker {
            id,
            //threadam_opt: Some(ThreadHandleAM::new(ThreadHandleM::new(thread))),
            //threadam: ThreadHandleAM::new(ThreadHandleM::new(thread)),
            thread_opt: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::<Worker>::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }

    pub fn wait(&mut self) {
        debug_eprintln!("{}ThreadPool::wait()", sn());
        // the sending channel must be closed so receivers know to stop waiting on it, yes? no?
        //std::mem::drop(self.sender);
        //self.sender.drop();
        for worker in self.workers.iter_mut() {
            let id = worker.id;
            debug_eprintln!("{}ThreadPool::wait join …", so());

            //worker.threadam.lock().unwrap().join().unwrap();
            //worker.threadam_opt.take().unwrap().lock().unwrap().join().unwrap();
            //worker.threada.join().unwrap();
            //worker.thread.join().unwrap();

            // BUG: [2021/09/29] this compiles but runtime gets stuck here.
            // XXX: is it because of the printing?
            match worker.thread_opt.take().unwrap().join() {
                Ok(_) => {
                    debug_eprintln!("joined thread {}!", id);
                }
                Err(err) => {
                    eprintln!("Error thread {} joining {:?}", id, err);
                }
            }
        }
        debug_eprintln!("{}ThreadPool::wait()", sx());
    }
}

/// testing helper
#[cfg(test)]
fn exec1() {
    let _ = 0;
    debug_eprintln!("exec1");
}

/// testing helper
#[cfg(test)]
fn test_threading_1() {
    debug_eprintln!("{}test_threading_1()", sn());

    debug_eprintln!("{}ThreadPool::new(5)", so());
    let mut tp = ThreadPool::new(5);
    for _ in 0..5 {
        tp.execute(exec1);
    }
    tp.wait();
    debug_eprintln!("{}test_threading_1()", sx());
}

// -------------------------------------------------------------------------------------------------
// threading try #2
// -------------------------------------------------------------------------------------------------

extern crate rayon;

/// testing helper
#[cfg(test)]
fn exec2(path: FPath) -> i32 {
    let r_ = rand::random::<i32>();
    debug_eprintln!("exec2 {:?} {}", path, r_);
    //return 33;
    return r_;
}

// based on
// https://pkolaczk.github.io/multiple-threadpools-rust/
#[cfg(test)]
fn test_threading_2() {
    debug_eprintln!("{}test_threading_2()", sn());

    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let files: Vec<String> = Vec::<String>::from([
        String::from("./logs/other/tests/basic-basic-dt20.log"),
        String::from("./logs/other/tests/basic-basic-dt30-repeats-longlines.log"),
        String::from("./logs/other/tests/basic-dt5.log"),
    ]);
    let (tx, rx) = std::sync::mpsc::channel();
    for f in files.into_iter() {
        let tx = tx.clone();
        pool.spawn(move || {
            tx.send(exec2(f)).unwrap();
        });
    }
    drop(tx); // need to close all senders, otherwise...
    let hashes: Vec<i32> = rx.into_iter().collect(); // ... this would block
    debug_eprintln!("{}hashes {:?}", so(), hashes);

    debug_eprintln!("{}test_threading_2()", sx());
}

// -------------------------------------------------------------------------------------------------

/// given the vector of `DateTimeL`, return the vector index and value of the soonest
/// (minimum) value within a `Some`
/// If the vector is empty then return `None`
#[allow(dead_code)]
fn datetime_soonest2(vec_dt: &Vec<DateTimeL>) -> Option<(usize, DateTimeL)> {
    if vec_dt.is_empty() {
        return None;
    }

    let mut index: usize = 0;
    for (index_, _) in vec_dt.iter().enumerate() {
        if vec_dt[index_] < vec_dt[index] {
            index = index_;
        }
    }

    Some((index, vec_dt[index].clone()))
}

/// test function `datetime_soonest2`
#[test]
fn test_datetime_soonest2() {
    debug_eprintln!("{}test_datetime_soonest2()", sn());
    let vec0 = Vec::<DateTimeL>::with_capacity(0);
    let val = datetime_soonest2(&vec0);
    assert!(val.is_none());

    let dt1_a = Local
        .datetime_from_str("2001-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S")
        .unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let dt2_a = Local
        .datetime_from_str("2002-01-01T11:00:00", "%Y-%m-%dT%H:%M:%S")
        .unwrap();
    let vec2a: Vec<DateTimeL> = vec![dt1_a.clone(), dt2_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec2a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let vec2b: Vec<DateTimeL> = vec![dt2_a.clone(), dt1_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec2b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt1_a);

    let dt3 = Local
        .datetime_from_str("2000-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S")
        .unwrap();
    let vec3a: Vec<DateTimeL> = vec![dt1_a.clone(), dt2_a.clone(), dt3.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec3a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3a");
        }
    };
    assert_eq!(i_, 2);
    assert_eq!(dt_, dt3);

    let vec3b: Vec<DateTimeL> = vec![dt1_a.clone(), dt3.clone(), dt2_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec3b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt3);

    let vec3c: Vec<DateTimeL> = vec![dt3.clone(), dt1_a.clone(), dt2_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec3c) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3c");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt3);

    debug_eprintln!("{}test_datetime_soonest2()", sx());
}

// -------------------------------------------------------------------------------------------------
// threading try #3
// -------------------------------------------------------------------------------------------------

/*
type Share_Stdout = Arc<Mutex<i32>>;

fn print_slp_color_guarded(mutex_stdout: &Share_Stdout, slp: &SyslineP, color: &Color) {
    let slices = (*slp).get_slices();
    let vigil = mutex_stdout.lock().unwrap();
    for slice in slices.iter() {
        #[allow(unused_must_use)]
        print_colored(color.clone(), slice);
    }
    println!();
    drop(vigil);
}
*/

type Thread_Init_Data = (FPath, BlockSz, DateTimeL_Opt, DateTimeL_Opt);
type Is_SLP_Last = bool;
type SLP_Last = (SyslineP, Is_SLP_Last);
type Chan_Send_SLP = crossbeam_channel::Sender<SLP_Last>;
type Chan_Recv_SLP = crossbeam_channel::Receiver<SLP_Last>;

fn exec_3(chan_send_dt: Chan_Send_SLP, thread_init_data: Thread_Init_Data) -> thread::ThreadId {
    stack_offset_set(None);
    debug_eprintln!("{}exec_3(…)", sn());
    let (path, blocksz, filter_dt_after_opt, filter_dt_before_opt) = thread_init_data;
    let tid = thread::current().id();

    let mut slr = match SyslineReader::new(&path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", path, blocksz, err);
            return tid;
        }
    };

    // find and print first sysline acceptable to the passed filters
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    let result = slr.find_sysline_at_datetime_filter(fo1, &filter_dt_after_opt);
    match result {
        ResultS4_SyslineFind::Found((fo, slp)) => {
            let is_last = slr.is_sysline_last(&slp);
            fo1 = fo;
            debug_eprintln!("thread {:?}:A chan_send_dt.send(…);", tid);
            match chan_send_dt.send((slp, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send((slp.clone(), is_last)) failed {}", err);
                }
            }
        }
        ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
            let is_last = slr.is_sysline_last(&slp);
            debug_eprintln!("thread {:?}:B chan_send_dt.send(…);", tid);
            match chan_send_dt.send((slp, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: B chan_send_dt.send((slp.clone(), is_last)) failed {}", err);
                }
            }
            search_more = false;
        }
        ResultS4_SyslineFind::Done => {
            search_more = false;
        }
        ResultS4_SyslineFind::Err(err) => {
            eprintln!("ERROR: {}", err);
            search_more = false;
        }
    }
    if !search_more {
        debug_eprintln!("{}exec_3(…)", sx());
        return tid;
    }
    // print all proceeding syslines acceptable to the passed filters
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                fo2 = fo;
                match SyslineReader::sysline_pass_filters(&slp, &filter_dt_after_opt, &filter_dt_before_opt) {
                    Result_Filter_DateTime2::OccursBeforeRange | Result_Filter_DateTime2::OccursAfterRange => {
                        continue;
                    }
                    Result_Filter_DateTime2::OccursInRange => {
                        let is_last = slr.is_sysline_last(&slp);
                        debug_eprintln!("thread {:?}:C chan_send_dt.send(…);", tid);
                        match chan_send_dt.send((slp, is_last)) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("ERROR: C chan_send_dt.send((slp.clone(), is_last)) failed {}", err);
                            }
                        }
                    }
                }
                if eof {
                    break;
                }
            }
            ResultS4_SyslineFind::Done => {
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }

    debug_eprintln!("{}exec_3(…)", sx());
    return tid;
}

/// basic threading implementation. Satisfies multi-threaded milestone.
/// TODO: [2021/10/03] put this into a proper function or struct, clean it up.
fn basic_threading_3(
    paths: &Vec<FPath>, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!("{}basic_threading_3()", sn());

    let queue_sz_dt: usize = 10;
    let file_count = paths.len();

    //
    // create a single ThreadPool with one thread per path
    //
    let pool = rayon::ThreadPoolBuilder::new().num_threads(file_count).build().unwrap();
    //
    // prepare per-thread data
    // create necessary channels for each thread
    // launch each thread
    //
    //type Map_Path_Chan_Recv_SLP = HashMap::<&FPath, Chan_Recv_SLP>;
    let mut map_path_recv_dt = HashMap::<&FPath, Chan_Recv_SLP>::with_capacity(file_count);
    let mut map_path_color = HashMap::<&FPath, Color>::with_capacity(file_count);
    // XXX: are these channels necessary?
    let (chan_send_1, chan_recv_1) = std::sync::mpsc::channel();
    for fpath in paths.iter() {
        let thread_data: Thread_Init_Data =
            (fpath.clone(), blocksz, filter_dt_after_opt.clone(), filter_dt_before_opt.clone());
        let (chan_send_dt, chan_recv_dt): (Chan_Send_SLP, Chan_Recv_SLP) = crossbeam_channel::bounded(queue_sz_dt);
        map_path_recv_dt.insert(fpath, chan_recv_dt);
        map_path_color.insert(fpath, color_rand());
        let chan_send_1_thrd = chan_send_1.clone();
        // XXX: how to name the threads?
        pool.spawn(move || match chan_send_1_thrd.send(exec_3(chan_send_dt, thread_data)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: chan_send_1_thrd.send(exec_3(chan_send_dt, thread_data)) failed {}", err);
            }
        });
    }

    // XXX: is this needed?
    debug_eprintln!("{}drop({:?});", so(), chan_send_1);
    drop(chan_send_1); // close sender so chan.into_iter.collect does not block

    type Map_Path_SLP = BTreeMap<FPath, SLP_Last>;

    /// run `.recv` on many Receiver channels simultaneously with the help of `crossbeam_channel::Select`
    /// https://docs.rs/crossbeam-channel/0.5.1/crossbeam_channel/struct.Select.html
    /// XXX: I would like to return a `&FPath` to avoid one `FPath.clone()` but it causes
    ///      compiler error about mutable and immutable borrows of `map_path_slp` occurring simultaneously
    ///          cannot borrow `map_path_slp` as mutable because it is also borrowed as immutable
    fn recv_many_chan(
        fpath_chans: &HashMap<&FPath, Chan_Recv_SLP>, filter_: &Map_Path_SLP,
    ) -> (FPath, std::result::Result<SLP_Last, crossbeam_channel::RecvError>) {
        debug_eprintln!("{}recv_many_chan();", sn());
        // "mapping" of index to data; required for various `Select` and `SelectedOperation` procedures
        let mut imap = Vec::<(&FPath, &Chan_Recv_SLP)>::with_capacity(fpath_chans.len());
        // Build a list of operations
        let mut select = crossbeam_channel::Select::new();
        for fp_chan in fpath_chans.iter() {
            // if there is already a DateTime "on hand" for the given fpath then
            // skip receiving on the associated channel
            if filter_.contains_key(*fp_chan.0) {
                continue;
            }
            imap.push((fp_chan.0, fp_chan.1));
            debug_eprintln!("{}select.recv({:?});", so(), fp_chan.1);
            // load `select` with `recv` operations, to be run during later `.select()`
            select.recv(fp_chan.1);
        }
        assert_gt!(imap.len(), 0, "No recv operations to select on");
        debug_eprintln!("{}v: {:?}", so(), imap);
        // Do the `select` operation
        let soper = select.select();
        // get the index of the chosen "winner" of the `select` operation
        let index = soper.index();
        debug_eprintln!("{}soper.recv(&v[{:?}]);", so(), index);
        let fpath = imap[index].0;
        let chan = &imap[index].1;
        debug_eprintln!("{}chan: {:?}", so(), chan);
        // Get the result of the `recv` done during `select`
        let result = soper.recv(chan);
        debug_eprintln!("{}recv_many_chan() return ({:?}, {:?});", sx(), fpath, result);
        return (fpath.clone(), result);
    }

    //
    // main coordination loop (e.g. "main game loop")
    // process the "receiving sysline" channels from the running threads
    // print the soonest available sysline
    //

    // XXX: BTreeMap does not implement `with_capacity`
    let mut map_path_slp = Map_Path_SLP::new();
    // crude debugging stats
    let mut _count_recv_ok: usize = 0;
    let mut _count_recv_di: usize = 0;
    loop {
        let mut disconnected = Vec::<FPath>::with_capacity(map_path_recv_dt.len());
        // if there is a DateTime "on hand" for every FPath channel (one channel is one FPath) then
        // they can all be compared, and the soonest DateTime selected then printed
        if map_path_recv_dt.len() == map_path_slp.len() {
            let mut fp1: FPath = FPath::new();
            // XXX: arbitrary code block here to allow later `map_path_slp.remove`;
            //      hacky workaround for a difficult error:
            //          "cannot borrow `map_path_slp` as mutable more than once at a time"
            {
                // XXX: my small investigation into `min`, `max`, `min_by`, `max_by`
                //      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=a6d307619a7797b97ef6cfc1635c3d33
                let (fpath_min, slp_min, is_last) =
                    match map_path_slp.iter_mut().min_by(|x, y| x.1 .0.dt.cmp(&y.1 .0.dt)) {
                        Some(val) => (val.0, &val.1 .0, val.1 .1),
                        None => {
                            eprintln!("map_path_slp.iter().min_by returned None");
                            // XXX: not sure what else to do here
                            continue;
                        }
                    };
                // print the sysline!
                if cfg!(debug_assertions) {
                    let out = fpath_min.to_string()
                        + &String::from(": ")
                        + &(slp_min.to_String_noraw())
                        + &String::from("\n");
                    let clr: Color = map_path_color.get(fpath_min).unwrap().clone();
                    print_colored(clr, out.as_bytes());
                    if is_last {
                        write_stdout(&NLu8a);
                    }
                } else {
                    (*slp_min).print2();
                    if is_last {
                        write_stdout(&NLu8a);
                    }
                }
                fp1 = (*fpath_min).clone();
            }
            assert_ne!(fp1, String::from(""), "Empty filepath");
            map_path_slp.remove(&fp1);
        } else {
            // else waiting on a (datetime, syslinep) from a file
            let (fp1, result1) = recv_many_chan(&map_path_recv_dt, &map_path_slp);
            match result1 {
                Ok(slp_last) => {
                    debug_eprintln!("{}crossbeam_channel::Found for FPath {:?};", so(), fp1);
                    map_path_slp.insert(fp1, slp_last);
                    _count_recv_ok += 1;
                }
                Err(crossbeam_channel::RecvError) => {
                    debug_eprintln!("{}crossbeam_channel::RecvError for FPath {:?};", so(), fp1);
                    disconnected.push(fp1);
                    _count_recv_di += 1;
                }
            }
        }
        // remove channels that have been disconnected
        for fpath in disconnected.into_iter() {
            debug_eprintln!("{}map_path_recv_dt.remove({:?});", so(), fpath);
            map_path_recv_dt.remove(&fpath);
        }
        // are there any channels to receive from?
        if map_path_recv_dt.is_empty() {
            debug_eprintln!("{}map_path_recv_dt.is_empty();", so());
            break;
        }
        debug_eprintln!("{}map_path_recv_dt: {:?}", so(), map_path_recv_dt);
        debug_eprintln!("{}map_path_slp: {:?}", so(), map_path_slp);
    } // loop

    debug_eprintln!("{}_count_recv_ok {:?} _count_recv_di {:?}", so(), _count_recv_ok, _count_recv_di);
    debug_eprintln!("{}basic_threading_3()", sx());
}

// -------------------------------------------------------------------------------------------------
// threading try #4
// -------------------------------------------------------------------------------------------------

fn exec_4(chan_send_dt: Chan_Send_SLP, thread_init_data: Thread_Init_Data) -> thread::ThreadId {
    debug_eprintln!("{}exec_4(…)", sn());
    let (path, blocksz, filter_dt_after_opt, filter_dt_before_opt) = thread_init_data;
    let tid = thread::current().id();

    let mut slr = match SyslineReader::new(&path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", path, blocksz, err);
            return tid;
        }
    };

    // find first sysline acceptable to the passed filters
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    let result = slr.find_sysline_at_datetime_filter(fo1, &filter_dt_after_opt);
    match result {
        ResultS4_SyslineFind::Found((fo, slp)) => {
            let is_last = slr.is_sysline_last(&slp);
            fo1 = fo;
            debug_eprintln!("thread {:?}:A chan_send_dt.send(…);", tid);
            match chan_send_dt.send((slp, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send((slp.clone(), is_last)) failed {}", err);
                }
            }
        }
        ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
            let is_last = slr.is_sysline_last(&slp);
            debug_eprintln!("thread {:?}:B chan_send_dt.send(…);", tid);
            match chan_send_dt.send((slp, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: B chan_send_dt.send((slp.clone(), is_last)) failed {}", err);
                }
            }
            search_more = false;
        }
        ResultS4_SyslineFind::Done => {
            search_more = false;
        }
        ResultS4_SyslineFind::Err(err) => {
            eprintln!("ERROR: {}", err);
            search_more = false;
        }
    }
    if !search_more {
        debug_eprintln!("{}exec_3(…)", sx());
        return tid;
    }
    // find all proceeding syslines acceptable to the passed filters
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                fo2 = fo;
                match SyslineReader::sysline_pass_filters(&slp, &filter_dt_after_opt, &filter_dt_before_opt) {
                    Result_Filter_DateTime2::OccursBeforeRange | Result_Filter_DateTime2::OccursAfterRange => {
                        continue;
                    }
                    Result_Filter_DateTime2::OccursInRange => {
                        let is_last = slr.is_sysline_last(&slp);
                        debug_eprintln!("thread {:?}:C chan_send_dt.send(…);", tid);
                        match chan_send_dt.send((slp, is_last)) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("ERROR: C chan_send_dt.send((slp.clone(), is_last)) failed {}", err);
                            }
                        }
                    }
                }
                if eof {
                    break;
                }
            }
            ResultS4_SyslineFind::Done => {
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }

    debug_eprintln!("{}exec_4(…)", sx());
    return tid;
}

fn test_threading_4(
    paths: &Vec<FPath>, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!("{}test_threading_4()", sn());

    let queue_sz_dt: usize = 10;
    let file_count = paths.len();

    //
    // create a single ThreadPool with one thread per path
    //
    debug_eprintln!("{}test_threading_4: rayon::ThreadPoolBuilder::new().num_threads({}).build()", so(), file_count);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(file_count).build().unwrap();
    //
    // prepare per-thread data
    // create necessary channels for each thread
    // launch each thread
    //
    //type Map_Path_Chan_Recv_SLP = HashMap::<&FPath, Chan_Recv_SLP>;
    let mut map_path_recv_dt = HashMap::<&FPath, Chan_Recv_SLP>::with_capacity(file_count);
    let mut map_path_color = HashMap::<&FPath, Color>::with_capacity(file_count);
    // XXX: are these channels necessary?
    let (chan_send_1, chan_recv_1) = std::sync::mpsc::channel();
    for fpath in paths.iter() {
        let thread_data: Thread_Init_Data =
            (fpath.clone(), blocksz, filter_dt_after_opt.clone(), filter_dt_before_opt.clone());
        let (chan_send_dt, chan_recv_dt): (Chan_Send_SLP, Chan_Recv_SLP) = crossbeam_channel::bounded(queue_sz_dt);
        map_path_recv_dt.insert(fpath, chan_recv_dt);
        map_path_color.insert(fpath, color_rand());
        let chan_send_1_thrd = chan_send_1.clone();
        // XXX: how to name the threads?
        pool.spawn(move || match chan_send_1_thrd.send(exec_3(chan_send_dt, thread_data)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: chan_send_1_thrd.send(exec_3(chan_send_dt, thread_data)) failed {}", err);
            }
        });
    }

    // XXX: is this needed?
    debug_eprintln!("{}test_threading_4: drop({:?});", so(), chan_send_1);
    drop(chan_send_1); // close sender so chan.into_iter.collect does not block

    type Map_Path_SLP = BTreeMap<FPath, SLP_Last>;

    /// run `.recv` on many Receiver channels simultaneously with the help of `crossbeam_channel::Select`
    /// https://docs.rs/crossbeam-channel/0.5.1/crossbeam_channel/struct.Select.html
    /// XXX: I would like to return a `&FPath` to avoid one `FPath.clone()` but it causes
    ///      compiler error about mutable and immutable borrows of `map_path_slp` occurring simultaneously
    ///      cannot borrow `map_path_slp` as mutable because it is also borrowed as immutable
    fn recv_many_chan(
        fpath_chans: &HashMap<&FPath, Chan_Recv_SLP>, filter_: &Map_Path_SLP,
    ) -> (FPath, std::result::Result<SLP_Last, crossbeam_channel::RecvError>) {
        debug_eprintln!("{}test_threading_4:recv_many_chan();", sn());
        // "mapping" of index to data; required for various `Select` and `SelectedOperation` procedures
        let mut imap = Vec::<(&FPath, &Chan_Recv_SLP)>::with_capacity(fpath_chans.len());
        // Build a list of operations
        let mut select = crossbeam_channel::Select::new();
        for fp_chan in fpath_chans.iter() {
            // if there is already a DateTime "on hand" for the given fpath then
            // skip receiving on the associated channel
            if filter_.contains_key(*fp_chan.0) {
                continue;
            }
            imap.push((fp_chan.0, fp_chan.1));
            debug_eprintln!("{}test_threading_4:recv_many_chan: select.recv({:?});", so(), fp_chan.1);
            // load `select` with `recv` operations, to be run during later `.select()`
            select.recv(fp_chan.1);
        }
        assert_gt!(imap.len(), 0, "No recv operations to select on");
        debug_eprintln!("{}test_threading_4:recv_many_chan: v: {:?}", so(), imap);
        // Do the `select` operation
        let soper = select.select();
        // get the index of the chosen "winner" of the `select` operation
        let index = soper.index();
        debug_eprintln!("{}test_threading_4:recv_many_chan: soper.recv(&v[{:?}]);", so(), index);
        let fpath = imap[index].0;
        let chan = &imap[index].1;
        debug_eprintln!("{}test_threading_4:recv_many_chan: chan: {:?}", so(), chan);
        // Get the result of the `recv` done during `select`
        let result = soper.recv(chan);
        debug_eprintln!("{}test_threading_4:recv_many_chan() return ({:?}, {:?});", sx(), fpath, result);
        return (fpath.clone(), result);
    }

    //
    // main coordination loop (e.g. "main game loop")
    // process the "receiving sysline" channels from the running threads
    // print the soonest available sysline
    //

    // XXX: BTreeMap does not implement `with_capacity`
    let mut map_path_slp = Map_Path_SLP::new();
    // crude debugging stats
    let mut _count_recv_ok: usize = 0;
    let mut _count_recv_di: usize = 0;
    loop {
        let mut disconnected = Vec::<FPath>::with_capacity(map_path_recv_dt.len());
        // if there is a DateTime "on hand" for every FPath channel (one channel is one FPath) then
        // they can all be compared, and the soonest DateTime selected then printed
        if map_path_recv_dt.len() == map_path_slp.len() {
            let mut fp1: FPath = FPath::new();
            // XXX: arbitrary code block here to allow later `map_path_slp.remove`;
            //      hacky workaround for a difficult error:
            //          "cannot borrow `map_path_slp` as mutable more than once at a time"
            {
                // XXX: my small investigation into `min`, `max`, `min_by`, `max_by`
                //      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=a6d307619a7797b97ef6cfc1635c3d33
                let (fpath_min, slp_min, is_last) =
                    match map_path_slp.iter_mut().min_by(|x, y| x.1 .0.dt.cmp(&y.1 .0.dt)) {
                        Some(val) => (val.0, &val.1 .0, val.1 .1),
                        None => {
                            eprintln!("map_path_slp.iter().min_by returned None");
                            // XXX: not sure what else to do here
                            continue;
                        }
                    };
                // print the sysline!
                if cfg!(debug_assertions) {
                    let out = fpath_min.to_string()
                        + &String::from(": ")
                        + &(slp_min.to_String_noraw())
                        + &String::from("\n");
                    let clr: Color = map_path_color.get(fpath_min).unwrap().clone();
                    print_colored(clr, out.as_bytes());
                    if is_last {
                        write_stdout(&NLu8a);
                    }
                } else {
                    (*slp_min).print2();
                    // TODO: [2021/10/4]
                    //       do not write extra stdout if only one file was processed, so program output
                    //       can be tested against `cat`
                    if is_last {
                        write_stdout(&NLu8a);
                    }
                }
                fp1 = (*fpath_min).clone();
            }
            assert_ne!(fp1, String::from(""), "Empty filepath");
            map_path_slp.remove(&fp1);
        } else {
            // else waiting on a (datetime, syslinep) from a file
            debug_eprintln!("{}test_threading_4: recv_many_chan(map_path_recv_dt: {:?}, map_path_slp: {:?})", so(), map_path_recv_dt, map_path_slp);
            let (fp1, result1) = recv_many_chan(&map_path_recv_dt, &map_path_slp);
            match result1 {
                Ok(slp_last) => {
                    debug_eprintln!("{}test_threading_4: crossbeam_channel::Found for FPath {:?};", so(), fp1);
                    map_path_slp.insert(fp1, slp_last);
                    _count_recv_ok += 1;
                }
                Err(crossbeam_channel::RecvError) => {
                    debug_eprintln!("{}test_threading_4: crossbeam_channel::RecvError for FPath {:?};", so(), fp1);
                    disconnected.push(fp1);
                    _count_recv_di += 1;
                }
            }
        }
        // remove channels that have been disconnected
        for fpath in disconnected.into_iter() {
            debug_eprintln!("{}test_threading_4: map_path_recv_dt.remove({:?});", so(), fpath);
            map_path_recv_dt.remove(&fpath);
        }
        // are there any channels to receive from?
        if map_path_recv_dt.is_empty() {
            debug_eprintln!("{}test_threading_4: map_path_recv_dt.is_empty();", so());
            break;
        }
        debug_eprintln!("{}test_threading_4: map_path_recv_dt: {:?}", so(), map_path_recv_dt);
        debug_eprintln!("{}test_threading_4: map_path_slp: {:?}", so(), map_path_slp);
    } // loop

    debug_eprintln!("{}test_threading_4: _count_recv_ok {:?} _count_recv_di {:?}", so(), _count_recv_ok, _count_recv_di);
    debug_eprintln!("{}test_threading_4()", sx());
}
