//! Runtime adapter boundaries for the Rust conversion.
//!
//! This crate owns integration-facing contracts that are intentionally kept out
//! of `iw2wth_core`: input polling, rendering, audio, assets, time, and the
//! eventual runtime shell. Adapter code may translate those concerns into core
//! data, but core gameplay code should not depend on this crate.

pub mod assets;
pub mod audio;
pub mod harness;
pub mod input;
pub mod map;
pub mod render;
pub mod shell;
pub mod tiles;
pub mod time;
