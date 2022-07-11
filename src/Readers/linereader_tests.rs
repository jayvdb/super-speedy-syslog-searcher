// Readers/linereader_tests.rs
//

use crate::common::{
    FileOffset,
    FPath,
    Bytes,
};

use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::filepreprocessor::{
    guess_filetype_from_fpath,
};

use crate::Data::line::{
    LineIndex,
    enum_BoxPtrs,
};

use crate::Readers::linereader::{
    LineReader,
    ResultS4_LineFind,
};

#[cfg(test)]
use crate::Readers::helpers::{
    randomize,
    fill,
};

use crate::printer_debug::helpers::{
    NamedTempFile,
    create_temp_file,
    NTF_Path,
};

#[cfg(test)]
use crate::printer_debug::helpers::{
    eprint_file,
};

use crate::printer_debug::printers::{
    byte_to_char_noraw,
    buffer_to_String_noraw,
};

use crate::printer_debug::stack::{
    sn,
    so,
    sx,
    stack_offset_set,
};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_lt,
    assert_ge,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_empty0: NamedTempFile = create_temp_file("");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_empty0_path: FPath = NTF_Path(&NTF_empty0);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_1: NamedTempFile = create_temp_file("\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_1_path: FPath = NTF_Path(&NTF_nl_1);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_2: NamedTempFile = create_temp_file("\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_2_path: FPath = NTF_Path(&NTF_nl_2);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_3: NamedTempFile = create_temp_file("\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_3_path: FPath = NTF_Path(&NTF_nl_3);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_4: NamedTempFile = create_temp_file("\n\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_4_path: FPath = NTF_Path(&NTF_nl_4);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_5: NamedTempFile = create_temp_file("\n\n\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_5_path: FPath = NTF_Path(&NTF_nl_5);
}

// -------------------------------------------------------------------------------------------------

/// dummy version of `ResultS4_LineFind` for asserting return enum of `LineReader::find_line`
#[derive(Debug, Eq, PartialEq)]
enum ResultS4_LineFind_Test {
    Found,
    Found_EOF,
    Done,
}

// -------------------------------------------------------------------------------------------------

/// helper to wrap the match and panic checks
#[cfg(test)]
fn new_LineReader(path: &FPath, blocksz: BlockSz) -> LineReader {
    let (filetype, _mimeguess) = guess_filetype_from_fpath(path);
    match LineReader::new(path.clone(), filetype, blocksz) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: LineReader::new({:?}, {}) failed {}", path, blocksz, err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// loop on `LineReader.find_line` until it is done
/// this is the most straightforward use of `LineReader`
#[cfg(test)]
fn process_LineReader(lr1: &mut LineReader) {
    eprintln!("{}process_LineReader()", sn());
    let mut fo1: FileOffset = 0;
    loop {
        eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let count = lr1.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    count,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                fo1 = fo;
                (*lp).print(false);
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let count = lr1.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    count,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                fo1 = fo;
                (*lp).print(false);
            }
            ResultS4_LineFind::Done => {
                eprintln!("{}ResultS4_LineFind::Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                panic!("ERROR: {}", err);
            }
        }
    }
    eprintln!("{}process_LineReader()", sx());
}

// -----------------------------------------------------------------------------

/// test `LineReader::find_line`
///
/// the `LineReader` instance reads `data`
/// assert the line count
#[allow(non_snake_case)]
#[cfg(test)]
fn do_test_LineReader_count(data: &str, line_count: usize) {
    eprintln!("{}do_test_LineReader_count(…, {:?})", sn(), line_count);
    let blocksz: BlockSz = 64;
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let mut lr1 = new_LineReader(&path, blocksz);
    let bufnoraw = buffer_to_String_noraw(data.as_bytes());
    eprintln!("{}File {:?}", so(), bufnoraw);
    process_LineReader(&mut lr1);
    let lc = lr1.count_lines_processed();
    assert_eq!(line_count as u64, lc, "Expected {} count of lines, found {}", line_count, lc);
    eprintln!("{}{:?}", so(), data.as_bytes());
    eprintln!("{}do_test_LineReader_count()", sx());
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count0() {
    do_test_LineReader_count("", 0);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1_() {
    do_test_LineReader_count(" ", 1);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1__() {
    do_test_LineReader_count("  ", 1);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1_n() {
    do_test_LineReader_count(" \n", 1);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2_n_() {
    do_test_LineReader_count(" \n ", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2__n__() {
    do_test_LineReader_count("  \n  ", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2_n_n() {
    do_test_LineReader_count(" \n \n", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2__n__n() {
    do_test_LineReader_count("  \n  \n", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count3_n_n_() {
    do_test_LineReader_count(" \n \n ", 3);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count3__n__n__() {
    do_test_LineReader_count("  \n  \n  ", 3);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count3__n__n__n() {
    do_test_LineReader_count("  \n  \n  \n", 3);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1() {
    do_test_LineReader_count("  \n  \n  \n  ", 4);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count4__n__n_n__n() {
    do_test_LineReader_count("  \n  \n  \n  \n", 4);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count4_uhi_n__n__n__n() {
    do_test_LineReader_count("two unicode points é\n  \n  \n  \n", 4);
}

// -------------------------------------------------------------------------------------------------

/// testing helper
/// call `LineReader.find_line` for all `FileOffset` in passed `offsets`
#[cfg(test)]
fn find_line_all(linereader: &mut LineReader, offsets: &Vec::<FileOffset>) {
    for fo1 in offsets {
        eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = linereader.find_line(*fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = linereader.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = linereader.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
            }
            ResultS4_LineFind::Done => {
                eprintln!("{}ResultS4_LineFind::Done!", so());
            }
            ResultS4_LineFind::Err(err) => {
                eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                panic!("ERROR: find_line({:?}) {:?}", fo1, err);
            }
        }
    }
}

/// compare contents of a file `path` to contents of `linereader`
/// assert they are the same
/// presumes the linereader has processed the entire file
#[cfg(test)]
fn compare_file_linereader(path: &FPath, linereader: &LineReader) {
    eprintln!("{}_compare_file_linereader({:?})", sn(), path);
    let contents_file: String = std::fs::read_to_string(path).unwrap();
    let contents_file_count: usize = contents_file.lines().count();
    eprint_file(path);
    let mut contents_lr: String = String::with_capacity(0);
    linereader.copy_all_lines(&mut contents_lr);
    eprintln!(
        "{}contents_lr ({} lines processed):\n{}{:?}\n",
        so(), linereader.count_lines_processed(), so(), contents_lr,
    );
    let mut i: usize = 0;
    for lines_file_lr1 in contents_file.lines().zip(contents_lr.lines()) {
        i += 1;
        eprintln!(
            "{}compare {}\n{}{:?}\n{}{:?}\n",
            so(), i, so(), lines_file_lr1.0, so(), lines_file_lr1.1,
        );
        assert_eq!(
            lines_file_lr1.0, lines_file_lr1.1,
            "Lines {:?} differ\nFile      : {:?}\nLineReader: {:?}\n",
            i, lines_file_lr1.0, lines_file_lr1.1,
        );
    }
    assert_eq!(
        contents_file_count, i, "Expected to compare {} lines, only compared {}",
        contents_file_count, i
    );
    eprintln!("{}_compare_file_linereader({:?})", sx(), &path);
}

/// test `LineReader::find_line` read all file offsets
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_all(path: &FPath, cache_enabled: bool, blocksz: BlockSz) {
    stack_offset_set(None);
    eprintln!("{}_test_LineReader_all({:?}, {:?})", sn(), path, blocksz);
    eprint_file(path);
    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_all = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_all);
    eprintln!("{}offsets_all: {:?}", so(), offsets_all);
    find_line_all(&mut lr1, &offsets_all);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_all({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_all0_empty() {;
    _test_LineReader_all(&NTF_empty0_path, true, 0x4);
}

#[test]
fn test_LineReader_all1() {
    let data: &str = "\
test_LineReader_all1 line 1";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}


#[test]
fn test_LineReader_all1n() {
    let data: &str = "\
test_LineReader_all1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all2() {
    let data: &str = "\
test_LineReader_all2 line 1
test_LineReader_all2 line 2";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, true, 0xFF);
}


#[test]
fn test_LineReader_all2n() {
    let data: &str = "\
test_LineReader_all2n line 1
test_LineReader_all2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all2n_noLRUcache() {
    let data: &str = "\
test_LineReader_all2n line 1
test_LineReader_all2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, false, 0x4);
}

#[test]
fn test_LineReader_all3_empty() {
    _test_LineReader_all(&NTF_nl_3_path, true, 0x4);
}

#[test]
fn test_LineReader_all3() {
    let data: &str = "\
test_LineReader_all3 line 1
test_LineReader_all3 line 2
test_LineReader_all3 line 3";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all3_noLRUcache() {
    let data: &str = "\
test_LineReader_all3 line 1
test_LineReader_all3 line 2
test_LineReader_all3 line 3";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, false, 0x4);
}

#[test]
fn test_LineReader_all3n() {
    let data: &str = "\
test_LineReader_all3n line 1
test_LineReader_all3n line 2
test_LineReader_all3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all3n_noLRUcache() {
    let data: &str = "\
test_LineReader_all3n line 1
test_LineReader_all3n line 2
test_LineReader_all3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all(&fpath, false, 0x4);
}

/// test `LineReader::find_line` read all file offsets but in reverse
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_all_reversed(path: &FPath, cache_enabled: bool, blocksz: BlockSz) {
    eprintln!("{}_test_LineReader_all_reversed({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(path, blocksz);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_all_rev = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_all_rev);
    offsets_all_rev.sort_by(|a, b| b.cmp(a));

    eprintln!("{}offsets_all_rev: {:?}", so(), offsets_all_rev);
    find_line_all(&mut lr1, &offsets_all_rev);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_all_reversed({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_all_reversed0_empty() {
    _test_LineReader_all_reversed(&NTF_empty0_path, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed1() {
    let data: &str = "\
test_LineReader_all_reversed1 line 1";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all_reversed(&fpath, true, 0x4);
}


#[test]
fn test_LineReader_all_reversed1n() {
    let data: &str = "\
test_LineReader_all_reversed1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed2() {
    let data: &str = "\
test_LineReader_all_reversed2 line 1
test_LineReader_all_reversed2 line 2";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all_reversed(&fpath, true, 0xFF);
}


#[test]
fn test_LineReader_all_reversed2n() {
    let data: &str = "\
test_LineReader_all_reversed2n line 1
test_LineReader_all_reversed2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3_empty() {
    _test_LineReader_all_reversed(&NTF_nl_3_path, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3() {
    let data: &str = "\
test_LineReader_all_reversed3 line 1
test_LineReader_all_reversed3 line 2
test_LineReader_all_reversed3 line 3";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3n() {
    let data: &str = "\
test_LineReader_all_reversed3n line 1
test_LineReader_all_reversed3n line 2
test_LineReader_all_reversed3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3n_noLRUcache() {
    let data: &str = "\
test_LineReader_all_reversed3n line 1
test_LineReader_all_reversed3n line 2
test_LineReader_all_reversed3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_all_reversed(&fpath, false, 0x4);
}

/// test `LineReader::find_line` read all file offsets but only the even ones
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_half_even(path: &FPath, blocksz: BlockSz) {
    eprintln!("{}_test_LineReader_half_even({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(&path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_half_even = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_half_even);
    offsets_half_even.retain(|x| *x % 2 == 0);

    eprintln!("{}offsets_half: {:?}", so(), offsets_half_even);
    find_line_all(&mut lr1, &offsets_half_even);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_half_even({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_half_even_0_empty() {
    _test_LineReader_half_even(&NTF_empty0_path, 0x4);
}

#[test]
fn test_LineReader_half_even_1() {
    let data: &str = "\
test_LineReader_half_even_1 line 1";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x4);
}


#[test]
fn test_LineReader_half_even_1n() {
    let data: &str = "\
test_LineReader_half_even_1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_2() {
    let data: &str = "\
test_LineReader_half_even_2 line 1
test_LineReader_half_even_2 line 2";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0xFF);
}


#[test]
fn test_LineReader_half_even_2n() {
    let data: &str = "\
test_LineReader_half_even_2n line 1
test_LineReader_half_even_2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_4_sparse1_0x4() {
    let data: &str = "a\nb\nc\nd";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_4_sparse1_0x2() {
    let data: &str = "a\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x2);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x4() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x6() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x6);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x8() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x8);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0xA() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0xA);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x2() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x2);
}

#[test]
fn test_LineReader_half_even_3() {
    let data: &str = "\
test_LineReader_half_even_3 line 1
test_LineReader_half_even_3 line 2
test_LineReader_half_even_3 line 3";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_3n() {
    let data: &str = "\
test_LineReader_half_even_3n line 1
test_LineReader_half_even_3n line 2
test_LineReader_half_even_3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_even(&fpath, 0x4);
}

/// test `LineReader::find_line` read all file offsets but only the even ones
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_half_odd(path: &FPath, blocksz: BlockSz) {
    eprintln!("{}_test_LineReader_half_odd({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_half_odd = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_half_odd);
    offsets_half_odd.retain(|x| *x % 2 != 0);

    eprintln!("{}offsets_half: {:?}", so(), offsets_half_odd);
    find_line_all(&mut lr1, &offsets_half_odd);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_half_odd({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_half_odd_0_empty() {
    _test_LineReader_half_odd(&NTF_empty0_path, 0x4);
}

#[test]
fn test_LineReader_half_odd_1() {
    let data: &str = "\
test_LineReader_half_odd_1 line 1";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_odd(&fpath, 0x4);
}


#[test]
fn test_LineReader_half_odd_1n() {
    let data: &str = "\
test_LineReader_half_odd_1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_2() {
    let data: &str = "\
test_LineReader_half_odd_2 line 1
test_LineReader_half_odd_2 line 2";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_odd(&fpath, 0xFF);
}


#[test]
fn test_LineReader_half_odd_2n() {
    let data: &str = "\
test_LineReader_half_odd_2n line 1
test_LineReader_half_odd_2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_3_sparse1() {
    let data: &str = "a\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_3() {
    let data: &str = "\
test_LineReader_half_odd_3 line 1
test_LineReader_half_odd_3 line 2
test_LineReader_half_odd_3 line 3";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_3n() {
    let data: &str = "\
test_LineReader_half_odd_3n line 1
test_LineReader_half_odd_3n line 2
test_LineReader_half_odd_3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_half_odd(&fpath, 0x4);
}

/// test `LineReader::find_line` read all file offsets but in random order
/// TODO: `randomize` should be predictable
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_rand(path: &FPath, blocksz: BlockSz) {
    stack_offset_set(None);
    eprintln!("{}_test_LineReader_rand({:?}, {:?})", sn(), &path, blocksz);

    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    offsets_rand.insert(0, 0);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    find_line_all(&mut lr1, &offsets_rand);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_rand({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_rand0_empty() {
    _test_LineReader_rand(&NTF_empty0_path, 0x4);
}

#[test]
fn test_LineReader_rand1() {
    let data: &str = "\
test_LineReader_rand1 line 1";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_rand(&fpath, 0x4);
}


#[test]
fn test_LineReader_rand1n() {
    let data: &str = "\
test_LineReader_rand1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand2() {
    let data: &str = "\
test_LineReader_rand2 line 1
test_LineReader_rand2 line 2";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_rand(&fpath, 0xFF);
}


#[test]
fn test_LineReader_rand2n() {
    let data: &str = "\
test_LineReader_rand2n line 1
test_LineReader_rand2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand3_nl3() {
    _test_LineReader_rand(&NTF_nl_3_path, 0x4);
}

#[test]
fn test_LineReader_rand3() {
    let data: &str = "\
test_LineReader_rand3 line 1
test_LineReader_rand3 line 2
test_LineReader_rand3 line 3";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand3n() {
    let data: &str = "\
test_LineReader_rand3n line 1
test_LineReader_rand3n line 2
test_LineReader_rand3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_LineReader_rand(&fpath, 0x4);
}

// -------------------------------------------------------------------------------------------------

/// test `LineReader::find_line` read all file offsets in a precise order
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_precise_order(path: &FPath, cache_enabled: bool, blocksz: BlockSz, offsets: &Vec::<FileOffset>) {
    stack_offset_set(None);
    eprintln!("{}_test_LineReader_rand({:?}, {:?}, {:?})", sn(), &path, blocksz, offsets);
    eprint_file(path);
    let mut lr1: LineReader = new_LineReader(path, blocksz);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }

    find_line_all(&mut lr1, offsets);

    eprintln!("\n{}{:?}\n", so(), lr1);
    for (fo, linep) in lr1.lines.iter() {
        eprintln!("{}  Line@{:02}: {:?}", so(), fo, linep);
        for linepart in (*linep).lineparts.iter() {
            eprintln!("{}    LinePart: {:?} {:?}", so(), linepart, linepart.to_String_noraw());
        }
    }

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_rand({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_precise_order_2__0_44__0xF() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1 of 2
test_LineReader_precise_order_2 line 2 of 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let offsets: Vec::<FileOffset> = vec![0, 44];
    _test_LineReader_precise_order(&fpath, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_2__0_44__0xFF() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1 of 2
test_LineReader_precise_order_2 line 2 of 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let offsets: Vec::<FileOffset> = vec![0, 44];
    _test_LineReader_precise_order(&fpath, true, 0xFF, &offsets);
}

#[test]
fn test_LineReader_precise_order_2__44_0() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1 of 2
test_LineReader_precise_order_2 line 2 of 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let offsets: Vec::<FileOffset> = vec![44, 0];
    _test_LineReader_precise_order(&fpath, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty0__0_1() {
    let offsets: Vec::<FileOffset> = vec![0, 1];
    _test_LineReader_precise_order(&NTF_empty0_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl1__0_1() {
    let offsets: Vec::<FileOffset> = vec![0, 1];
    _test_LineReader_precise_order(&NTF_nl_1_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_1_2() {
    let offsets: Vec::<FileOffset> = vec![0, 1, 2];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_2_1() {
    let offsets: Vec::<FileOffset> = vec![0, 2, 1];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_2_0() {
    let offsets: Vec::<FileOffset> = vec![1, 2, 0];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_0_2() {
    let offsets: Vec::<FileOffset> = vec![1, 0, 2];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__2_0_1() {
    let offsets: Vec::<FileOffset> = vec![2, 0, 1];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__2_1_0() {
    let offsets: Vec::<FileOffset> = vec![2, 1, 0];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_0_2_1_2() {
    let offsets: Vec::<FileOffset> = vec![1, 0, 2, 1, 2];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_2_1_2_0() {
    let offsets: Vec::<FileOffset> = vec![1, 2, 1, 2, 0];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_1_2_2() {
    let offsets: Vec::<FileOffset> = vec![0, 1, 2, 2];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_2_1_1() {
    let offsets: Vec::<FileOffset> = vec![0, 2, 1, 1];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_2_0_0() {
    let offsets: Vec::<FileOffset> = vec![1, 2, 0, 0];
    _test_LineReader_precise_order(&NTF_nl_3_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__0_1_2_3() {
    let offsets: Vec::<FileOffset> = vec![0, 1, 2, 3];
    _test_LineReader_precise_order(&NTF_nl_4_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__1_2_3_0() {
    let offsets: Vec::<FileOffset> = vec![1, 2, 3, 0];
    _test_LineReader_precise_order(&NTF_nl_4_path, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__2_3_0_1() {
    let offsets: Vec::<FileOffset> = vec![2, 3, 0, 1];
    _test_LineReader_precise_order(&NTF_nl_4_path, true, 0xF, &offsets);
}


#[test]
fn test_LineReader_precise_order_nl4__3_0_1_2() {
    let offsets: Vec::<FileOffset> = vec![3, 0, 1, 2];
    _test_LineReader_precise_order(&NTF_nl_4_path, true, 0xF, &offsets);
}


#[test]
fn test_LineReader_precise_order_nl4__3_0_1_2_3_0_1_2__noLRUcache() {
    let offsets: Vec::<FileOffset> = vec![3, 0, 1, 2, 3, 0, 1, 2];
    _test_LineReader_precise_order(&NTF_nl_4_path, false, 0xF, &offsets);
}


#[test]
fn test_LineReader_precise_order_3__0_88_44() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let offsets: Vec::<FileOffset> = vec![0, 88, 44];
    _test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

#[test]
fn test_LineReader_precise_order_3__0_100_50() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let offsets: Vec::<FileOffset> = vec![0, 100, 50];
    _test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

#[test]
fn test_LineReader_precise_order_3__50_0_100() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let offsets: Vec::<FileOffset> = vec![50, 0, 100];
    _test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

#[test]
fn test_LineReader_precise_order_3__100_50_0() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let offsets: Vec::<FileOffset> = vec![100, 50, 0];
    _test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

// -------------------------------------------------------------------------------------------------

/// call `LineReader.find_line_in_block` for all `FileOffset` in passed `offsets`
#[cfg(test)]
fn find_line_in_block_all(linereader: &mut LineReader, offsets: &Vec::<FileOffset>) {
    for fo1 in offsets {
        eprintln!("{}LineReader.find_line_in_block({})", so(), fo1);
        let result = linereader.find_line_in_block(*fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = linereader.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = linereader.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
            }
            ResultS4_LineFind::Done => {
                eprintln!("{}ResultS4_LineFind::Done!", so());
            }
            ResultS4_LineFind::Err(err) => {
                eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                panic!("ERROR: find_line({:?}) {:?}", fo1, err);
            }
        }
    }
}

/// test `LineReader::find_line_in_block` read all file offsets
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_find_line_in_block_all(path: &FPath, cache_enabled: bool, blocksz: BlockSz) {
    stack_offset_set(None);
    eprintln!("{}_test_find_line_in_block_all({:?}, {:?})", sn(), path, blocksz);
    eprint_file(path);
    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_all = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_all);
    eprintln!("{}offsets_all: {:?}", so(), offsets_all);
    find_line_in_block_all(&mut lr1, &offsets_all);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_find_line_in_block_all({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_find_line_in_block_all_empty0() {
    _test_find_line_in_block_all(&NTF_empty0_path, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_empty0_nocache() {
    _test_find_line_in_block_all(&NTF_empty0_path, false, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl1() {
    _test_find_line_in_block_all(&NTF_nl_1_path, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl2() {
    _test_find_line_in_block_all(&NTF_nl_2_path, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl3() {
    _test_find_line_in_block_all(&NTF_nl_3_path, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl4() {
    _test_find_line_in_block_all(&NTF_nl_4_path, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl5() {
    _test_find_line_in_block_all(&NTF_nl_5_path, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl2_2() {
    _test_find_line_in_block_all(&NTF_nl_2_path, true, 2);
}

#[test]
fn test_find_line_in_block_all_nl3_2() {
    _test_find_line_in_block_all(&NTF_nl_3_path, true, 2);
}

#[test]
fn test_find_line_in_block_all_nl4_2() {
    _test_find_line_in_block_all(&NTF_nl_4_path, true, 2);
}

#[test]
fn test_find_line_in_block_all_nl5_2() {
    _test_find_line_in_block_all(&NTF_nl_5_path, true, 2);
}

#[test]
fn test_find_line_in_block_all_nl5_4() {
    _test_find_line_in_block_all(&NTF_nl_5_path, true, 4);
}


#[test]
fn test_find_line_in_block_all_5_2() {
    let data: &str = "a\nb\nc\nd\ne\n";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_find_line_in_block_all(&fpath, true, 2);
}

#[test]
fn test_find_line_in_block_all_5_4() {
    let data: &str = "a\nb\nc\nd\ne\n";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    _test_find_line_in_block_all(&fpath, true, 4);
}

// -------------------------------------------------------------------------------------------------

type test_find_line_in_block_check = Vec<(FileOffset, ResultS4_LineFind_Test, String)>;

/// test `LineReader::find_line_in_block` reads passed file offsets
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_find_line_in_block(
    path: &FPath,
    cache_enabled: bool,
    blocksz: BlockSz,
    in_out: &test_find_line_in_block_check,
) {
    stack_offset_set(Some(2));
    eprintln!("{}_test_find_line_in_block({:?}, {:?}, {:?}, {:?})", sn(), &path, cache_enabled, blocksz, in_out);
    eprint_file(path);
    let mut lr1: LineReader = new_LineReader(path, blocksz);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }

    for (fo_in, rs4_expect, str_expect) in in_out.iter() {
        eprintln!("{}LineReader.find_line_in_block({})", so(), fo_in);
        let result = lr1.find_line_in_block(*fo_in);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = lr1.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                let str_actual = (*lp).to_String();
                assert_eq!(&str_actual, str_expect,
                    "find_line_in_block({})\nexpect {:?}\nactual {:?}\n", *fo_in, str_expect, str_actual,
                );
                assert_eq!(rs4_expect, &ResultS4_LineFind_Test::Found, "Expected {:?}, got Found", rs4_expect);
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = lr1.count_lines_processed();
                eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                let str_actual = (*lp).to_String();
                assert_eq!(&str_actual, str_expect,
                    "find_line_in_block({})\nexpect {:?}\nactual {:?}\n", *fo_in, str_expect, str_actual,
                );
                assert_eq!(rs4_expect, &ResultS4_LineFind_Test::Found_EOF, "Expected {:?}, got Found_EOF", rs4_expect);
            }
            ResultS4_LineFind::Done => {
                eprintln!("{}ResultS4_LineFind::Done!", so());
                assert_eq!(&"", &str_expect.as_str(),
                    "find_line_in_block({}) returned Done\nexpected {:?}\n", *fo_in, str_expect,
                );
                assert_eq!(rs4_expect, &ResultS4_LineFind_Test::Done, "Expected {:?}, got Done", rs4_expect);
            }
            ResultS4_LineFind::Err(err) => {
                eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                panic!("ERROR: find_line_in_block({:?}) {:?}", fo_in, err);
            }
        }
    }

    eprintln!("\n{}{:?}\n", so(), lr1);

    //for (fo, linep) in lr1.lines.iter() {
    //    eprintln!("{}  Line@{:02}: {:?}", so(), fo, linep);
    //    for linepart in (*linep).lineparts.iter() {
    //        eprintln!("{}    LinePart: {:?} {:?}", so(), linepart, linepart.to_String_noraw());
    //    }
    //}

    eprintln!("{}_test_find_line_in_block()", sx());
}

#[test]
fn test_find_line_in_block_empty0() {
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&NTF_empty0_path, true, 0xFF, &in_out);
}

#[test]
fn test_find_line_in_block_nl1() {
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found_EOF, String::from("\n"),),
        (1, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&NTF_nl_1_path, true, 0xFF, &in_out);
}

#[test]
fn test_find_line_in_block_nl2() {
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found, String::from("\n"),),
        (1, ResultS4_LineFind_Test::Found_EOF, String::from("\n"),),
        (2, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&NTF_nl_2_path, true, 0xFF, &in_out);
}

#[test]
fn test_find_line_in_block_1() {
    let data: &str = "abcdef";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found_EOF, String::from("abcdef"),),
        (1, ResultS4_LineFind_Test::Found_EOF, String::from("abcdef"),),
        (2, ResultS4_LineFind_Test::Found_EOF, String::from("abcdef"),),
        (3, ResultS4_LineFind_Test::Found_EOF, String::from("abcdef"),),
        (4, ResultS4_LineFind_Test::Found_EOF, String::from("abcdef"),),
        (5, ResultS4_LineFind_Test::Found_EOF, String::from("abcdef"),),
        (6, ResultS4_LineFind_Test::Done, String::from(""),),
        (7, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&fpath, true, 0xFF, &in_out);
}

#[test]
fn test_find_line_in_block_2() {
    let data: &str = "a\nb";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found, String::from("a\n"),),
        (1, ResultS4_LineFind_Test::Found, String::from("a\n"),),
        (2, ResultS4_LineFind_Test::Found_EOF, String::from("b"),),
        (3, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&fpath, true, 0xFF, &in_out);
}

#[test]
fn test_find_line_in_block_3() {
    let data: &str = "a\nb\nc";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found, String::from("a\n"),),
        (1, ResultS4_LineFind_Test::Found, String::from("a\n"),),
        (2, ResultS4_LineFind_Test::Found, String::from("b\n"),),
        (3, ResultS4_LineFind_Test::Found, String::from("b\n"),),
        (4, ResultS4_LineFind_Test::Found_EOF, String::from("c"),),
        (5, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&fpath, true, 0xFF, &in_out);
}

#[test]
fn test_find_line_in_block_4() {
    let data: &str = "a\nb\nc\nd\n";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found, String::from("a\n"),),
        (1, ResultS4_LineFind_Test::Found, String::from("a\n"),),
        (2, ResultS4_LineFind_Test::Found, String::from("b\n"),),
        (3, ResultS4_LineFind_Test::Found, String::from("b\n"),),
        (4, ResultS4_LineFind_Test::Found, String::from("c\n"),),
        (5, ResultS4_LineFind_Test::Found, String::from("c\n"),),
        (6, ResultS4_LineFind_Test::Found_EOF, String::from("d\n"),),
        (7, ResultS4_LineFind_Test::Found_EOF, String::from("d\n"),),
        (8, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&fpath, true, 0xFF, &in_out);
}

#[test]
fn test_find_line_in_block_4x2_2() {
    let data: &str = "abc\ndef\n";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Done, String::from(""),),
        (1, ResultS4_LineFind_Test::Done, String::from(""),),
        (2, ResultS4_LineFind_Test::Done, String::from(""),),
        (3, ResultS4_LineFind_Test::Done, String::from(""),),
        (4, ResultS4_LineFind_Test::Done, String::from(""),),
        (5, ResultS4_LineFind_Test::Done, String::from(""),),
        (6, ResultS4_LineFind_Test::Done, String::from(""),),
        (7, ResultS4_LineFind_Test::Done, String::from(""),),
        (8, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&fpath, true, 2, &in_out);
}

#[test]
fn test_find_line_in_block_4x2_3() {
    let data: &str = "abc\ndef\n";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Done, String::from(""),),
        (1, ResultS4_LineFind_Test::Done, String::from(""),),
        (2, ResultS4_LineFind_Test::Done, String::from(""),),
        (3, ResultS4_LineFind_Test::Done, String::from(""),),
        (4, ResultS4_LineFind_Test::Done, String::from(""),),
        (5, ResultS4_LineFind_Test::Done, String::from(""),),
        (6, ResultS4_LineFind_Test::Done, String::from(""),),
        (7, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&fpath, true, 3, &in_out);
}

#[test]
fn test_find_line_in_block_4x2_4() {
    let data: &str = "abc\ndef\n";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (1, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (2, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (3, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (4, ResultS4_LineFind_Test::Found_EOF, String::from("def\n"),),
        (5, ResultS4_LineFind_Test::Found_EOF, String::from("def\n"),),
        (6, ResultS4_LineFind_Test::Found_EOF, String::from("def\n"),),
        (7, ResultS4_LineFind_Test::Found_EOF, String::from("def\n"),),
    ];
    _test_find_line_in_block(&fpath, true, 4, &in_out);
}

#[test]
fn test_find_line_in_block_4x2_5() {
    let data: &str = "abc\ndef\n";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let in_out: test_find_line_in_block_check = vec![
        (0, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (1, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (2, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (3, ResultS4_LineFind_Test::Found, String::from("abc\n"),),
        (4, ResultS4_LineFind_Test::Done, String::from(""),),
        (5, ResultS4_LineFind_Test::Done, String::from(""),),
        (6, ResultS4_LineFind_Test::Done, String::from(""),),
        (7, ResultS4_LineFind_Test::Done, String::from(""),),
        (8, ResultS4_LineFind_Test::Done, String::from(""),),
    ];
    _test_find_line_in_block(&fpath, true, 5, &in_out);
}

// -------------------------------------------------------------------------------------------------

type test_Line_get_boxptrs_check = Vec<(FileOffset, (LineIndex, LineIndex), Bytes)>;

/// test `Line.get_boxptrs`
/// assert result equals passed `checks`
fn _test_Line_get_boxptrs(path: &FPath, blocksz: BlockSz, checks: &test_Line_get_boxptrs_check) {
    let fn_: &str = "_test_Line_get_boxptrs";
    eprintln!("{}{}({:?}, {}, checks)", sn(), fn_, path, blocksz);
    // create a `LineReader` and read all the lines in the file
    let mut lr = new_LineReader(path, blocksz);
    let mut fo: FileOffset = 0;
    loop {
        match lr.find_line(fo) {
            ResultS4_LineFind::Found((fo_, _)) => {
                fo = fo_;
            },
            ResultS4_LineFind::Found_EOF((fo_, _)) => {
                fo = fo_;
            },
            ResultS4_LineFind::Done => {
                break;
            },
            ResultS4_LineFind::Err(err) => {
                panic!("LineReader::new({:?}, {:?}) ResultS4_LineFind::Err {}", path, blocksz, err);
            },
        }
    }

    // then test the `Line.get_boxptrs`
    for (linenum, (a, b), bytes_check) in checks.iter() {
        assert_lt!(a, b, "bad check args a {} b {}", a, b);
        assert_ge!(b-a, bytes_check.len(), "Bad check args ({}-{})={} < {} bytes_check.len()", b, a, b-a, bytes_check.len());
        eprintln!("{}{}: linereader.get_linep({})", so(), fn_, linenum);
        let line = lr.get_linep(linenum).unwrap();
        eprintln!("{}{}: returned {:?}", so(), fn_, line.to_String_noraw());
        eprintln!("{}{}: line.get_boxptrs({}, {})", so(), fn_, a, b);
        let boxptrs = match line.get_boxptrs(*a, *b) {
            enum_BoxPtrs::NoPtr => {
                assert!(bytes_check.is_empty(), "Expected bytes_check {:?}, received NoPtr (no bytes)", bytes_check);
                continue;
            }
            enum_BoxPtrs::SinglePtr(box_) => {
                vec![box_,]
            },
            enum_BoxPtrs::DoublePtr(box2) => {
                vec![box2.0, box2.1,]
            },
            enum_BoxPtrs::MultiPtr(boxes) => {
                boxes
            },
        };
        let mut at: usize = 0;
        for boxptr in boxptrs.iter() {
            for byte_ in (*boxptr).iter() {
                let byte_check = &bytes_check[at];
                eprintln!("{}{}: {:3?} ≟ {:3?} ({:?} ≟ {:?})", so(), fn_, byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check));
                assert_eq!(byte_, byte_check, "byte {} from boxptr {:?} ≠ {:?} ({:?} ≠ {:?}) check value; returned boxptr segement {:?} Line {:?}", at, byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check), buffer_to_String_noraw(&(*boxptr)), line.to_String_noraw());
                at += 1;
            }
        }
    }
    eprintln!("{}{}", sx(), fn_);
}

#[test]
fn test_Line_get_boxptrs_1() {
    let data: &str = "\
this is line 1";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let checks: test_Line_get_boxptrs_check = vec![
        (0, (0, 1), vec![b't']),
    ];
    _test_Line_get_boxptrs(&fpath, 0xFF, &checks);
}

/// for given `blocksz`, get `LineReader.get_boxptrs` for a predetermined
/// inputs and assert the comparison to `checks` outputs
#[cfg(test)]
fn _test_Line_get_boxptrs_2_(blocksz: BlockSz) {
    eprintln!("{}_test_Line_get_boxptrs_2_({:?})", sn(), blocksz);
    let data: &str = "\
One 1
Two 2
";
    let ntf = create_temp_file(data);
    let fpath = NTF_Path(&ntf);
    let checks: test_Line_get_boxptrs_check = vec![
        // fileoffset, (a, b), check
        (6, (0, 1), vec![b'T',]),
        (6, (0, 2), vec![b'T', b'w']),
        (7, (0, 2), vec![b'T', b'w']),
        (7, (0, 5), vec![b'T', b'w', b'o', b' ', b'2']),
        (8, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (8, (0, 7), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (9, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (10, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (10, (1, 6), vec![b'w', b'o', b' ', b'2', b'\n']),
        (10, (2, 6), vec![b'o', b' ', b'2', b'\n']),
        (10, (3, 6), vec![b' ', b'2', b'\n']),
        (10, (4, 6), vec![b'2', b'\n']),
        (10, (5, 6), vec![b'\n']),
    ];
    _test_Line_get_boxptrs(&fpath, blocksz, &checks);
    eprintln!("{}_test_Line_get_boxptrs_2_({:?})", sx(), blocksz);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xF() {
    _test_Line_get_boxptrs_2_(0xF);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xE() {
    _test_Line_get_boxptrs_2_(0xE);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xD() {
    _test_Line_get_boxptrs_2_(0xD);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xC() {
    _test_Line_get_boxptrs_2_(0xC);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xB() {
    _test_Line_get_boxptrs_2_(0xB);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xA() {
    _test_Line_get_boxptrs_2_(0xA);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x9() {
    _test_Line_get_boxptrs_2_(0x9);
}


#[test]
fn test_Line_get_boxptrs_2_bsz_0x8() {
    _test_Line_get_boxptrs_2_(0x8);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x7() {
    _test_Line_get_boxptrs_2_(0x7);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x6() {
    _test_Line_get_boxptrs_2_(0x6);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x5() {
    _test_Line_get_boxptrs_2_(0x5);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x4() {
    _test_Line_get_boxptrs_2_(0x4);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x3() {
    _test_Line_get_boxptrs_2_(0x3);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x2() {
    _test_Line_get_boxptrs_2_(0x2);
}
