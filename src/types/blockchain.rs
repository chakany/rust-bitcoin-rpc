//! Response types for the blockchain RPCs.
//!
//! Transcribed from the `RPCResult` blocks of `src/rpc/blockchain.cpp` in
//! Bitcoin Core v31.1. No struct rejects unknown fields, so a newer node adding
//! a field does not break deserialization.

use std::collections::BTreeMap;

/// Result of `getblockchaininfo`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetBlockchainInfo {
    /// Current network name.
    pub chain: String,
    /// The height of the most-work fully-validated chain.
    pub blocks: u64,
    /// The current number of headers we have validated. `-1` if unknown.
    pub headers: i64,
    /// The hash of the currently best block.
    #[cfg_attr(feature = "serde", serde(rename = "bestblockhash"))]
    pub best_block_hash: String,
    /// nBits: compact representation of the block difficulty target.
    pub bits: String,
    /// The difficulty target.
    pub target: String,
    /// The current difficulty.
    pub difficulty: f64,
    /// The block time, in seconds since the epoch.
    pub time: i64,
    /// The median block time, in seconds since the epoch.
    #[cfg_attr(feature = "serde", serde(rename = "mediantime"))]
    pub median_time: i64,
    /// Estimate of verification progress, in `[0..1]`.
    #[cfg_attr(feature = "serde", serde(rename = "verificationprogress"))]
    pub verification_progress: f64,
    /// Estimate of whether this node is in Initial Block Download mode.
    #[cfg_attr(feature = "serde", serde(rename = "initialblockdownload"))]
    pub initial_block_download: bool,
    /// Total amount of work in the active chain, in hexadecimal.
    pub chainwork: String,
    /// Estimated size of the block and undo files on disk.
    pub size_on_disk: u64,
    /// Whether the blocks are subject to pruning.
    pub pruned: bool,
    /// The first unpruned block. Only present when pruning is enabled.
    #[cfg_attr(feature = "serde", serde(default, rename = "pruneheight"))]
    pub prune_height: Option<u64>,
    /// Whether automatic pruning is enabled. Only present when pruning is enabled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub automatic_pruning: Option<bool>,
    /// Target size used by pruning. Only present when automatic pruning is enabled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub prune_target_size: Option<u64>,
    /// The signet block challenge. Only present on signet.
    #[cfg_attr(feature = "serde", serde(default, rename = "signet_challenge"))]
    pub signet_challenge: Option<String>,
    /// Any network and blockchain warnings.
    #[cfg_attr(feature = "serde", serde(default))]
    pub warnings: Vec<String>,
}

/// Result of `getblockheader` with `verbose` set to `true`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockHeader {
    /// The block hash (same as provided).
    pub hash: String,
    /// The number of confirmations, or `-1` if the block is not on the main chain.
    pub confirmations: i64,
    /// The block height or index.
    pub height: u64,
    /// The block version.
    pub version: i64,
    /// The block version formatted in hexadecimal.
    #[cfg_attr(feature = "serde", serde(rename = "versionHex"))]
    pub version_hex: String,
    /// The merkle root.
    #[cfg_attr(feature = "serde", serde(rename = "merkleroot"))]
    pub merkle_root: String,
    /// The block time, in seconds since the epoch.
    pub time: i64,
    /// The median block time, in seconds since the epoch.
    #[cfg_attr(feature = "serde", serde(rename = "mediantime"))]
    pub median_time: i64,
    /// The nonce.
    pub nonce: u64,
    /// nBits: compact representation of the block difficulty target.
    pub bits: String,
    /// The difficulty target.
    pub target: String,
    /// The difficulty.
    pub difficulty: f64,
    /// Expected number of hashes required to produce the current chain.
    pub chainwork: String,
    /// The number of transactions in the block.
    #[cfg_attr(feature = "serde", serde(rename = "nTx"))]
    pub n_tx: u64,
    /// The hash of the previous block, if available.
    #[cfg_attr(feature = "serde", serde(default, rename = "previousblockhash"))]
    pub previous_block_hash: Option<String>,
    /// The hash of the next block, if available.
    #[cfg_attr(feature = "serde", serde(default, rename = "nextblockhash"))]
    pub next_block_hash: Option<String>,
}

/// Coinbase transaction metadata, reported by `getblock` at verbosity 1 and above.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoinbaseTx {
    /// The coinbase transaction version.
    pub version: u64,
    /// The coinbase transaction's locktime (nLockTime).
    pub locktime: u64,
    /// The coinbase input's sequence number (nSequence).
    pub sequence: u64,
    /// The coinbase input's script.
    pub coinbase: String,
    /// The coinbase input's first (and only) witness stack element, if present.
    #[cfg_attr(feature = "serde", serde(default))]
    pub witness: Option<String>,
}

/// Result of `getblock` at verbosity 1: the block with transaction ids only.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    /// The block hash (same as provided).
    pub hash: String,
    /// The number of confirmations, or `-1` if the block is not on the main chain.
    pub confirmations: i64,
    /// The block size.
    pub size: u64,
    /// The block size excluding witness data.
    #[cfg_attr(feature = "serde", serde(rename = "strippedsize"))]
    pub stripped_size: u64,
    /// The block weight as defined in BIP 141.
    pub weight: u64,
    /// Coinbase transaction metadata.
    pub coinbase_tx: CoinbaseTx,
    /// The block height or index.
    pub height: u64,
    /// The block version.
    pub version: i64,
    /// The block version formatted in hexadecimal.
    #[cfg_attr(feature = "serde", serde(rename = "versionHex"))]
    pub version_hex: String,
    /// The merkle root.
    #[cfg_attr(feature = "serde", serde(rename = "merkleroot"))]
    pub merkle_root: String,
    /// The transaction ids.
    pub tx: Vec<String>,
    /// The block time, in seconds since the epoch.
    pub time: i64,
    /// The median block time, in seconds since the epoch.
    #[cfg_attr(feature = "serde", serde(rename = "mediantime"))]
    pub median_time: i64,
    /// The nonce.
    pub nonce: u64,
    /// nBits: compact representation of the block difficulty target.
    pub bits: String,
    /// The difficulty target.
    pub target: String,
    /// The difficulty.
    pub difficulty: f64,
    /// Expected number of hashes required to produce the chain up to this block,
    /// in hexadecimal.
    pub chainwork: String,
    /// The number of transactions in the block.
    #[cfg_attr(feature = "serde", serde(rename = "nTx"))]
    pub n_tx: u64,
    /// The hash of the previous block, if available.
    #[cfg_attr(feature = "serde", serde(default, rename = "previousblockhash"))]
    pub previous_block_hash: Option<String>,
    /// The hash of the next block, if available.
    #[cfg_attr(feature = "serde", serde(default, rename = "nextblockhash"))]
    pub next_block_hash: Option<String>,
}

/// One element of the `tx` array of a `getblock` verbosity-2 result.
///
/// Core documents these elements as an elision — "the transactions in the format
/// of the `getrawtransaction` RPC" — plus a `fee`. Only `fee` is modelled here;
/// use `getrawtransaction` for the full transaction body.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockTransaction {
    /// The transaction fee in BTC. Omitted if block undo data is not available.
    #[cfg_attr(feature = "serde", serde(default))]
    pub fee: Option<f64>,
}

/// Result of `getblock` at verbosity 2: the block with per-transaction detail.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockWithTxs {
    /// The block hash (same as provided).
    pub hash: String,
    /// The number of confirmations, or `-1` if the block is not on the main chain.
    pub confirmations: i64,
    /// The block size.
    pub size: u64,
    /// The block size excluding witness data.
    #[cfg_attr(feature = "serde", serde(rename = "strippedsize"))]
    pub stripped_size: u64,
    /// The block weight as defined in BIP 141.
    pub weight: u64,
    /// Coinbase transaction metadata.
    pub coinbase_tx: CoinbaseTx,
    /// The block height or index.
    pub height: u64,
    /// The block version.
    pub version: i64,
    /// The block version formatted in hexadecimal.
    #[cfg_attr(feature = "serde", serde(rename = "versionHex"))]
    pub version_hex: String,
    /// The merkle root.
    #[cfg_attr(feature = "serde", serde(rename = "merkleroot"))]
    pub merkle_root: String,
    /// The transactions in the block.
    pub tx: Vec<BlockTransaction>,
    /// The block time, in seconds since the epoch.
    pub time: i64,
    /// The median block time, in seconds since the epoch.
    #[cfg_attr(feature = "serde", serde(rename = "mediantime"))]
    pub median_time: i64,
    /// The nonce.
    pub nonce: u64,
    /// nBits: compact representation of the block difficulty target.
    pub bits: String,
    /// The difficulty target.
    pub target: String,
    /// The difficulty.
    pub difficulty: f64,
    /// Expected number of hashes required to produce the chain up to this block,
    /// in hexadecimal.
    pub chainwork: String,
    /// The number of transactions in the block.
    #[cfg_attr(feature = "serde", serde(rename = "nTx"))]
    pub n_tx: u64,
    /// The hash of the previous block, if available.
    #[cfg_attr(feature = "serde", serde(default, rename = "previousblockhash"))]
    pub previous_block_hash: Option<String>,
    /// The hash of the next block, if available.
    #[cfg_attr(feature = "serde", serde(default, rename = "nextblockhash"))]
    pub next_block_hash: Option<String>,
}

/// One known tip in the block tree, as reported by `getchaintips`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChainTip {
    /// Height of the chain tip.
    pub height: u64,
    /// Block hash of the tip.
    pub hash: String,
    /// Zero for the main chain, otherwise the length of the branch connecting
    /// the tip to the main chain.
    #[cfg_attr(feature = "serde", serde(rename = "branchlen"))]
    pub branch_len: u64,
    /// Status of the chain: one of `invalid`, `headers-only`, `valid-headers`,
    /// `valid-fork` or `active`.
    pub status: String,
}

/// Signalling statistics for a BIP 9 deployment.
///
/// Only present for the `started` and `locked_in` statuses.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bip9Statistics {
    /// The length in blocks of the signalling period.
    pub period: u64,
    /// The number of blocks with the version bit set required to activate the
    /// feature. Only present for the `started` status.
    #[cfg_attr(feature = "serde", serde(default))]
    pub threshold: Option<u64>,
    /// The number of blocks elapsed since the beginning of the current period.
    pub elapsed: u64,
    /// The number of blocks with the version bit set in the current period.
    pub count: u64,
    /// `false` if there are not enough blocks left in this period to pass the
    /// activation threshold. Only present for the `started` status.
    #[cfg_attr(feature = "serde", serde(default))]
    pub possible: Option<bool>,
}

/// BIP 9 softfork status of a deployment.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bip9Info {
    /// The bit (0-28) in the block version field used to signal this softfork.
    /// Only present for the `started` and `locked_in` statuses.
    #[cfg_attr(feature = "serde", serde(default))]
    pub bit: Option<u64>,
    /// The minimum median time past of a block at which the bit gains its meaning.
    pub start_time: i64,
    /// The median time past of a block at which the deployment is considered
    /// failed if not yet locked in.
    pub timeout: i64,
    /// Minimum height of blocks for which the rules may be enforced.
    pub min_activation_height: u64,
    /// Status of the deployment at the specified block: one of `defined`,
    /// `started`, `locked_in`, `active` or `failed`.
    pub status: String,
    /// Height of the first block to which the status applies.
    pub since: u64,
    /// Status of the deployment at the next block.
    pub status_next: String,
    /// Numeric statistics about signalling for the softfork. Only present for
    /// the `started` and `locked_in` statuses.
    #[cfg_attr(feature = "serde", serde(default))]
    pub statistics: Option<Bip9Statistics>,
    /// Blocks that signalled, as `#`, and blocks that did not, as `-`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub signalling: Option<String>,
}

/// State of one consensus deployment.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Deployment {
    /// One of `buried` or `bip9`.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub deployment_type: String,
    /// Height of the first block which the rules are or will be enforced. Only
    /// present for the `buried` type, or the `bip9` type with `active` status.
    #[cfg_attr(feature = "serde", serde(default))]
    pub height: Option<u64>,
    /// `true` if the rules are enforced for the mempool and the next block.
    pub active: bool,
    /// Status of the BIP 9 softfork. Only present for the `bip9` type.
    #[cfg_attr(feature = "serde", serde(default))]
    pub bip9: Option<Bip9Info>,
}

/// Result of `getdeploymentinfo`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeploymentInfo {
    /// Requested block hash (or tip).
    pub hash: String,
    /// Requested block height (or tip).
    pub height: u64,
    /// Script verify flags for the block.
    pub script_flags: Vec<String>,
    /// Deployment state, keyed by deployment name.
    pub deployments: BTreeMap<String, Deployment>,
}

/// The output script of an unspent transaction output.
///
/// Named apart from the raw-transaction script type because `gettxout` documents
/// its own `scriptPubKey` result independently.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxOutScriptPubKey {
    /// Disassembly of the output script.
    pub asm: String,
    /// Inferred descriptor for the output.
    pub desc: String,
    /// The raw output script bytes, hex-encoded.
    pub hex: String,
    /// The type, e.g. `pubkeyhash`.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub script_type: String,
    /// The Bitcoin address. Only present if a well-defined address exists.
    #[cfg_attr(feature = "serde", serde(default))]
    pub address: Option<String>,
}

/// Result of `gettxout` when the output was found.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxOut {
    /// The hash of the block at the tip of the chain.
    #[cfg_attr(feature = "serde", serde(rename = "bestblock"))]
    pub best_block: String,
    /// The number of confirmations.
    pub confirmations: i64,
    /// The transaction value in BTC.
    pub value: f64,
    /// The output script.
    #[cfg_attr(feature = "serde", serde(rename = "scriptPubKey"))]
    pub script_pub_key: TxOutScriptPubKey,
    /// Coinbase or not.
    pub coinbase: bool,
}

/// A block hash paired with its height, as returned by `waitfornewblock` and
/// `waitforblockheight`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockHashAndHeight {
    /// The blockhash.
    pub hash: String,
    /// Block height.
    pub height: u64,
}
