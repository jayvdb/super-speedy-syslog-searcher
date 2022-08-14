// src/lib.rs

//! _Super Speedy Syslog Searcher_ library, _s4lib_!
//!
//! This is the library implementation used by binary program _s4_.
//! This library is documented in part to have a presence on _crates.io_ and
//! _docs.rs_.

pub mod common;

pub mod data;

#[doc(hidden)]
pub mod printer_debug;

pub mod printer;

pub mod readers;

#[cfg(test)]
pub mod tests;

#[doc(hidden)]
pub fn main() {}
