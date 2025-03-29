//! Implements a data channel for interactive session.

/// TODO: Add a description of the data channel.
#[mockall::automock]
pub trait DataChannel {}

/// TODO: Add a description of the default data channel.
#[derive(Debug, Default)]
pub struct DefaultDataChannel {}

impl DataChannel for DefaultDataChannel {}
