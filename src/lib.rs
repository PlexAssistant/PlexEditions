//! Writes a movie's edition title through the Plex HTTP API.
//!
//! The `editions` binary is the standalone entry; PlexAssistant calls
//! these same pieces as a library, so the logic lives here once.

pub mod api;
pub mod config;
pub mod error;
pub mod i18n;
pub mod model;
pub mod modules;
pub mod run;
pub mod startup;
pub mod ws;
