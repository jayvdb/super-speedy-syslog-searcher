// summary.rs

#![allow(non_camel_case_types)]

/// CLI option `--summary` printing functions
///
/// Only used by `s4.rs`

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::str;

use ::chrono::{
    DateTime,
    Datelike,
    FixedOffset,
    Local,
    TimeZone,
    Timelike,
};
use ::lazy_static::lazy_static;
use ::mime_guess::MimeGuess;
use ::si_trace_print::defñ;

use crate::debug_panic;
use crate::common::{
    Count,
    FPath,
    PathId,
    FileType,
    LogMessageType,
    SetPathId,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    DateTimeParseDatasCompiledCount,
    DateTimeParseInstr,
    Utc,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
};
use crate::data::common::LogMessage;
use crate::data::evtx::Evtx;
use crate::data::journal::JournalEntry;
use crate::data::fixedstruct::FixedStruct;
use crate::data::sysline::SyslineP;
use crate::debug::printers::e_err;
use crate::readers::blockreader::SummaryBlockReader;
use crate::readers::filepreprocessor::ProcessPathResult;
use crate::readers::fixedstructreader::SummaryFixedStructReader;
use crate::readers::linereader::SummaryLineReader;
use crate::readers::summary::{
    Summary,
    SummaryOpt,
    SummaryReaderData,
};
use crate::readers::syslinereader::SummarySyslineReader;
use crate::readers::syslogprocessor::FileProcessingResultBlockZero;
use crate::printer::printers::{
    print_colored_stderr,
    write_stderr,
    // termcolor imports
    Color,
    ColorChoice,
    PrinterLogMessage,
    //
    COLOR_ERROR,
    COLOR_DIMMED,
};

pub type MapPathIdSummaryPrint = BTreeMap<PathId, SummaryPrinted>;
pub type MapPathIdSummary = HashMap<PathId, Summary>;
pub type MapPathIdToProcessPathResult = HashMap<PathId, ProcessPathResult>;
pub type MapPathIdToProcessPathResultOrdered = BTreeMap<PathId, ProcessPathResult>;
pub type MapPathIdToFPath = BTreeMap<PathId, FPath>;
pub type MapPathIdToColor = HashMap<PathId, Color>;
pub type MapPathIdToPrinterLogMessage = HashMap<PathId, PrinterLogMessage>;
pub type MapPathIdToModifiedTime = HashMap<PathId, DateTimeLOpt>;
pub type MapPathIdToFileProcessingResultBlockZero = HashMap<PathId, FileProcessingResultBlockZero>;
pub type MapPathIdToFileType = HashMap<PathId, FileType>;
pub type MapPathIdToLogMessageType = HashMap<PathId, LogMessageType>;
pub type MapPathIdToMimeGuess = HashMap<PathId, MimeGuess>;

lazy_static! {
    pub static ref FIXEDOFFSET0: FixedOffset = FixedOffset::east_opt(0).unwrap();
}

/// Print the various caching statistics.
const OPT_SUMMARY_PRINT_CACHE_STATS: bool = true;

/// Print the various drop statistics.
const OPT_SUMMARY_PRINT_DROP_STATS: bool = true;

/// For printing various levels of indentation.
const OPT_SUMMARY_PRINT_INDENT1: &str = "  ";
const OPT_SUMMARY_PRINT_INDENT2: &str = "      ";
const OPT_SUMMARY_PRINT_INDENT3: &str = "                   ";

/// print the passed `DateTimeL` as UTC with dimmed color
fn print_datetime_utc_dimmed(
    dt: &DateTimeL,
    color_choice_opt: Option<ColorChoice>,
) {
    let dt_utc = dt.with_timezone(&*FIXEDOFFSET0);
    match print_colored_stderr(
        COLOR_DIMMED,
        color_choice_opt,
        format!("({})", dt_utc).as_bytes()
    ) {
        Err(e) => {
            eprintln!("\nERROR: print_colored_stderr {:?}", e);
            return;
        }
        Ok(_) => {}
    }
}

/// Statistics about the main processing thread's printing activity.
/// Used with CLI option `--summary`.
#[derive(Copy, Clone, Default)]
pub struct SummaryPrinted {
    /// count of bytes printed
    pub bytes: Count,
    /// underlying `LogMessageType`
    pub logmessagetype: LogMessageType,
    /// count of `Lines` printed
    pub lines: Count,
    /// count of `Syslines` printed
    pub syslines: Count,
    /// count of `FixedStruct` printed
    pub fixedstructentries: Count,
    /// count of `Evtx` printed
    pub evtxentries: Count,
    /// count of `JournalEntry` printed
    pub journalentries: Count,
    /// last datetime printed
    pub dt_first: DateTimeLOpt,
    pub dt_last: DateTimeLOpt,
}

pub type SummaryPrintedOpt = Option<SummaryPrinted>;

impl fmt::Debug for SummaryPrinted {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Printed:")
            .field("bytes", &self.bytes)
            .field("lines", &self.lines)
            .field("syslines", &self.syslines)
            .field(
                "dt_first",
                &format_args!(
                    "{}",
                    match self.dt_first {
                        Some(dt) => {
                            dt.to_string()
                        }
                        None => {
                            String::from("None")
                        }
                    }
                ),
            )
            .field(
                "dt_last",
                &format_args!(
                    "{}",
                    match self.dt_last {
                        Some(dt) => {
                            dt.to_string()
                        }
                        None => {
                            String::from("None")
                        }
                    }
                ),
            )
            .finish()
    }
}

// TODO: move `SummaryPrinted` into `printer/summary.rs`
impl SummaryPrinted {
    pub fn new(logmessagetype: LogMessageType) -> SummaryPrinted {
        SummaryPrinted {
            bytes: 0,
            logmessagetype,
            lines: 0,
            syslines: 0,
            fixedstructentries: 0,
            evtxentries: 0,
            journalentries: 0,
            dt_first: None,
            dt_last: None,
        }
    }

    /// Print a `SummaryPrinted` with color on stderr for a file.
    ///
    /// Mimics debug print but with colorized zero values.
    /// Only colorize if associated `SummaryOpt` has corresponding
    /// non-zero values.
    pub fn print_colored_stderr(
        &self,
        color_choice_opt: Option<ColorChoice>,
        summary_opt: &SummaryOpt,
        indent1: &str,
        indent2: &str,
    ) {
        let summary: &Summary = match summary_opt {
            Some(s) => s,
            None => return,
        };
        let (
            summaryblockreader_opt,
            summarylinereader_opt,
            _summarysyslinereader_opt,
            _summarysyslogprocessor_opt,
            summaryfixedstructreader_opt,
            summaryevtxreader_opt,
            summaryjournalreader_opt,
        ) = match &summary.readerdata {
            // `Dummy` may occur for files without adequate read permissions
            SummaryReaderData::Dummy => return,
            SummaryReaderData::Syslog(
                (
                    summaryblockreader,
                    summarylinereader,
                    summarysyslinereader,
                    summarysyslogprocessor,
                )
            ) => {
                (
                    Some(summaryblockreader),
                    Some(summarylinereader),
                    Some(summarysyslinereader),
                    Some(summarysyslogprocessor),
                    None,
                    None,
                    None,
                )
            }
            SummaryReaderData::FixedStruct(
                (
                    summaryblockreader,
                    summaryfixedstructreader,
                )
            ) => {
                (
                    Some(summaryblockreader),
                    None,
                    None,
                    None,
                    Some(summaryfixedstructreader),
                    None,
                    None,
                )
            }
            SummaryReaderData::Etvx(summaryevtxreader) => {
                (
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(summaryevtxreader),
                    None,
                )
            }
            SummaryReaderData::Journal(summaryjournalreader) => {
                (
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(summaryjournalreader),
                )
            }
        };
        eprintln!("{}Printed:", indent1);

        match summaryblockreader_opt {
            Some(summaryblockreader) => {
                eprint!("{}bytes         : ", indent2);
                if self.bytes == 0 && summaryblockreader.blockreader_bytes != 0 {
                    match print_colored_stderr(
                        COLOR_ERROR,
                        color_choice_opt,
                        self.bytes
                            .to_string()
                            .as_bytes(),
                    ) {
                        Err(e) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", e);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    eprintln!("{}", self.bytes);
                }

                if summarylinereader_opt.is_some() {
                    eprint!("{}lines         : ", indent2);
                    if self.lines == 0 && summaryblockreader.blockreader_bytes != 0 {
                        match print_colored_stderr(
                            COLOR_ERROR,
                            color_choice_opt,
                            self.lines
                                .to_string()
                                .as_bytes(),
                        ) {
                            Err(e) => {
                                eprintln!("\nERROR: print_colored_stderr {:?}", e);
                                return;
                            }
                            Ok(_) => eprintln!(),
                        }
                    } else {
                        eprintln!("{}", self.lines);
                    }
                }
            }
            None => {}
        }

        match summaryfixedstructreader_opt {
            Some(summaryfixedstructreader) => {
                eprint!("{}entries       : ", indent2);
                if self.fixedstructentries == 0 && summaryfixedstructreader.fixedstructreader_utmp_entries != 0 {
                    match print_colored_stderr(
                        COLOR_ERROR,
                        color_choice_opt,
                        self.fixedstructentries
                            .to_string()
                            .as_bytes(),
                    ) {
                        Err(e) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", e);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    eprintln!("{}", self.fixedstructentries);
                }
            }
            None => {}
        }

        match summarylinereader_opt {
            // if lines were processed but no syslines were processed
            // then hint at an error with colored text
            Some(summarylinereader) => {
                eprint!("{}syslines      : ", indent2);
                if self.syslines == 0 && summarylinereader.linereader_lines != 0 {
                    match print_colored_stderr(
                        COLOR_ERROR,
                        color_choice_opt,
                        self.syslines
                            .to_string()
                            .as_bytes(),
                    ) {
                        Err(e) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", e);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    eprintln!("{}", self.syslines);
                }
            }
            None => {}
        }

        match summarylinereader_opt {
            Some(summarylinereader) => {
                if self.dt_first.is_none() && summarylinereader.linereader_lines != 0 {
                    // if no datetime_first was processed but lines were processed
                    // then hint at an error with colored text
                    eprint!("{}datetime first: ", indent2);
                    match print_colored_stderr(COLOR_ERROR, color_choice_opt, "None Found".as_bytes()) {
                        Err(e) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", e);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    match self.dt_first {
                        Some(dt) => {
                            eprint!("{}datetime first: {:?} ", indent2, dt);
                            print_datetime_utc_dimmed(&dt, color_choice_opt);
                            eprintln!();
                        },
                        None => {}
                    }
                }
                if self.dt_last.is_none() && summarylinereader.linereader_lines != 0 {
                    // if no datetime_last was processed but lines were processed
                    // then hint at an error with colored text
                    eprint!("{}datetime last : ", indent2);
                    match print_colored_stderr(COLOR_ERROR, color_choice_opt, "None Found".as_bytes()) {
                        Err(e) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", e);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    match self.dt_last {
                        Some(dt) => {
                            eprint!("{}datetime last : {:?} ", indent2, dt);
                            print_datetime_utc_dimmed(&dt, color_choice_opt);
                            eprintln!();
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }

        match summaryevtxreader_opt {
            Some(summaryevtxreader) => {
                eprintln!("{}bytes         : {}", indent2, self.bytes);
                eprintln!("{}Events        : {}", indent2, self.evtxentries);
                match summaryevtxreader.evtxreader_datetime_first_accepted {
                    Some(dt) => {
                        eprint!("{}datetime first: {:?} ", indent2, dt);
                        print_datetime_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
                match summaryevtxreader.evtxreader_datetime_last_accepted {
                    Some(dt) => {
                        eprint!("{}datetime last : {:?} ", indent2, dt);
                        print_datetime_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
            }
            None => {}
        }

        match summaryjournalreader_opt {
            Some(summaryjournalreader) => {
                eprintln!("{}bytes         : {}", indent2, self.bytes);
                eprintln!("{}journal events: {}", indent2, self.journalentries);
                match summaryjournalreader.journalreader_datetime_first_accepted {
                    Some(dt) => {
                        eprint!("{}datetime first: {:?} ", indent2, dt);
                        print_datetime_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
                match summaryjournalreader.journalreader_datetime_last_accepted {
                    Some(dt) => {
                        eprint!("{}datetime last : {:?} ", indent2, dt);
                        print_datetime_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
            }
            None => {}
        }
    }

    fn summaryprint_update_dt(
        &mut self,
        dt: &DateTimeL,
    ) {
        defñ!();
        match self.dt_first {
            Some(dt_first) => {
                if dt < &dt_first {
                    self.dt_first = Some(*dt);
                };
            }
            None => {
                self.dt_first = Some(*dt);
            }
        };
        match self.dt_last {
            Some(dt_last) => {
                if dt > &dt_last {
                    self.dt_last = Some(*dt);
                };
            }
            None => {
                self.dt_last = Some(*dt);
            }
        };
    }

    /// Update a `SummaryPrinted` with information from a printed `Sysline`.
    pub fn summaryprint_update_sysline(
        &mut self,
        syslinep: &SyslineP,
        printed: Count,
    ) {
        defñ!();
        debug_assert!(matches!(self.logmessagetype,
            LogMessageType::Sysline | LogMessageType::All), "Unexpected LogMessageType {:?}", self.logmessagetype);
        self.syslines += 1;
        self.lines += (*syslinep).count_lines();
        self.bytes += printed;
        self.summaryprint_update_dt((*syslinep).dt());
    }

    /// Update a `SummaryPrinted` with information from a printed `FixedStruct`.
    pub fn summaryprint_update_fixedstruct(
        &mut self,
        entry: &FixedStruct,
        printed: Count,
    )
    {
        defñ!();
        debug_assert!(matches!(self.logmessagetype,
            LogMessageType::FixedStruct | LogMessageType::All),
            "Unexpected LogMessageType {:?}", self.logmessagetype);
        self.fixedstructentries += 1;
        self.bytes += printed;
        self.summaryprint_update_dt(entry.dt());
    }

    /// Update a `SummaryPrinted` with information from a printed `Evtx`.
    pub fn summaryprint_update_evtx(
        &mut self,
        evtx: &Evtx,
        printed: Count,
    ) {
        defñ!();
        debug_assert!(matches!(self.logmessagetype,
            LogMessageType::Evtx | LogMessageType::All), "Unexpected LogMessageType {:?}", self.logmessagetype);
        self.evtxentries += 1;
        self.bytes += printed;
        self.summaryprint_update_dt(evtx.dt());
    }

    pub fn summaryprint_update_journalentry(
        &mut self,
        journalentry: &JournalEntry,
        printed: Count,
    ) {
        defñ!();
        debug_assert!(matches!(self.logmessagetype,
            LogMessageType::Journal | LogMessageType::All), "Unexpected LogMessageType {:?}", self.logmessagetype);
        self.journalentries += 1;
        self.bytes += printed;
        self.summaryprint_update_dt(journalentry.dt());
    }

    /// Update a `SummaryPrinted` with information from a printed `LogMessage`.
    pub fn _summaryprint_update(
        &mut self,
        logmessage: &LogMessage,
        printed: Count,
    ) {
        defñ!();
        match logmessage {
            LogMessage::Sysline(syslinep) => {
                self.summaryprint_update_sysline(syslinep, printed);
            }
            LogMessage::FixedStruct(entry) => {
                self.summaryprint_update_fixedstruct(entry, printed);
            }
            LogMessage::Evtx(evtx) => {
                self.summaryprint_update_evtx(evtx, printed);
            }
            LogMessage::Journal(journalentry) => {
                self.summaryprint_update_journalentry(journalentry, printed);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `Sysline`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_sysline(
        syslinep: &SyslineP,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_sysline(syslinep, printed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Sysline);
                sp.summaryprint_update_sysline(syslinep, printed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `FixedStruct`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_fixedstruct(
        fixedstruct: &FixedStruct,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
    )
    {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_fixedstruct(fixedstruct, printed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::FixedStruct);
                sp.summaryprint_update_fixedstruct(fixedstruct, printed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `FixedStruct`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_evtx(
        evtx: &Evtx,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_evtx(evtx, printed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Evtx);
                sp.summaryprint_update_evtx(evtx, printed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `JournalEntry`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_journalentry(
        journalentry: &JournalEntry,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_journalentry(journalentry, printed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Journal);
                sp.summaryprint_update_journalentry(journalentry, printed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn _summaryprint_map_update(
        logmessage: &LogMessage,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
    ) {
        defñ!();
        match logmessage {
            LogMessage::Evtx(evtx) => {
                Self::summaryprint_map_update_evtx(evtx, pathid, map_, printed)
            }
            LogMessage::FixedStruct(entry) => {
                Self::summaryprint_map_update_fixedstruct(entry, pathid, map_, printed)
            }
            LogMessage::Journal(journalentry) => {
                Self::summaryprint_map_update_journalentry(journalentry, pathid, map_, printed)
            }
            LogMessage::Sysline(syslinep) => {
                Self::summaryprint_map_update_sysline(syslinep, pathid, map_, printed)
            }
        }
    }
}

/// Helper function to function `processing_loop`.
#[inline(always)]
pub fn summary_update(
    pathid: &PathId,
    summary: Summary,
    map_: &mut MapPathIdSummary,
) {
    if let Some(val) = map_.insert(*pathid, summary) {
        eprintln!(
            "Error: processing_loop: map_pathid_summary already contains key {:?} with {:?}, overwritten",
            pathid, val
        );
    };
}

/// Print the entire `--summary`.
/// This is the "entry point" for print the summary of all files.
pub fn print_summary(
    map_pathid_results: MapPathIdToProcessPathResult,
    map_pathid_results_invalid: MapPathIdToProcessPathResultOrdered,
    map_pathid_path: MapPathIdToFPath,
    map_pathid_modified_time: MapPathIdToModifiedTime,
    map_pathid_file_processing_result: MapPathIdToFileProcessingResultBlockZero,
    map_pathid_filetype: MapPathIdToFileType,
    map_pathid_logmessagetype: MapPathIdToLogMessageType,
    map_pathid_mimeguess: MapPathIdToMimeGuess,
    map_pathid_color: MapPathIdToColor,
    mut map_pathid_summary: MapPathIdSummary,
    mut map_pathid_sumpr: MapPathIdSummaryPrint,
    color_choice: ColorChoice,
    color_default: Color,
    paths_total: usize,
    paths_printed_logmessages: SetPathId,
    summaryprinted: SummaryPrinted,
    filter_dt_after_opt: &DateTimeLOpt,
    filter_dt_before_opt: &DateTimeLOpt,
    local_now: &DateTime<Local>,
    utc_now: &DateTime<Utc>,
    chan_recv_ok: Count,
    chan_recv_err: Count,
) {
    eprintln!();
    eprintln!("Files:");
    // print details about all the valid files
    print_all_files_summaries(
        &map_pathid_path,
        &map_pathid_modified_time,
        &map_pathid_file_processing_result,
        &map_pathid_filetype,
        &map_pathid_logmessagetype,
        &map_pathid_mimeguess,
        &map_pathid_color,
        &mut map_pathid_summary,
        &mut map_pathid_sumpr,
        &color_choice,
        &color_default,
    );
    if !map_pathid_path.is_empty(){
        eprintln!();
    }
    // print a short note about the invalid files
    print_files_processpathresult(
        &map_pathid_results_invalid,
        &color_choice,
        &color_default,
        &COLOR_ERROR,
    );
    eprintln!();

    // here is the final printed summary of the all files
    eprintln!("Program Summary:\n");
    eprintln!("Paths considered      : {}", paths_total);
    eprintln!("Paths not processed   : {}", map_pathid_results_invalid.len());
    eprintln!("Files processed       : {}", map_pathid_results.len());
    eprintln!("Files printed         : {}", paths_printed_logmessages.len());
    eprintln!("Printed bytes         : {}", summaryprinted.bytes);
    eprintln!("Printed lines         : {}", summaryprinted.lines);
    eprintln!("Printed syslines      : {}", summaryprinted.syslines);
    eprintln!("Printed evtx events   : {}", summaryprinted.evtxentries);
    // TODO: [2023/03/26] eprint count of EVTX files "out of order".
    eprintln!("Printed fixedstruct   : {}", summaryprinted.fixedstructentries);
    // TODO: [2024/02/25] eprint count of FixedStruct files "out of order".
    eprintln!("Printed journal events: {}", summaryprinted.journalentries);
    let count: isize = match DateTimeParseDatasCompiledCount.read() {
        Ok(count) => *count as isize,
        // XXX: hacky hint that the count is not available
        Err(_) => -1,
    };
    eprintln!("Regex patterns        : {}", DATETIME_PARSE_DATAS_LEN);
    eprintln!("Regex compiled        : {}", count);

    eprint!("Datetime filter -a    :");
    match filter_dt_after_opt {
        Some(dt) => {
            eprint!(" {:?} ", dt);
            print_datetime_utc_dimmed(dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    eprint!("Datetime printed first:");
    match summaryprinted.dt_first {
        Some(dt) => {
            eprint!(" {:?} ", dt);
            print_datetime_utc_dimmed(&dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    eprint!("Datetime printed last :");
    match summaryprinted.dt_last {
        Some(dt) => {
            eprint!(" {:?} ", dt);
            print_datetime_utc_dimmed(&dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    eprint!("Datetime filter -b    :");
    match filter_dt_before_opt {
        Some(dt) => {
            eprint!(" {:?} ", dt);
            print_datetime_utc_dimmed(&dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    // print the time now as this program sees it, drop sub-second values
    let local_now = Local
        .with_ymd_and_hms(
            local_now.year(),
            local_now.month(),
            local_now.day(),
            local_now.hour(),
            local_now.minute(),
            local_now.second(),
        )
        .unwrap();
    // BUG: local time prints with middle "T", UTC prints with midle space " "
    //      e.g. 
    //           Datetime Now          : 2024-02-07T22:33:50-08:00 (2024-02-08 06:33:50 +00:00)
    eprint!("Datetime Now          : {:?} ", local_now);
    // print UTC now without fractional, and with numeric offset `-00:00`
    // instead of `Z`
    let utc_now = Utc
        .with_ymd_and_hms(
            utc_now.year(),
            utc_now.month(),
            utc_now.day(),
            utc_now.hour(),
            utc_now.minute(),
            utc_now.second(),
        )
        .unwrap()
        .with_timezone(&*FIXEDOFFSET0);
    print_datetime_utc_dimmed(&utc_now, Some(color_choice));
    eprintln!();
    // print basic stats about the channel
    eprintln!("Channel Receive ok    : {}", chan_recv_ok);
    eprintln!("Channel Receive err   : {}", chan_recv_err);
}

// TODO: [2023/04/05] move printing of `file size` from per-file "Processed:"
//       section to "About:" section. Having in the "Processed:" section is
//       confusing about what was actually read from storage (implies the
//       entire file was read, which is not true in most cases).

/// Print the file About section (multiple lines).
fn print_file_about(
    path: &FPath,
    modified_time: &DateTimeLOpt,
    file_processing_result: Option<&FileProcessingResultBlockZero>,
    filetype: &FileType,
    logmessagetype: &LogMessageType,
    mimeguess: &MimeGuess,
    summary_opt: &SummaryOpt,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprint!("File: ");
    // print path
    match print_colored_stderr(
        *color,
        Some(*color_choice),
        path.as_bytes()
    ) {
        Ok(_) => {}
        Err(e) => e_err!("print_colored_stderr: {:?}", e)
    }
    eprintln!("\n{}About:", OPT_SUMMARY_PRINT_INDENT1);
    // if symlink or relative path then print target
    // XXX: experimentation revealed std::fs::Metadata::is_symlink to be unreliable on WSL Ubuntu
    match std::fs::canonicalize(path) {
        Ok(pathb) => match pathb.to_str() {
            Some(s) => {
                if s != path.as_str() {
                    eprint!("{}realpath       : ", OPT_SUMMARY_PRINT_INDENT2);
                    write_stderr(s.as_bytes());
                    eprintln!();
                }
            }
            None => {}
        },
        Err(_) => {}
    }
    // print other facts
    match modified_time {
        Some(dt) => {
            eprint!("{}Modified Time  : {:?} ", OPT_SUMMARY_PRINT_INDENT2, dt);
            print_datetime_utc_dimmed(&dt, Some(*color_choice));
            eprintln!();
        }
        None => {}
    }
    eprintln!("{}filetype       : {}", OPT_SUMMARY_PRINT_INDENT2, filetype);
    match filetype {
        FileType::FixedStruct{type_: fixedstructfiletype } => {
            eprintln!("{}fixedstructtype: {:?}", OPT_SUMMARY_PRINT_INDENT2, fixedstructfiletype);
        }
        _ => {}
    }
    eprintln!("{}logmessagetype : {}", OPT_SUMMARY_PRINT_INDENT2, logmessagetype);
    match summary_opt {
        Some(summary) => {
            match &summary.readerdata {
                SummaryReaderData::FixedStruct((_, summaryfixedstructreader)) => {
                    match summaryfixedstructreader.fixedstructreader_fixedstructtype_opt {
                        Some(fst) => {
                            eprintln!(
                                "{}fixedstructtype: {:?}",
                                OPT_SUMMARY_PRINT_INDENT2,
                                fst,
                            );
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }
        None => {}
    }
    eprintln!("{}MIME guess     : {:?}", OPT_SUMMARY_PRINT_INDENT2, mimeguess);
    // print `FileProcessingResult` if it was not okay
    match file_processing_result {
        Some(result) => {
            if !result.is_ok() {
                eprint!("{}Processing Err : ", OPT_SUMMARY_PRINT_INDENT2);
                match print_colored_stderr(
                    COLOR_ERROR,
                    Some(*color_choice),
                    match result {
                        // only print ErrorKind here
                        // later the Error message will be printed
                        FileProcessingResultBlockZero::FileErrIoPath(err)
                        | FileProcessingResultBlockZero::FileErrIo(err) =>
                            format!("{}", err.kind()),
                        FileProcessingResultBlockZero::FileErrTooSmallS(_) =>
                            format!("FileErrTooSmall"),
                        FileProcessingResultBlockZero::FileErrNoSyslinesInDtRange =>
                            format!("No Syslines in DateTime Range"),
                        FileProcessingResultBlockZero::FileErrNoFixedStructInDtRange =>
                            format!("No FixedStruct in DateTime Range"),
                        _ => format!("{:?}", result),
                    }.as_bytes()
                ) {
                    Ok(_) => {}
                    Err(e) => e_err!("print_colored_stderr: {:?}", e)
                }
                eprintln!();
            }
        }
        None => {}
    }
}

/// Print the (optional) [`Summary`] (multiple lines) processed sections.
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_summary_opt_processed(
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    let summary = match summary_opt {
        Some(summary) => {
            summary
        }
        None => {
            eprintln!("{}Processed: None", OPT_SUMMARY_PRINT_INDENT1);
            return;
        }
    };
    if summary.readerdata.is_dummy() {
        // `Dummy` may occur for files without adequate read permissions
        // there will be no interested information in gathered statistics
        return;
    }
    let indent1 = OPT_SUMMARY_PRINT_INDENT1;
    let indent2 = OPT_SUMMARY_PRINT_INDENT2;
    eprintln!("{}Processed:", indent1);
    print_summary_opt_processed_summaryblockreader(
        summary,
        indent2,
    );
    match &summary.readerdata {
        // `Dummy` may occur for files without adequate read permissions
        SummaryReaderData::Dummy => return,
        SummaryReaderData::Syslog(
            (
                _summaryblockreader,
                summarylinereader,
                summarysyslinereader,
                _summarysyslogprocessor,
            ),
        ) => {
            eprintln!("{}lines         : {}", indent2, summarylinereader.linereader_lines);
            eprintln!(
                "{}lines high    : {}",
                indent2, summarylinereader.linereader_lines_stored_highest
            );
            eprintln!("{}syslines      : {}", indent2, summarysyslinereader.syslinereader_syslines);
            eprintln!(
                "{}syslines high : {}",
                indent2, summarysyslinereader.syslinereader_syslines_stored_highest
            );
        }
        SummaryReaderData::FixedStruct((
            _summaryblockreader,
            summaryfixedstructreader,
        )) => {
            eprintln!("{}entries       : {}", indent2, summaryfixedstructreader.fixedstructreader_utmp_entries);
            eprintln!("{}entry size    : {} (bytes)",
                indent2, summaryfixedstructreader.fixedstructreader_fixedstruct_size
            );
            eprintln!("{}entry hi-score: {}",
                indent2, summaryfixedstructreader.fixedstructreader_high_score
            );
            eprint!("{}first entry   : ",
                indent2,
            );
            eprintln!("@{:?}", summaryfixedstructreader.fixedstructreader_first_entry_fileoffset);
            eprintln!(
                "{}entry high    : {}",
                indent2, summaryfixedstructreader.fixedstructreader_utmp_entries_max,
            );
            eprintln!(
                "{}peak map size : {}",
                indent2, summaryfixedstructreader.fixedstructreader_map_tvpair_fo_max_len
            );
            eprintln!(
                "{}out of order? : {}",
                indent2, summaryfixedstructreader.fixedstructreader_entries_out_of_order
            );
        }
        SummaryReaderData::Etvx(summaryevtxreader) => {
            eprintln!(
                "{}file size          : {1} (0x{1:X}) (bytes)",
                indent2, summaryevtxreader.evtxreader_filesz,
            );
            // TODO: [2023/04/05] add `sourced` size. Requires additional
            //       tracking in `EvtxReader` (small `EvtxReader` refactor)
            //       and `SummaryEvtxReader`.
            eprintln!("{}Events processed   : {}", indent2, summaryevtxreader.evtxreader_events_processed);
            eprintln!("{}Events accepted    : {}", indent2, summaryevtxreader.evtxreader_events_accepted);
            // print out of order. If there are any, print in red.
            eprint!("{}Events out of order: ", indent2);
            if summaryevtxreader.evtxreader_out_of_order == 0 {
                eprintln!("{}", summaryevtxreader.evtxreader_out_of_order);
            } else {
                let data = format!("{}", summaryevtxreader.evtxreader_out_of_order);
                match print_colored_stderr(
                    COLOR_ERROR,
                    Some(*color_choice),
                    data.as_bytes(),
                ) {
                    Ok(_) => eprintln!(),
                    Err(e) => e_err!("print_colored_stderr: {:?}", e)
                }
            }
            match summaryevtxreader.evtxreader_datetime_first_processed {
                Some(dt) => {
                    eprint!("{}datetime first     : {:?} ", indent2, dt);
                    print_datetime_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            match summaryevtxreader.evtxreader_datetime_last_processed {
                Some(dt) => {
                    eprint!("{}datetime last      : {:?} ", indent2, dt);
                    print_datetime_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            // for evtx files, nothing left to print about it so return
            return;
        }
        SummaryReaderData::Journal(summaryjournalreader) => {
            eprintln!(
                "{}file size     : {1} (0x{1:X}) (bytes)",
                indent2, summaryjournalreader.journalreader_filesz,
            );
            eprintln!(
                "{}journal events: {}",
                indent2, summaryjournalreader.journalreader_events_processed,
            );
            // print out of order. If there are any, print in red.
            eprint!("{}out of order  : ", indent2);
            if summaryjournalreader.journalreader_out_of_order == 0 {
                eprintln!("{}", summaryjournalreader.journalreader_out_of_order);
            } else {
                let data = format!("{}", summaryjournalreader.journalreader_out_of_order);
                match print_colored_stderr(
                    COLOR_ERROR,
                    Some(*color_choice),
                    data.as_bytes(),
                ) {
                    Ok(_) => eprintln!(),
                    Err(e) => e_err!("print_colored_stderr: {:?}", e)
                }
            }
            eprintln!(
                "{}lib. API calls: {}",
                indent2, summaryjournalreader.journalreader_api_calls,
            );
            // print API call errors. If there are any, print in red.
            eprint!("{}API errors    : ", indent2);
            if summaryjournalreader.journalreader_api_call_errors == 0 {
                eprintln!("{}", summaryjournalreader.journalreader_api_call_errors);
            } else {
                let data = format!("{}", summaryjournalreader.journalreader_api_call_errors);
                match print_colored_stderr(
                    COLOR_ERROR,
                    Some(*color_choice),
                    data.as_bytes(),
                ) {
                    Ok(_) => eprintln!(),
                    Err(e) => e_err!("print_colored_stderr: {:?}", e)
                }
            }
            match summaryjournalreader.journalreader_datetime_first_processed {
                Some(dt) => {
                    eprint!("{}datetime first: {:?} ",indent2, dt);
                    print_datetime_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            match summaryjournalreader.journalreader_datetime_last_processed {
                Some(dt) => {
                    eprint!("{}datetime last : {:?} ", indent2, dt);
                    print_datetime_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            return;
        }
    }
    // print datetime first and last
    match (summary.datetime_first(), summary.datetime_last()) {
        (Some(dt_first), Some(dt_last)) => {
            eprint!("{}datetime first: {:?} ", indent2, dt_first);
            print_datetime_utc_dimmed(&dt_first, Some(*color_choice));
            eprintln!();
            eprint!("{}datetime last : {:?} ", indent2, dt_last);
            print_datetime_utc_dimmed(&dt_last, Some(*color_choice));
            eprintln!();
        }
        (None, Some(_)) | (Some(_), None) =>
            e_err!("only one of dt_first or dt_last fulfilled; this is unexpected."),
        _ => {}
    }
    // print datetime patterns
    match &summary.readerdata {
        SummaryReaderData::Syslog((
            _summaryblockreader,
            _summarylinereader,
            summarysyslinereader,
            summarysyslogprocessor,
        )) => {
            if !summarysyslinereader.syslinereader_patterns.is_empty()
            {
                eprintln!("{}Parsers:", OPT_SUMMARY_PRINT_INDENT1);
            }
            for patt in summarysyslinereader
                .syslinereader_patterns
                .iter()
            {
                let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*patt.0];
                eprintln!("{}@[{}] uses {} {:?}", indent2, patt.0, patt.1, dtpd);
            }
            match summarysyslogprocessor.syslogprocessor_missing_year {
                Some(year) => {
                    eprintln!(
                        "{}datetime format missing year; estimated year of last sysline {:?}",
                        OPT_SUMMARY_PRINT_INDENT3, year
                    );
                }
                None => {}
            }
        }
        _ => {}
    }
}

/// Helper to `print_summary_opt_processed`
fn print_summary_opt_processed_summaryblockreader(
    summary: &Summary,
    indent: &str,
) {
    if summary.readerdata.is_dummy() {
        return;
    }
    let summaryblockreader = match summary.blockreader() {
        Some(summaryblockreader) => {
            summaryblockreader
        }
        None => {
            return;
        }
    };
    debug_assert_ne!(summary.filetype, FileType::Evtx);
    debug_assert_ne!(summary.filetype, FileType::Journal);
    match summary.filetype {
        FileType::File
        | FileType::FixedStruct{..}
        | FileType::Unknown
        => {
            eprintln!(
                "{}file size     : {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
        }
        FileType::Tar => {
            eprintln!(
                "{}file size archive   : {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
            eprintln!(
                "{}file size unarchived: {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz_actual
            );
        }
        FileType::Gz | FileType::Xz => {
            eprintln!(
                "{}file size compressed  : {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
            eprintln!(
                "{}file size uncompressed: {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz_actual
            );
        }
        FileType::TarGz
        | FileType::Evtx
        | FileType::Journal
        | FileType::Unset
        | FileType::Unparsable
        => {
            eprintln!("{}unsupported filetype: {:?}", indent, summary.filetype);
            return;
        }
    }
    // TODO: [2023/04/05] add `sourced` count. Requires additional
    //       tracking in `BlockReader`.
    //       i.e. bytes read from storage.
    eprintln!("{}bytes         : {1} (0x{1:X})", indent, summaryblockreader.blockreader_bytes);
    eprintln!("{}bytes total   : {1} (0x{1:X})", indent, summaryblockreader.blockreader_bytes_total);
    eprintln!(
        "{}block size    : {1} (0x{1:X})",
        indent, summaryblockreader.blockreader_blocksz
    );
    eprintln!("{}blocks        : {}", indent, summaryblockreader.blockreader_blocks);
    eprintln!("{}blocks total  : {}", indent, summaryblockreader.blockreader_blocks_total);
    eprintln!("{}blocks high   : {}", indent, summaryblockreader.blockreader_blocks_highest);
}

/// Print the (optional) [`&SummaryPrinted`] (one line) printed section for
/// one file.
///
/// [`&SummaryPrinted`]: self::SummaryPrinted
pub fn print_summary_opt_printed(
    summary_print_opt: &SummaryPrintedOpt,
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_print_opt {
        Some(summary_print) => {
            defñ!("Some(summary_print)");
            
            summary_print.print_colored_stderr(
                Some(*color_choice),
                summary_opt,
                OPT_SUMMARY_PRINT_INDENT1,
                OPT_SUMMARY_PRINT_INDENT2,
            );
        }
        None => {
            defñ!("None");
            SummaryPrinted::default().print_colored_stderr(
                Some(*color_choice),
                summary_opt,
                OPT_SUMMARY_PRINT_INDENT1,
                OPT_SUMMARY_PRINT_INDENT2,
            );
        }
    }
}

/// create percentage of `a` to `a + b`
fn percent64(
    a: &u64,
    b: &u64,
) -> f64 {
    let den = (*a as f64) + (*b as f64);
    if den == 0.0 {
        return 0.0;
    }
    ((*a as f64) / den) * 100.0
}

const WIDEP: usize = 4;

fn print_cache_stats_summaryblockreader(
    summaryblockreader: &SummaryBlockReader,
    color_choice: &ColorChoice,
    indent: &str,
    wide: usize,
) {
    // BlockReader::_read_blocks
    let mut percent = percent64(&summaryblockreader.blockreader_read_blocks_hit, &summaryblockreader.blockreader_read_blocks_miss);
    eprint!(
        "{}storage: BlockReader::read_block() blocks                    : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summaryblockreader.blockreader_read_blocks_hit,
        summaryblockreader.blockreader_read_blocks_miss,
        percent,
        summaryblockreader.blockreader_read_blocks_put,
        wide = wide,
        widep = WIDEP,
    );
    // append the rereads count, colorize if greater than 0
    let rereads_err_str = format!(
        " (rereads {})\n",
        summaryblockreader.blockreader_read_blocks_reread_error,
    );
    if summaryblockreader.blockreader_read_blocks_reread_error > 0 {
        match print_colored_stderr(
            COLOR_ERROR, 
            Some(*color_choice),
            rereads_err_str.as_bytes()
        ) {
            Ok(_) => {}
            Err(e) => e_err!("print_colored_stderr: {:?}", e)
        }
    } else {
        write_stderr(rereads_err_str.as_bytes());
    }
    // BlockReader::_read_blocks_cache
    percent = percent64(
        &summaryblockreader.blockreader_read_block_lru_cache_hit,
        &summaryblockreader.blockreader_read_block_lru_cache_miss,
    );
    eprintln!(
        "{}caching: BlockReader::read_block() LRU cache                 : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summaryblockreader.blockreader_read_block_lru_cache_hit,
        summaryblockreader.blockreader_read_block_lru_cache_miss,
        percent,
        summaryblockreader.blockreader_read_block_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
}

fn print_cache_stats_summarylinereader(
    summarylinereader: &SummaryLineReader,
    indent: &str,
    wide: usize,
) {
    // LineReader::_lines
    let mut percent = percent64(&summarylinereader.linereader_lines_hits, &summarylinereader.linereader_lines_miss);
    eprintln!(
        "{}storage: LineReader::find_line() lines                       : hit {:wide$}, miss {:wide$}, {:widep$.1}%",
        indent,
        summarylinereader.linereader_lines_hits,
        summarylinereader.linereader_lines_miss,
        percent,
        wide = wide,
        widep = WIDEP,
    );
    // LineReader::_find_line_lru_cache
    percent =
        percent64(&summarylinereader.linereader_find_line_lru_cache_hit, &summarylinereader.linereader_find_line_lru_cache_miss);
    eprintln!(
        "{}caching: LineReader::find_line() LRU cache                   : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarylinereader.linereader_find_line_lru_cache_hit,
        summarylinereader.linereader_find_line_lru_cache_miss,
        percent,
        summarylinereader.linereader_find_line_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
}

fn print_cache_stats_summarysyslinereader(
    summarysyslinereader: &SummarySyslineReader,
    indent: &str,
    wide: usize,
) {
    // SyslineReader
    // SyslineReader::get_boxptrs
    eprintln!(
        "{}copying: SyslineReader::get_boxptrs()                        : sgl {:wide$}, dbl  {:wide$}, mult {:wide$}",
        indent,
        summarysyslinereader.syslinereader_get_boxptrs_singleptr,
        summarysyslinereader.syslinereader_get_boxptrs_doubleptr,
        summarysyslinereader.syslinereader_get_boxptrs_multiptr,
        wide = wide,
    );
    // SyslineReader::syslines
    let mut percent = percent64(&summarysyslinereader.syslinereader_syslines_hit, &summarysyslinereader.syslinereader_syslines_miss);
    eprintln!(
        "{}storage: SyslineReader::find_sysline() syslines              : hit {:wide$}, miss {:wide$}, {:widep$.1}%",
        indent,
        summarysyslinereader.syslinereader_syslines_hit,
        summarysyslinereader.syslinereader_syslines_miss,
        percent,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::_syslines_by_range
    percent =
        percent64(&summarysyslinereader.syslinereader_syslines_by_range_hit, &summarysyslinereader.syslinereader_syslines_by_range_miss);
    eprintln!(
        "{}caching: SyslineReader::find_sysline() syslines_by_range_map : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_syslines_by_range_hit,
        summarysyslinereader.syslinereader_syslines_by_range_miss,
        percent,
        summarysyslinereader.syslinereader_syslines_by_range_put,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::_find_sysline_lru_cache
    percent = percent64(
        &summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
        &summarysyslinereader.syslinereader_find_sysline_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::find_sysline() LRU cache             : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_miss,
        percent,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::_parse_datetime_in_line_lru_cache
    percent = percent64(
        &summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_hit,
        &summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::parse_datetime_in_line() LRU cache   : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_hit,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_miss,
        percent,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::ezcheck12
    percent = percent64(
        &summarysyslinereader.syslinereader_ezcheck12_hit,
        &summarysyslinereader.syslinereader_ezcheck12_miss,
    );
    eprintln!(
        "{}optimize:SyslineReader::ezcheck12                            : hit {:wide$}, miss {:wide$}, {:widep$.1}%, largest skipped {}",
        indent,
        summarysyslinereader.syslinereader_ezcheck12_hit,
        summarysyslinereader.syslinereader_ezcheck12_miss,
        percent,
        summarysyslinereader.syslinereader_ezcheck12_hit_max,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::ezcheckd2
    percent = percent64(
        &summarysyslinereader.syslinereader_ezcheckd2_hit,
        &summarysyslinereader.syslinereader_ezcheckd2_miss,
    );
    eprintln!(
        "{}optimize:SyslineReader::ezcheckd2                            : hit {:wide$}, miss {:wide$}, {:widep$.1}%, largest skipped {}",
        indent,
        summarysyslinereader.syslinereader_ezcheckd2_hit,
        summarysyslinereader.syslinereader_ezcheckd2_miss,
        percent,
        summarysyslinereader.syslinereader_ezcheckd2_hit_max,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::ezcheck12d2
    percent = percent64(
        &summarysyslinereader.syslinereader_ezcheck12d2_hit,
        &summarysyslinereader.syslinereader_ezcheck12d2_miss,
    );
    eprintln!(
        "{}optimize:SyslineReader::ezcheck12d2                          : hit {:wide$}, miss {:wide$}, {:widep$.1}%, largest skipped {}",
        indent,
        summarysyslinereader.syslinereader_ezcheck12d2_hit,
        summarysyslinereader.syslinereader_ezcheck12d2_miss,
        percent,
        summarysyslinereader.syslinereader_ezcheck12d2_hit_max,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::regex_captures_attempted
    eprintln!(
        "{}process: regex captures attempted                            : {:?}",
        indent,
        summarysyslinereader.syslinereader_regex_captures_attempted,
    );
}

fn print_cache_stats_summaryfixedstructreader(
    summaryfixedstructreader: &SummaryFixedStructReader,
    indent: &str,
    wide: usize,
) {
    let percent = percent64(
        &summaryfixedstructreader.fixedstructreader_utmp_entries_hit,
        &summaryfixedstructreader.fixedstructreader_utmp_entries_miss,
    );
    eprintln!(
        "{}storage: FixedStructReader::find_entry()                     : hit {:wide$}, miss {:wide$}, {:widep$.1}%",
        indent,
        summaryfixedstructreader.fixedstructreader_utmp_entries_hit,
        summaryfixedstructreader.fixedstructreader_utmp_entries_miss,
        percent,
        wide = wide,
        widep = WIDEP,
    );
}

/// Print the various (optional) [`Summary`] caching and storage statistics
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_cache_stats(summary_opt: &SummaryOpt, color_choice: &ColorChoice) {
    if summary_opt.is_none() {
        return;
    }
    let summary: &Summary = match summary_opt.as_ref() {
        Some(summary_) => summary_,
        None => {
            e_err!("unexpected None from match summary_opt");
            return;
        }
    };
    // `Dummy` may occur for files without adequate read permissions
    if summary.readerdata.is_dummy() {
        return;
    }
    let wide: usize = summary
        .max_hit_miss()
        .to_string()
        .len();
    match &summary.readerdata {
        SummaryReaderData::Syslog((
            summaryblockreader,
            summarylinereader,
            summarysyslinereader,
            _summarysyslogprocessor,
        )) => {
            eprintln!("{}Processing Stores:", OPT_SUMMARY_PRINT_INDENT1);
            print_cache_stats_summaryblockreader(
                summaryblockreader,
                color_choice,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summarylinereader(
                summarylinereader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summarysyslinereader(
                summarysyslinereader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
        }
        SummaryReaderData::FixedStruct((
            summaryblockreader,
            summaryfixedstructreader,
        )) => {
            eprintln!("{}Processing Stores:", OPT_SUMMARY_PRINT_INDENT1);
            print_cache_stats_summaryfixedstructreader(
                summaryfixedstructreader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summaryblockreader(
                summaryblockreader,
                color_choice,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
        }
        SummaryReaderData::Etvx(_summaryevtxreader) => {}
        SummaryReaderData::Journal(_summaryjournalreader) => {}
        SummaryReaderData::Dummy => panic!("Unexpected SummaryReaderData::Dummy"),
    }
}

/// Print the (optional) various [`Summary`] drop error statistics
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_drop_stats(summary_opt: &SummaryOpt) {
    let summary: &Summary = match summary_opt {
        Some(ref summary) => summary,
        None => {
            return;
        }
    };
    if summary.readerdata.is_dummy() {
        return;
    }
    // force early return for Evtx or Journal
    // the `EvtxReader` and `JournalReader` do not use `BlockReader`
    match summary.filetype {
        FileType::Evtx => { return; }
        FileType::Journal => { return; }
        _ => {}
    }
    eprintln!("{}Processing Drops:", OPT_SUMMARY_PRINT_INDENT1);
    let wide: usize = summary
        .max_drop()
        .to_string()
        .len();
    match summary.blockreader() {
        Some(summaryblockreader) => {
            eprintln!(
                "{}streaming: BlockReader::drop_block()     : Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summaryblockreader.blockreader_blocks_dropped_ok,
                summaryblockreader.blockreader_blocks_dropped_err,
                wide = wide,
            );
        }
        None => {}
    }
    match &summary.readerdata {
        SummaryReaderData::Syslog(
            (
                _summaryblockreader,
                summarylinereader,
                summarysyslinereader,
                _summarysyslogreader,
            )
        ) => {
            eprintln!(
                "{}streaming: LineReader::drop_line()       : Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summarylinereader.linereader_drop_line_ok,
                summarylinereader.linereader_drop_line_errors,
                wide = wide,
            );
            eprintln!(
                "{}streaming: SyslineReader::drop_sysline() : Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summarysyslinereader.syslinereader_drop_sysline_ok,
                summarysyslinereader.syslinereader_drop_sysline_errors,
                wide = wide,
            );
        }
        SummaryReaderData::FixedStruct(
            (
                _summaryblockreader,
                summaryfixedstructreader,
            )
        ) => {
            eprintln!(
                "{}streaming: FixedStructReader::drop_entry()    : Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summaryfixedstructreader.fixedstructreader_drop_entry_ok,
                summaryfixedstructreader.fixedstructreader_drop_entry_errors,
                wide = wide,
            );
        }
        SummaryReaderData::Etvx(_summaryevtxreader) => {
            panic!("Unexpected SummaryReaderData::Etvx");
        }
        SummaryReaderData::Journal(_summaryjournalreader) => {
            panic!("Unexpected SummaryReaderData::Journal");
        }
        SummaryReaderData::Dummy => panic!("Unexpected SummaryReaderData::Dummy"),
    }
}

/// Print the [`Summary.error`], if any (one line).
///
/// [`Summary.error`]: s4lib::readers::summary::Summary
fn print_error_summary(
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_opt.as_ref() {
        Some(summary_) => match &summary_.error {
            Some(err_string) => {
                eprint!("{}Error: ", OPT_SUMMARY_PRINT_INDENT1);
                #[allow(clippy::single_match)]
                match print_colored_stderr(COLOR_ERROR, Some(*color_choice), err_string.as_bytes()) {
                    Ok(_) => {}
                    Err(e) => e_err!("print_colored_stderr: {:?}", e)
                }
                eprintln!();
            }
            None => {}
        },
        None => {}
    }
}

/// For one file, print the [`Summary`] and [`SummaryPrinted`]
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
/// [`SummaryPrinted`]: self::SummaryPrinted
#[allow(clippy::too_many_arguments)]
fn print_file_summary(
    path: &FPath,
    modified_time: &DateTimeLOpt,
    file_processing_result: Option<&FileProcessingResultBlockZero>,
    filetype: &FileType,
    logmessagetype: &LogMessageType,
    mimeguess: &MimeGuess,
    summary_opt: &SummaryOpt,
    summary_print_opt: &SummaryPrintedOpt,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprintln!();

    print_file_about(
        path,
        modified_time,
        file_processing_result,
        filetype,
        logmessagetype,
        mimeguess,
        summary_opt,
        color,
        color_choice,
    );
    // do not print any summary numbers for empty files (they should all be zero)
    if let Some(result) = file_processing_result {
        match result {
            FileProcessingResultBlockZero::FileErrEmpty => {
                return;
            }
            FileProcessingResultBlockZero::FileErrStub => {
                debug_panic!("result is FileErrStub");
                return;
            }
            _ => {}
        }
    }
    print_summary_opt_printed(summary_print_opt, summary_opt, color_choice);
    print_summary_opt_processed(summary_opt, color_choice);
    if OPT_SUMMARY_PRINT_CACHE_STATS {
        print_cache_stats(summary_opt, color_choice);
    }
    if OPT_SUMMARY_PRINT_DROP_STATS {
        print_drop_stats(summary_opt);
    }
    print_error_summary(summary_opt, color_choice);
}

/// Printing for CLI option `--summary`. Print each files'
/// [`Summary`] and [`SummaryPrinted`].
///
/// Helper function to function `processing_loop`.
///
/// [`Summary`]: s4lib::readers::summary::Summary
/// [`SummaryPrinted`]: self::SummaryPrinted
#[allow(clippy::too_many_arguments)]
fn print_all_files_summaries(
    map_pathid_path: &MapPathIdToFPath,
    map_pathid_modified_time: &MapPathIdToModifiedTime,
    map_pathid_file_processing_result: &MapPathIdToFileProcessingResultBlockZero,
    map_pathid_filetype: &MapPathIdToFileType,
    map_pathid_logmessagetype: &MapPathIdToLogMessageType,
    map_pathid_mimeguess: &MapPathIdToMimeGuess,
    map_pathid_color: &MapPathIdToColor,
    map_pathid_summary: &mut MapPathIdSummary,
    map_pathid_sumpr: &mut MapPathIdSummaryPrint,
    color_choice: &ColorChoice,
    color_default: &Color,
) {
    for (pathid, path) in map_pathid_path.iter() {
        let color: &Color = map_pathid_color
            .get(pathid)
            .unwrap_or(color_default);
        let modified_time: &DateTimeLOpt = map_pathid_modified_time.get(pathid)
            .unwrap_or(&DateTimeLOpt::None);
        let file_processing_result = map_pathid_file_processing_result.get(pathid);
        let filetype: &FileType = map_pathid_filetype
            .get(pathid)
            .unwrap_or(&FileType::Unknown);
        let logmessagetype: &LogMessageType = map_pathid_logmessagetype
            .get(pathid)
            .unwrap_or(&LogMessageType::Sysline);
        let mimeguess_default: MimeGuess = MimeGuess::from_ext("");
        let mimeguess: &MimeGuess = map_pathid_mimeguess
            .get(pathid)
            .unwrap_or(&mimeguess_default);
        let summary_opt: SummaryOpt = map_pathid_summary.remove(pathid);
        let summary_print_opt: SummaryPrintedOpt = map_pathid_sumpr.remove(pathid);
        print_file_summary(
            path,
            modified_time,
            file_processing_result,
            filetype,
            logmessagetype,
            mimeguess,
            &summary_opt,
            &summary_print_opt,
            color,
            color_choice,
        );
    }
}

/// Printing for CLI option `--summary`; print an entry for invalid files.
///
/// Helper function to function `processing_loop`.
fn print_files_processpathresult(
    map_pathid_result: &MapPathIdToProcessPathResultOrdered,
    color_choice: &ColorChoice,
    color_default: &Color,
    color_error: &Color,
) {
    // local helper
    fn print_(
        buffer: String,
        color_choice: &ColorChoice,
        color: &Color,
    ) {
        match print_colored_stderr(*color, Some(*color_choice), buffer.as_bytes()) {
            Ok(_) => {}
            Err(e) => e_err!("print_colored_stderr: {:?}", e)
        };
    }

    for (_pathid, result) in map_pathid_result.iter() {
        match result {
            ProcessPathResult::FileValid(path, mimeguess, _filetype) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
            }
            ProcessPathResult::FileErrNoPermissions(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(no permissions)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotSupported(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(not supported)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotAFile(path) => {
                print_(format!("File: {} ", path), color_choice, color_default);
                print_("(not a file)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotExist(path) => {
                print_(format!("File: {} ", path), color_choice, color_default);
                print_("(does not exist)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrLoadingLibrary(path, libname, filetype) => {
                print_(format!("File: {} {:?} ", path, filetype), color_choice, color_default);
                print_(format!("(failed to load shared library {:?})", libname), color_choice, color_error);
            }
            ProcessPathResult::FileErr(path, message) => {
                print_(format!("File: {} ", path), color_choice, color_default);
                print_(format!("({})", message), color_choice, color_error);
            }
        }
        eprintln!();
    }
}