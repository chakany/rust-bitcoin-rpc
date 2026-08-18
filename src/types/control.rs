//! Request and response types for the control RPCs.
//!
//! Transcribed from the `RPCResult` block of `getrpcinfo` in
//! `src/rpc/server.cpp` in Bitcoin Core v31.1. `help`, `stop` and `uptime`
//! return bare strings or numbers and need no dedicated type. No response
//! struct here rejects unknown fields, so a newer node adding a field does
//! not break deserialization.

/// Result of `getrpcinfo`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetRpcInfo {
    /// All active commands.
    pub active_commands: Vec<RpcInfoCommand>,
    /// The complete file path to the debug log.
    pub logpath: String,
}

/// Information about one active command, held in
/// [`GetRpcInfo::active_commands`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RpcInfoCommand {
    /// The name of the RPC command.
    pub method: String,
    /// The running time in microseconds.
    ///
    /// Measured as `SteadyClock::now() - info.start` for a command still
    /// executing (`server.cpp:203`), so it is never negative.
    pub duration: u64,
}
