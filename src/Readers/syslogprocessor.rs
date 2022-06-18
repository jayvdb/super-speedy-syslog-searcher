// Readers/syslogprocessor.rs
//
// …

use crate::common::{
    FPath,
    FileOffset,
    FileProcessingResult,
    FileType,
    SYSLOG_SZ_MAX,
};

use crate::Readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockSz,
    BlockP,
    ResultS3_ReadBlock,
};

use crate::printer::printers::{
    Color,
    ColorSpec,
    WriteColor,
};

use crate::dbgpr::stack::{
    sn,
    snx,
    so,
    sx,
};

use crate::Data::datetime::{
    FixedOffset,
    DateTimeL,
    DateTimeL_Opt,
};

use crate::Data::line::{
    BlockOffsets,
};

pub use crate::Readers::linereader::{
    ResultS4_LineFind,
};

pub use crate::Readers::syslinereader::{
    ResultS4_SyslineFind,
    Sysline,
    SyslineP,
    SyslineReader,
};

use crate::Readers::summary::{
    Summary,
};

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io::{
    Error,
    Result,
    ErrorKind,
};
use std::sync::Arc;

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate mime_sniffer;
use mime_sniffer::MimeTypeSniffer;  // adds extension method `sniff_mime_type` to `[u8]`

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
};

extern crate rangemap;
use rangemap::RangeMap;

extern crate static_assertions;
use static_assertions::{
    const_assert,
};

extern crate walkdir;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogProcessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub type FileProcessingResult_BlockZero = FileProcessingResult<std::io::Error>;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProcessingMode {
    /// does the file exist?
    stage0_valid_file_check,
    /// check file can be parsed
    stage1_blockzero_analysis,
    /// find the sysline with datetime that is allowed by the datetime filters
    stage2_find_dt,
    /// no more searching backwards in a file, and thus, previously processed data can be dropped
    stage3_stream_syslines,
    /// for CLI option --summary, print a summary about the file processing
    stage4_summary,
}

type BszRange = std::ops::Range<BlockSz>;
type Map_BszRange_To_Count = RangeMap<u64, u64>;

lazy_static! {
    // for files in blockzero_analyis, the number `Line` needed to found within
    // block zero will vary depending on the blocksz
    pub static ref BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP: Map_BszRange_To_Count = {
        let mut m = Map_BszRange_To_Count::new();
        m.insert(BszRange{start: 0, end: SYSLOG_SZ_MAX as BlockSz}, 1);
        m.insert(BszRange{start: SYSLOG_SZ_MAX as BlockSz, end: BlockSz::MAX}, 2);

        m
    };
    // for files in blockzero_analyis, the number `Sysline` needed to found within
    // block zero will vary depending on the blocksz
    pub static ref BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP: Map_BszRange_To_Count = {
        let mut m = Map_BszRange_To_Count::new();
        m.insert(BszRange{start: 0, end: SYSLOG_SZ_MAX as BlockSz}, 1);
        m.insert(BszRange{start: SYSLOG_SZ_MAX as BlockSz, end: BlockSz::MAX}, 2);

        m
    };
}

/// The `SyslogProcessor` uses `SyslineReader` to find `Sysline`s in a file.
/// 
/// A `SyslogProcessor` has knowledge of:
/// - the different stages of processing a syslog file
/// - stores optional datetime filters
///
/// A `SyslogProcessor` will manipulate the data stored by it's `SyslineReader`
/// and further underlying `LineReader` and `BlockReader`, i.e. during streaming
/// the `SyslogProcessor` will proactively `drop` data that has been processed
/// and printed.
pub struct SyslogProcessor {
    syslinereader: SyslineReader,
    processingmode: ProcessingMode,
    path: FPath,
    blocksz: BlockSz,
    tz_offset: FixedOffset,
    filter_dt_after_opt: DateTimeL_Opt,
    filter_dt_before_opt: DateTimeL_Opt,
    /// internal sanity check, has `self.blockzero_analysis()` completed?
    blockzero_analysis_done: bool,
    /// last `blockoffset` passed to `drop_block`
    drop_block_last: BlockOffset,
    /// internal cache: allocate this once (or at least, few times)
    /// instead of allocating one for every call to `drop_block_impl`
    drop_block_fo_keys: Vec<FileOffset>,
    /// internal stats for `drop` during "streaming"
    _drop_count_block: u64,
    _drop_count_linepart: u64,
    _drop_count_line: u64,
    _drop_count_sysline: u64,
}

impl std::fmt::Debug for SyslogProcessor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SyslogProcessor")
            .field("Path", &self.path)
            .field("Processing Mode", &self.processingmode)
            .field("BlockSz", &self.blocksz)
            .field("TimeOffset", &self.tz_offset)
            .field("filter_dt_after_opt", &self.filter_dt_after_opt)
            .field("filter_dt_before_opt", &self.filter_dt_before_opt)
            .field("BO Analysis done?", &self.blockzero_analysis_done)
            .field("filetype", &self.filetype())
            .field("MimeGuess", &self.mimeguess())
            .finish()
    }
}

impl SyslogProcessor {

    /// TODO: [2022/06/01] this should be predefined mapping of key range to value integer,
    ///       where blocksz keys to count of expected line.
    ///       e.g. blocksz [2, 64] expect 1 line, blocksz [64, 1024] expect 5 lines, etc.
    /// `SyslogProcessor::blockzero_analysis_lines` must find this many `Line` for the
    /// file to be considered a text file
    //pub (crate) const BLOCKZERO_ANALYSIS_LINE_COUNT: u64 = 15;

    /// `SyslogProcessor::blockzero_analysis_syslines` must find this many `Sysline` for the
    /// file to be considered a syslog file
    //pub (crate) const BLOCKZERO_ANALYSIS_SYSLINE_COUNT: u64 = 2;

    /// `SyslogProcessor` has it's own requirements for `BlockSz`
    /// Necessary for `blockzero_analysis` functions to have chance at success.
    const BLOCKSZ_MIN: BlockSz = 0x100;
    /// allow "streaming" (`drop`ping data in calls to `find_sysline`)?
    const STREAM_STAGE_DROP: bool = true;

    pub fn new(
        path: FPath,
        filetype: FileType,
        blocksz: BlockSz,
        tz_offset: FixedOffset,
        filter_dt_after_opt: DateTimeL_Opt,
        filter_dt_before_opt: DateTimeL_Opt,
    ) -> Result<SyslogProcessor> {
        debug_eprintln!("{}SyslogProcessor::new({:?}, {:?}, {:?}, {:?})", snx(), path, filetype, blocksz, tz_offset);
        if blocksz < SyslogProcessor::BLOCKSZ_MIN {
            return Result::Err(
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("BlockSz {0} (0x{0:08X}) is too small, SyslogProcessor has BlockSz minumum {1} (0x{1:08X})", blocksz, SyslogProcessor::BLOCKSZ_MIN)
                )
            );
        }
        let path_ = path.clone();
        let mut slr = match SyslineReader::new(path, filetype, blocksz, tz_offset) {
            Ok(val) => val,
            Err(err) => {
                return Result::Err(err);
            }
        };

        // XXX: 2022/06/15 experiemnt
        //slr.LRU_cache_disable();
        //slr.linereader.LRU_cache_disable();
        //slr.linereader.blockreader.LRU_cache_disable();

        let drop_block_fo_keys_sz: usize = std::cmp::max((blocksz as usize) / 0x100, 20);

        Result::Ok(
            SyslogProcessor {
                syslinereader: slr,
                processingmode: ProcessingMode::stage0_valid_file_check,
                path: path_,
                blocksz,
                tz_offset,
                filter_dt_after_opt,
                filter_dt_before_opt,
                blockzero_analysis_done: false,
                drop_block_last: 0,
                drop_block_fo_keys: Vec::<FileOffset>::with_capacity(drop_block_fo_keys_sz),
                _drop_count_block: 0,
                _drop_count_linepart: 0,
                _drop_count_line: 0,
                _drop_count_sysline: 0,
            }
        )
    }

    #[inline]
    pub fn lines_count(&self) -> u64 {
        self.syslinereader.linereader.lines_count
    }

    #[inline]
    pub const fn blocksz(&self) -> BlockSz {
        self.syslinereader.blocksz()
    }

    #[inline]
    pub const fn filesz(&self) -> u64 {
        self.syslinereader.filesz()
    }

    #[inline]
    pub const fn filetype(&self) -> FileType {
        self.syslinereader.filetype()
    }

    #[inline]
    pub const fn path(&self) -> &FPath {
        self.syslinereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub const fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.syslinereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub const fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.syslinereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub const fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.syslinereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub const fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.syslinereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub const fn file_blocks_count(&self) -> u64 {
        self.syslinereader.file_blocks_count()
    }

    /// last valid `BlockOffset` of the file
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.syslinereader.blockoffset_last()
    }

    /// smallest size character in bytes
    pub const fn charsz(&self) -> usize {
        self.syslinereader.charsz()
    }

    /// wrapper to `self.syslinereader.linereader.blockreader.mimeguess`
    pub const fn mimeguess(&self) -> MimeGuess {
        self.syslinereader.mimeguess()
    }

    /// wrapper to `self.syslinereader.find_sysline`
    ///
    /// This is where data is `drop`ped during streaming stage.
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        if self.processingmode == ProcessingMode::stage3_stream_syslines && SyslogProcessor::STREAM_STAGE_DROP {
            debug_eprintln!("{}syslogprocesser.find_sysline({})", sn(), fileoffset);
            // if processing mode is `stage3_stream_syslines`
            // then any prior processed syslines (and underlying data `Line`, `Block`, etc.)
            // can be dropped.
            let result: ResultS4_SyslineFind =
                self.syslinereader.find_sysline(fileoffset);
            match result {
                ResultS4_SyslineFind::Found((ref _fo, ref syslinep))
                | ResultS4_SyslineFind::Found_EOF((ref _fo, ref syslinep)) =>
                {
                    let bo_first = (*syslinep).blockoffset_first();
                    if bo_first > 0 {
                        self.drop_block(bo_first - 1);
                    }
                }
                ResultS4_SyslineFind::Done => {}
                ResultS4_SyslineFind::Err(ref _err) => {}
            }
            return result;
        }
        debug_eprintln!("{}syslogprocesser.find_sysline({})", snx(), fileoffset);

        self.syslinereader.find_sysline(fileoffset)
    }

    /// wrapper to `self.syslinereader.is_sysline_last`
    pub(crate) fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        self.syslinereader.is_sysline_last(syslinep)
    }

    /// drop all data at and before `blockoffset` (drop as much as possible)
    /// this includes underyling `Block`, `LineParts`, `Line`, `Sysline`
    ///
    /// Presumes the caller knows what they are doing!
    fn drop_block(&mut self, blockoffset: BlockOffset) {
        // `drop_block_impl` is an expensive function. only run it when needed
        if blockoffset <= self.drop_block_last {
            debug_eprintln!("{}syslogprocesser.drop_block({}) skip", snx(), blockoffset);
            return;
        }
        self.drop_block_last = blockoffset;

        self.drop_block_impl(blockoffset)
    }

    fn drop_block_impl(&mut self, blockoffset: BlockOffset) {
        debug_eprintln!("{}syslogprocesser.drop_block({})", sn(), blockoffset);
        debug_assert!(SyslogProcessor::STREAM_STAGE_DROP, "STREAM_STAGE_DROP is false yet call to drop_block");

        // TODO: move this loop into function `SyslineReader::drop_block`.
        // XXX: using `sylines.value_mut()` would be cleaner.
        //      But `sylines.value_mut()` causes a clone of the `SyslineP`, which then
        //      increments the `Arc` "strong_count". That in turn prevents `Arc::get_mut(&SyslineP)`
        //      from returning the original `Sysline`.
        //      Instead of `syslines.values_mut()`, use `syslines.keys()` and then `syslines.get_mut`
        //      to get a `&SyslineP`. This does not increase the "strong_count".

        self.drop_block_fo_keys.clear();
        for fo_key in self.syslinereader.syslines.keys() {
            self.drop_block_fo_keys.push(*fo_key);
        }
        //let fo_keys: Vec<FileOffset> = self.syslinereader.syslines.keys().copied().collect();
        debug_eprintln!("{}syslogprocesser.drop_block: collected keys {:?}", so(), self.drop_block_fo_keys);
        // sanity check assumption
        if cfg!(debug_assertions) {
            let mut fo_last: &FileOffset = &0;
            for fo_key in self.drop_block_fo_keys.iter() {
                assert_le!(fo_last, fo_key, "Collected keys were not in order {:?}", self.drop_block_fo_keys);
                fo_last = fo_key;
            }
        }
        let mut bo_dropped: HashSet<BlockOffset> = HashSet::<BlockOffset>::with_capacity(2);

        for fo_key in self.drop_block_fo_keys.iter() {
            let bo_last = self.syslinereader.syslines[fo_key].blockoffset_last();
            if bo_last > blockoffset {
                debug_eprintln!("{}syslogprocesser.drop_block: blockoffset_last {} > {} blockoffset, continue;", so(), bo_last, blockoffset);
                // presume all proceeding `Sysline.blockoffset_last()` will be after `blockoffset`
                break;
            }
            self.syslinereader.drop_sysline(*fo_key, &mut bo_dropped);
            debug_eprintln!("{}syslogprocesser.drop_block: bo_dropped {:?}", so(), bo_dropped);

            /*
            // XXX: must use `get_mut` then later `syslines.remove`
            //      cannot `syslines.remove` here becuase returned value is not `&mut`, only `&`
            //      and the call to `Arc::get_mut` emits rustc error about difference.
            let mut syslinep: SyslineP = match self.syslinereader.syslines.remove(fo_key) {
                Some(val) => val,
                None => {
                    debug_eprintln!("syslogprocesser.drop_block: syslines.remove({}) returned None which is unexpected", fo_key);
                    continue;
                }
            };
            let bo_last = (*syslinep).blockoffset_last();
            if bo_last > blockoffset {
                debug_eprintln!("{}syslogprocesser.drop_block: blockoffset_last {}; continue;", so(), bo_last);
                // presume all proceeding `Sysline.blockoffset_last()` will be after `blockoffset`
                break;
            }
            debug_eprintln!("{}syslogprocesser.drop_block: Processing SyslineP @[{}‥{}], Block @[{}‥{}] strong_count {}", so(), (*syslinep).fileoffset_begin(), (*syslinep).fileoffset_end(), (*syslinep).blockoffset_first(), (*syslinep).blockoffset_last(), Arc::strong_count(&syslinep));
            self.syslinereader._find_sysline_lru_cache.pop(&(*syslinep).fileoffset_begin());
            match Arc::try_unwrap(syslinep) {
                Ok(mut sysline) => {
                    debug_eprintln!("{}syslogprocesser.drop_block: Arc::try_unwrap(syslinep) Ok processing Sysline @[{}‥{}] Block @[{}‥{}]", so(), sysline.fileoffset_begin(), sysline.fileoffset_end(), sysline.blockoffset_first(), sysline.blockoffset_last());
                    // TODO: move this loop into function `Line::drop_block`.
                    for linep in sysline.lines.into_iter() {
                        self.syslinereader.linereader._find_line_lru_cache.pop(&(*linep).fileoffset_begin());
                        match Arc::try_unwrap(linep) {
                            Ok(mut line) => {
                                debug_eprintln!("{}syslogprocesser.drop_block: Arc::try_unwrap(linep) Ok processing Line @[{}‥{}] Block @[{}‥{}]", so(), line.fileoffset_begin(), line.fileoffset_end(), line.blockoffset_first(), line.blockoffset_last());
                                for bo in line.get_blockoffsets().iter() {
                                    if blockoffsets_dropped.contains(bo) {
                                        continue;
                                    }
                                    self.syslinereader.linereader.blockreader.drop_block(*bo);
                                    blockoffsets_dropped.insert(*bo);
                                }
                                debug_eprintln!("{}syslogprocesser.drop_block: Line.lineparts.clear()", so());
                                line.lineparts.clear();
                            },
                            Err(linep_) => {
                                debug_eprintln!("{}syslogprocesser.drop_block: Arc::try_unwrap(linep) Err, strong_count {}", so(), Arc::strong_count(&linep_));
                            }
                        }
                    }
                }
                Err(syslinep_) => {
                    debug_eprintln!("{}syslogprocesser.drop_block: Arc::try_unwrap(syslinep) Err strong_count {}", so(), Arc::strong_count(&syslinep_));
                }
            }
            */
        }
        debug_eprintln!("{}syslogprocesser.drop_block({})", sx(), blockoffset);
    }

    /// Wrapper for `self.syslinereader.find_sysline_between_datetime_filters`
    pub fn find_sysline_between_datetime_filters(
        &mut self, fileoffset: FileOffset,
    ) -> ResultS4_SyslineFind {
        debug_eprintln!("{}syslogprocesser.find_sysline_between_datetime_filters({})", snx(), fileoffset);

        self.syslinereader.find_sysline_between_datetime_filters(
            fileoffset, &self.filter_dt_after_opt, &self.filter_dt_before_opt,
        )
    }

    /// wrapper for a recurring sanity check
    /// good for checking `process_stageX` function calls are in correct order
    #[inline]
    fn assert_stage(&self, stage_expact: ProcessingMode) {
        assert_eq!(
            self.processingmode, stage_expact,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, stage_expact,
        );
    }

    /// stage 0 does some sanity checks on the file
    /// XXX: this is redundant and has already been performed by functions in
    ///      `filepreprocessor` and `BlockReader::new`.
    pub fn process_stage0_valid_file_check(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage0_valid_file_check", sn());
        // sanity check calls are in correct order
        self.assert_stage(ProcessingMode::stage0_valid_file_check);
        self.processingmode = ProcessingMode::stage0_valid_file_check;

        if self.filesz() == 0 {
            debug_eprintln!("{}syslogprocessor.process_stage0_valid_file_check: filesz 0; return {:?}", sx(), FileProcessingResult_BlockZero::FILE_ERR_EMPTY);
            return FileProcessingResult_BlockZero::FILE_ERR_EMPTY;
        }
        debug_eprintln!("{}syslogprocessor.process_stage0_valid_file_check: return {:?}", sx(), FileProcessingResult_BlockZero::FILE_OK);

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// stage 1: Can `Line`s and `Sysline`s be parsed from the first block (block zero)?
    pub fn process_stage1_blockzero_analysis(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage1_blockzero_analysis", sn());
        self.assert_stage(ProcessingMode::stage0_valid_file_check);
        self.processingmode = ProcessingMode::stage1_blockzero_analysis;

        let result = self.blockzero_analysis();
        debug_eprintln!("{}syslogprocessor.process_stage1_blockzero_analysis: return {:?}", sx(), result);

        result
    }

    /// stage 2: Given the two optional datetime filters, can a datetime be
    /// found between those filters?
    pub fn process_stage2_find_dt(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage2_find_dt", snx());
        self.assert_stage(ProcessingMode::stage1_blockzero_analysis);
        self.processingmode = ProcessingMode::stage2_find_dt;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// stage 3: during streaming, processed and printed data stored by underlying
    /// "Readers" is proactively dropped (removed from process memory).
    pub fn process_stage3_stream_syslines(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage3_stream_syslines", snx());
        self.assert_stage(ProcessingMode::stage2_find_dt);
        self.processingmode = ProcessingMode::stage3_stream_syslines;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// stage 4: no more syslines to process, only interested in the `self.summary()`
    pub fn process_stage4_summary(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage4_summary", snx());
        self.processingmode = ProcessingMode::stage4_summary;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// Attempt to find a minimum number of `Sysline` within the first block.
    /// If enough `Sysline` found then return `FILE_OK` else `FILE_ERR_NO_SYSLINES_FOUND`.
    pub(crate) fn blockzero_analysis_syslines(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines", sn());
        self.assert_stage(ProcessingMode::stage1_blockzero_analysis);

        let blockp: BlockP = match self.syslinereader.linereader.blockreader.read_block(0) {
            ResultS3_ReadBlock::Found(blockp_) => blockp_,
            ResultS3_ReadBlock::Done => {
                debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: return FILE_ERR_EMPTY", sx());
                return FileProcessingResult_BlockZero::FILE_ERR_EMPTY;
            },
            ResultS3_ReadBlock::Err(err) => {
                debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: return FILE_ERR_IO({:?})", sx(), err);
                return FileProcessingResult_BlockZero::FILE_ERR_IO(err);
            },
        };
        let blocksz0: BlockSz = (*blockp).len() as BlockSz;
        let mut fo: FileOffset = 0;
        // how many syslines have been found?
        let mut found: u64 = 0;
        // must find at least this many syslines in block zero to be FILE_OK
        let found_min: u64 = *BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP.get(&blocksz0).unwrap();
        debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: block zero blocksz {} found_min {:?}", sx(), blocksz0, found_min);
        // find `at_max` Syslines within block zero
        while found < found_min {
            fo = match self.syslinereader.find_sysline_in_block(fo) {
                ResultS4_SyslineFind::Found((fo_next, _slinep)) => {
                    found += 1;

                    fo_next
                },
                ResultS4_SyslineFind::Found_EOF((_fo_next, _slinep)) => {
                    found += 1;
                    break;
                }, ResultS4_SyslineFind::Done => {
                    found += 1;
                    break;
                }, ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: return FILE_ERR_IO({:?})", sx(), err);
                    return FileProcessingResult_BlockZero::FILE_ERR_IO(err);
                },
            };
            if 0 != self.syslinereader.block_offset_at_file_offset(fo) {
                break;
            }
        }

        let fpr: FileProcessingResult_BlockZero = match found >= found_min {
            true => FileProcessingResult_BlockZero::FILE_OK,
            false => FileProcessingResult_BlockZero::FILE_ERR_NO_SYSLINES_FOUND,
        };

        debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines() found {} syslines, require {} syslines, return {:?}", sx(), found, found_min, fpr);

        fpr
    }

    /// Attempt to find a minimum number of `Line`s within the first block (block zero).
    /// If enough `Line` found then return `FILE_OK` else `FILE_ERR_NO_LINES_FOUND`.
    #[inline]
    pub(crate) fn blockzero_analysis_lines(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis_lines()", sn());
        self.assert_stage(ProcessingMode::stage1_blockzero_analysis);
        
        let blockp: BlockP = match self.syslinereader.linereader.blockreader.read_block(0) {
            ResultS3_ReadBlock::Found(blockp_) => blockp_,
            ResultS3_ReadBlock::Done => {
                debug_eprintln!("{}syslogprocessor.blockzero_analysis_lines: return FILE_ERR_EMPTY", sx());
                return FileProcessingResult_BlockZero::FILE_ERR_EMPTY;
            },
            ResultS3_ReadBlock::Err(err) => {
                debug_eprintln!("{}syslogprocessor.blockzero_analysis_lines: return FILE_ERR_IO({:?})", sx(), err);
                return FileProcessingResult_BlockZero::FILE_ERR_IO(err);
            },
        };
        let blocksz0: BlockSz = (*blockp).len() as BlockSz;
        let mut fo: FileOffset = 0;
        // how many lines have been found?
        let mut found: u64 = 0;
        // must find at least this many lines in block zero to be FILE_OK
        let found_min: u64 = *BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP.get(&blocksz0).unwrap();
        debug_eprintln!("{}syslogprocessor.blockzero_analysis_lines: block zero blocksz {} found_min {}", sx(), blocksz0, found_min);
        // find `found_min` Lines or whatever can be found within block 0
        while found < found_min {
            fo = match self.syslinereader.linereader.find_line_in_block(fo) {
                ResultS4_LineFind::Found((fo_next, _linep)) => {
                    found += 1;

                    fo_next
                },
                ResultS4_LineFind::Found_EOF((_fo_next, _linep)) => {
                    found += 1;
                    break;
                },
                ResultS4_LineFind::Done => {
                    found += 1;
                    break;
                },
                ResultS4_LineFind::Err(err) => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_lines: return FILE_ERR_IO({:?})", sx(), err);
                    return FileProcessingResult_BlockZero::FILE_ERR_IO(err);
                },
            };
            if 0 != self.syslinereader.linereader.block_offset_at_file_offset(fo) {
                break;
            }
        }

        let fpr: FileProcessingResult_BlockZero = match found >= found_min {
            true => FileProcessingResult_BlockZero::FILE_OK,
            false => FileProcessingResult_BlockZero::FILE_ERR_NO_SYSLINES_FOUND,
        };

        debug_eprintln!("{}syslogprocessor.blockzero_analysis_lines: found {} lines, require {} lines, return {:?}", sx(), found, found_min, fpr);

        fpr
    }

    /// Call `self.blockzero_analysis_lines`.
    /// If that passes then call `self.blockzero_analysis_syslines`.
    pub fn blockzero_analysis(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis", sn());
        assert!(!self.blockzero_analysis_done, "blockzero_analysis_lines should only be completed once.");
        self.blockzero_analysis_done = true;
        self.assert_stage(ProcessingMode::stage1_blockzero_analysis);

        let result = self.blockzero_analysis_lines();
        if ! result.is_ok() {
            debug_eprintln!("{}syslogprocessor.blockzero_analysis: syslinereader.blockzero_analysis() was !is_ok(), return {:?}", sx(), result);
            return result;
        };

        let result = self.blockzero_analysis_syslines();
        debug_eprintln!("{}syslogprocessor.blockzero_analysis() return {:?}", sx(), result);

        result
    }

    /// return an up-to-date `Summary` instance for this `SyslogProcessor`
    pub fn summary(&self) -> Summary {
        let filetype = self.filetype();
        let BlockReader_bytes = self.syslinereader.linereader.blockreader.count_bytes();
        let BlockReader_bytes_total = self.filesz() as u64;
        let BlockReader_blocks = self.syslinereader.linereader.blockreader.count_blocks();
        let BlockReader_blocks_total = self.syslinereader.linereader.blockreader.blockn;
        let BlockReader_blocksz = self.blocksz();
        let BlockReader_filesz = self.syslinereader.linereader.blockreader.filesz;
        let BlockReader_filesz_actual = self.syslinereader.linereader.blockreader.filesz_actual;
        let LineReader_lines = self.syslinereader.linereader.count_lines_processed();
        let SyslineReader_syslines = self.syslinereader.count_syslines_processed();
        let SyslineReader_syslines_by_range_hit = self.syslinereader._syslines_by_range_hit;
        let SyslineReader_syslines_by_range_miss = self.syslinereader._syslines_by_range_miss;
        let SyslineReader_syslines_by_range_insert = self.syslinereader._syslines_by_range_insert;
        let SyslineReader_patterns = self.syslinereader.dt_patterns.clone();
        let SyslineReader_find_sysline_lru_cache_hit = self.syslinereader._find_sysline_lru_cache_hit;
        let SyslineReader_find_sysline_lru_cache_miss = self.syslinereader._find_sysline_lru_cache_miss;
        let SyslineReader_find_sysline_lru_cache_put = self.syslinereader._find_sysline_lru_cache_put;
        let SyslineReader_parse_datetime_in_line_lru_cache_hit = self.syslinereader._parse_datetime_in_line_lru_cache_hit;
        let SyslineReader_parse_datetime_in_line_lru_cache_miss = self.syslinereader._parse_datetime_in_line_lru_cache_miss;
        let SyslineReader_parse_datetime_in_line_lru_cache_put = self.syslinereader._parse_datetime_in_line_lru_cache_put;
        let LineReader_find_line_lru_cache_hit = self.syslinereader.linereader._find_line_lru_cache_hit;
        let LineReader_find_line_lru_cache_miss = self.syslinereader.linereader._find_line_lru_cache_miss;
        let LineReader_find_line_lru_cache_put = self.syslinereader.linereader._find_line_lru_cache_put;
        let BlockReader_read_block_lru_cache_hit = self.syslinereader.linereader.blockreader._read_block_cache_lru_hit;
        let BlockReader_read_block_lru_cache_miss = self.syslinereader.linereader.blockreader._read_block_cache_lru_miss;
        let BlockReader_read_block_lru_cache_put = self.syslinereader.linereader.blockreader._read_block_cache_lru_put;
        let BlockReader_read_blocks_hit = self.syslinereader.linereader.blockreader._read_blocks_hit;
        let BlockReader_read_blocks_miss = self.syslinereader.linereader.blockreader._read_blocks_miss;
        let BlockReader_read_blocks_insert = self.syslinereader.linereader.blockreader._read_blocks_insert;

        Summary::new(
            filetype,
            BlockReader_bytes,
            BlockReader_bytes_total,
            BlockReader_blocks,
            BlockReader_blocks_total,
            BlockReader_blocksz,
            BlockReader_filesz,
            BlockReader_filesz_actual,
            LineReader_lines,
            SyslineReader_syslines,
            SyslineReader_syslines_by_range_hit,
            SyslineReader_syslines_by_range_miss,
            SyslineReader_syslines_by_range_insert,
            SyslineReader_patterns,
            SyslineReader_find_sysline_lru_cache_hit,
            SyslineReader_find_sysline_lru_cache_miss,
            SyslineReader_find_sysline_lru_cache_put,
            SyslineReader_parse_datetime_in_line_lru_cache_hit,
            SyslineReader_parse_datetime_in_line_lru_cache_miss,
            SyslineReader_parse_datetime_in_line_lru_cache_put,
            LineReader_find_line_lru_cache_hit,
            LineReader_find_line_lru_cache_miss,
            LineReader_find_line_lru_cache_put,
            BlockReader_read_block_lru_cache_hit,
            BlockReader_read_block_lru_cache_miss,
            BlockReader_read_block_lru_cache_put,
            BlockReader_read_blocks_hit,
            BlockReader_read_blocks_miss,
            BlockReader_read_blocks_insert,
        )
    }
}
