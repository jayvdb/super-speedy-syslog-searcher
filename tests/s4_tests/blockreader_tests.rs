// Readers/blockreader_tests.rs
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate s4lib;

use s4lib::common::{
    FileType,
};

use s4lib::Readers::filepreprocessor::{
    fpath_to_filetype_mimeguess,
    MimeGuess,
};

use s4lib::Readers::blockreader::{
    FPath,
    FileOffset,
    BlockSz,
    BlockReader,
    ResultS3_ReadBlock,
    printblock,
    SUBPATH_SEP,
};

use s4lib::printer_debug::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_with_name_exact,
    create_temp_file_with_suffix,
    create_temp_file_bytes_with_suffix,
    NTF_Path,
};

use s4lib::printer_debug::stack::{
    stack_offset_set,
};

extern crate lazy_static;
use lazy_static::lazy_static;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper wrapper to create a new BlockReader
fn new_BlockReader(path: FPath, blocksz: BlockSz) -> BlockReader {
    stack_offset_set(Some(2));
    let (filetype, _mimeguess) = fpath_to_filetype_mimeguess(&path);
    match BlockReader::new(path.clone(), filetype, blocksz) {
        Ok(br) => {
            eprintln!("opened {:?}", path);
            eprintln!("new {:?}", &br);
            br
        },
        Err(err) => {
            panic!("ERROR: BlockReader.open({:?}, {}) {}", path, blocksz, err);
        },
    }
}

/// helper wrapper to create a new BlockReader
fn new_BlockReader2(path: FPath, blocksz: BlockSz, filetype: FileType) -> BlockReader {
    stack_offset_set(Some(2));
    match BlockReader::new(path.clone(), filetype, blocksz) {
        Ok(br) => {
            eprintln!("opened {:?}", path);
            eprintln!("new {:?}", &br);
            br
        },
        Err(err) => {
            panic!("ERROR: BlockReader.open({:?}, {}) {}", path, blocksz, err);
        },
    }
}

// -------------------------------------------------------------------------------------------------

/// quick and dirty test of basic test of BlockReader things
///
/// TODO: improve this: add proper checking with `assert`, allow other inputs
#[allow(non_snake_case)]
fn test_BlockReader(path: &FPath, blocksz: BlockSz) {
    eprintln!("test_BlockReader({:?}, {})", path, blocksz);
    let mut br1 = new_BlockReader(path.clone(), blocksz);
    let last_blk = BlockReader::block_offset_at_file_offset(br1.filesz(), blocksz);
    for offset in [0, 1, 5, 1, 99, 1, last_blk].iter() {
        {
            let rbp = br1.read_block(*offset);
            match rbp {
                ResultS3_ReadBlock::Found(val) => {
                    let boff: FileOffset = BlockReader::file_offset_at_block_offset(*offset, blocksz);
                    printblock(val.as_ref(), *offset, boff, blocksz, String::new());
                },
                ResultS3_ReadBlock::Done => {
                    continue;
                },
                ResultS3_ReadBlock::Err(err) => {
                    panic!("ERROR: blockreader.read({}) error {}", offset, err);
                }
            };
        }
    }
    eprintln!("after reads {:?}", &br1);
    // TODO: need to compare results to expected Block values
}

// -------------------------------------------------------------------------------------------------

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_EMPTY0: NamedTempFile = create_temp_file("");
    static ref NTF_EMPTY0_path: FPath = NTF_Path(&NTF_EMPTY0);
    static ref NTF_NL_1: NamedTempFile = create_temp_file("\n");
    static ref NTF_NL_1_PATH: FPath = NTF_Path(&NTF_NL_1);
    static ref NTF_basic_basic_dt10: NamedTempFile = create_temp_file(
"2000-01-01 00:00:01 1
2000-01-01 00:00:02 1
2000-01-01 00:00:02 2
2000-01-01 00:00:03 1
2000-01-01 00:00:03 2
2000-01-01 00:00:03 3
2000-01-01 00:00:04 1
2000-01-01 00:00:04 2
2000-01-01 00:00:04 3
2000-01-01 00:00:04 4
2000-01-01 00:00:05 1
2000-01-01 00:00:05 2
2000-01-01 00:00:05 3
2000-01-01 00:00:05 4
2000-01-01 00:00:05 5
2000-01-01 00:00:06 1
2000-01-01 00:00:06 2
2000-01-01 00:00:06 3
2000-01-01 00:00:06 4
2000-01-01 00:00:06 5
2000-01-01 00:00:06 6
2000-01-01 00:00:07 1
2000-01-01 00:00:07 2
2000-01-01 00:00:07 3
2000-01-01 00:00:07 4
2000-01-01 00:00:07 5
2000-01-01 00:00:07 6
2000-01-01 00:00:07 7
2000-01-01 00:00:08 1
2000-01-01 00:00:08 2
2000-01-01 00:00:08 3
2000-01-01 00:00:08 4
2000-01-01 00:00:08 5
2000-01-01 00:00:08 6
2000-01-01 00:00:08 7
2000-01-01 00:00:08 8
2000-01-01 00:00:09 1
2000-01-01 00:00:09 2
2000-01-01 00:00:09 3
2000-01-01 00:00:09 4
2000-01-01 00:00:09 5
2000-01-01 00:00:09 6
2000-01-01 00:00:09 7
2000-01-01 00:00:09 8
2000-01-01 00:00:09 9
2000-01-01 00:00:10 1
2000-01-01 00:00:10 2
2000-01-01 00:00:10 3
2000-01-01 00:00:10 4
2000-01-01 00:00:10 5
2000-01-01 00:00:10 6
2000-01-01 00:00:10 7
2000-01-01 00:00:10 8
2000-01-01 00:00:10 9
2000-01-01 00:00:10 10"
    );
    static ref NTF_basic_basic_dt10_path: FPath = NTF_Path(&NTF_basic_basic_dt10);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_BlockReader1() {
    test_BlockReader(&NTF_basic_basic_dt10_path, 2);
}

// TODO: [2022/04] add more tests

// -------------------------------------------------------------------------------------------------

/// quick self-test
#[test]
fn test_count_blocks() {
    eprintln!("test_count_blocks()");
    assert_eq!(1, BlockReader::count_blocks(1, 1));
    assert_eq!(2, BlockReader::count_blocks(2, 1));
    assert_eq!(3, BlockReader::count_blocks(3, 1));
    assert_eq!(4, BlockReader::count_blocks(4, 1));
    assert_eq!(1, BlockReader::count_blocks(1, 2));
    assert_eq!(1, BlockReader::count_blocks(2, 2));
    assert_eq!(2, BlockReader::count_blocks(3, 2));
    assert_eq!(2, BlockReader::count_blocks(4, 2));
    assert_eq!(3, BlockReader::count_blocks(5, 2));
    assert_eq!(1, BlockReader::count_blocks(1, 3));
    assert_eq!(1, BlockReader::count_blocks(2, 3));
    assert_eq!(1, BlockReader::count_blocks(3, 3));
    assert_eq!(2, BlockReader::count_blocks(4, 3));
    assert_eq!(1, BlockReader::count_blocks(1, 4));
    assert_eq!(1, BlockReader::count_blocks(4, 4));
    assert_eq!(2, BlockReader::count_blocks(5, 4));
    assert_eq!(1, BlockReader::count_blocks(4, 5));
    assert_eq!(1, BlockReader::count_blocks(5, 5));
    assert_eq!(2, BlockReader::count_blocks(6, 5));
    assert_eq!(2, BlockReader::count_blocks(10, 5));
    assert_eq!(3, BlockReader::count_blocks(11, 5));
    assert_eq!(3, BlockReader::count_blocks(15, 5));
    assert_eq!(4, BlockReader::count_blocks(16, 5));
}

/// quick self-test
#[test]
fn test_file_offset_at_block_offset() {
    eprintln!("test_file_offset_at_block_offset()");
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
#[test]
fn test_block_offset_at_file_offset() {
    eprintln!("test_block_offset_at_file_offset()");
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
#[test]
fn test_block_index_at_file_offset() {
    eprintln!("test_block_index_at_file_offset()");
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
#[test]
fn test_file_offset_at_block_offset_index() {
    eprintln!("test_file_offset_at_block_offset_index()");
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