#![doc = include_str!("../README.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
#![warn(missing_docs)]

pub mod config;
pub mod data_channel;
pub mod error;
mod retry;
pub mod session;
pub mod websocket_channel;
