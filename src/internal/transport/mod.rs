//! Transport layer for communicating with Claude Code CLI

pub mod subprocess;
mod trait_def;

pub use subprocess::SubprocessTransport;
pub use trait_def::Transport;
