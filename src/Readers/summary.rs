// Readers/summary.rs

use std::fmt;

use crate::Readers::blockreader::{
    BlockSz,
    BLOCKSZ_MAX,
    BLOCKSZ_MIN,
};

use crate::Readers::datetime::{
    DateTime_Parse_Datas_vec,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    //assert_lt,
    assert_ge,
    //assert_gt,
    //debug_assert_le,
    //debug_assert_lt,
    //debug_assert_ge,
    //debug_assert_gt,
};


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// statistics to print about `SyslineReader` activity
#[derive(Clone, Default)]
pub struct Summary {
    /// count of bytes stored by `BlockReader`
    pub BlockReader_bytes: u64,
    /// count of bytes in file
    pub BlockReader_bytes_total: u64,
    /// count of `Block`s read by `BlockReader`
    pub BlockReader_blocks: u64,
    /// count of `Block`s in file
    pub BlockReader_blocks_total: u64,
    /// `BlockSz` of `BlockReader`
    pub BlockReader_blocksz: BlockSz,
    /// count of `Lines` processed by `LineReader`
    pub LineReader_lines: u64,
    /// count of `Syslines` processed by `SyslineReader`
    pub SyslineReader_syslines: u64,
    /// datetime patterns used by `SyslineReader`
    pub SyslineReader_patterns: DateTime_Parse_Datas_vec,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_hit: u64,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_miss: u64,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_put: u64,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_hit: u64,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_miss: u64,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_put: u64,
    /// `LineReader::find_line`
    pub LineReader_find_line_lru_cache_hit: u64,
    /// `LineReader::find_line`
    pub LineReader_find_line_lru_cache_miss: u64,
    /// `LineReader::find_line`
    pub LineReader_find_line_lru_cache_put: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_cache_lru_hit: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_cache_lru_miss: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_cache_lru_put: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_hit: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_miss: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_insert: u32,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        BlockReader_bytes: u64,
        BlockReader_bytes_total: u64,
        BlockReader_blocks: u64,
        BlockReader_blocks_total: u64,
        BlockReader_blocksz: BlockSz,
        LineReader_lines: u64,
        SyslineReader_syslines: u64,
        SyslineReader_patterns: DateTime_Parse_Datas_vec,
        SyslineReader_find_sysline_lru_cache_hit: u64,
        SyslineReader_find_sysline_lru_cache_miss: u64,
        SyslineReader_find_sysline_lru_cache_put: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_hit: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_miss: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_put: u64,
        LineReader_find_line_lru_cache_hit: u64,
        LineReader_find_line_lru_cache_miss: u64,
        LineReader_find_line_lru_cache_put: u64,
        BlockReader_read_block_cache_lru_hit: u32,
        BlockReader_read_block_cache_lru_miss: u32,
        BlockReader_read_block_cache_lru_put: u32,
        BlockReader_read_blocks_hit: u32,
        BlockReader_read_blocks_miss: u32,
        BlockReader_read_blocks_insert: u32,
    ) -> Summary {
        // some sanity checks
        assert_ge!(BlockReader_bytes, BlockReader_blocks, "There is less bytes than Blocks");
        assert_ge!(BlockReader_bytes, LineReader_lines, "There is less bytes than Lines");
        assert_ge!(BlockReader_bytes, SyslineReader_syslines, "There is less bytes than Syslines");
        assert_ge!(BlockReader_blocksz, BLOCKSZ_MIN, "blocksz too small");
        assert_le!(BlockReader_blocksz, BLOCKSZ_MAX, "blocksz too big");
        assert_ge!(LineReader_lines, SyslineReader_syslines, "There is less Lines than Syslines");
        Summary {
            BlockReader_bytes,
            BlockReader_bytes_total,
            BlockReader_blocks,
            BlockReader_blocks_total,
            BlockReader_blocksz,
            LineReader_lines,
            SyslineReader_syslines,
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
            BlockReader_read_block_cache_lru_hit,
            BlockReader_read_block_cache_lru_miss,
            BlockReader_read_block_cache_lru_put,
            BlockReader_read_blocks_hit,
            BlockReader_read_blocks_miss,
            BlockReader_read_blocks_insert,
        }
    }
}

impl fmt::Debug for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("")
            .field("bytes", &self.BlockReader_bytes)
            .field("bytes total", &self.BlockReader_bytes_total)
            .field("lines", &self.LineReader_lines)
            .field("syslines", &self.SyslineReader_syslines)
            .field("blocks", &self.BlockReader_blocks)
            .field("blocks total", &self.BlockReader_blocks_total)
            .field("blocksz", &format_args!("{0} (0x{0:X})", &self.BlockReader_blocksz))
            .finish()
    }
}

pub type Summary_Opt = Option<Summary>;
