//! Typed RPC method traits for the async client.
//!
//! Each trait is blanket-implemented for every [`RpcCallAsync`], so bringing one
//! into scope adds its methods to any client — including `&dyn RpcCallAsync`.
//!
//! Method names, parameters and result types mirror [`crate::sync`] exactly;
//! only the return-type wrapper differs.

use std::collections::BTreeMap;
use std::future::Future;

use serde_json::json;

use super::call::{RpcCallAsync, RpcCallAsyncExt};
use crate::Result;
use crate::params::positional;
use crate::types::{
    Block, BlockHashAndHeight, BlockHeader, BlockTemplate, BlockTemplateRequest, BlockWithTxs,
    ChainTip, DeploymentInfo, GetBlockchainInfo, GetMempoolInfo, GetMiningInfo, GetNetTotals,
    GetNetworkInfo, GetRawMempoolSequence, MempoolEntry, PeerInfo, TxOut,
};

/// Blockchain RPCs.
pub trait BlockchainRpc: RpcCallAsync {
    /// Returns an object containing various state info regarding blockchain
    /// processing.
    fn get_blockchain_info(&self) -> impl Future<Output = Result<GetBlockchainInfo>> + Send + '_ {
        self.call("getblockchaininfo", positional(vec![]))
    }

    /// Returns the hash of the best (tip) block in the most-work
    /// fully-validated chain.
    fn get_best_block_hash(&self) -> impl Future<Output = Result<String>> + Send + '_ {
        self.call("getbestblockhash", positional(vec![]))
    }

    /// Returns the height of the most-work fully-validated chain. The genesis
    /// block has height 0.
    fn get_block_count(&self) -> impl Future<Output = Result<u64>> + Send + '_ {
        self.call("getblockcount", positional(vec![]))
    }

    /// Returns the hash of the block in the best-block-chain at `height`.
    fn get_block_hash(&self, height: u32) -> impl Future<Output = Result<String>> + Send + '_ {
        self.call("getblockhash", positional(vec![json!(height)]))
    }

    /// Returns the serialized, hex-encoded data for the block `hash`.
    fn get_block_hex(&self, hash: &str) -> impl Future<Output = Result<String>> + Send + '_ {
        self.call("getblock", positional(vec![json!(hash), json!(0)]))
    }

    /// Returns information about the block `hash`, with transaction ids only.
    fn get_block(&self, hash: &str) -> impl Future<Output = Result<Block>> + Send + '_ {
        self.call("getblock", positional(vec![json!(hash), json!(1)]))
    }

    /// Returns information about the block `hash` and about each of its
    /// transactions.
    fn get_block_with_txs(
        &self,
        hash: &str,
    ) -> impl Future<Output = Result<BlockWithTxs>> + Send + '_ {
        self.call("getblock", positional(vec![json!(hash), json!(2)]))
    }

    /// Returns information about the block header of block `hash`.
    fn get_block_header(
        &self,
        hash: &str,
    ) -> impl Future<Output = Result<BlockHeader>> + Send + '_ {
        self.call("getblockheader", positional(vec![json!(hash), json!(true)]))
    }

    /// Returns the serialized, hex-encoded data for the block header of block
    /// `hash`.
    fn get_block_header_hex(&self, hash: &str) -> impl Future<Output = Result<String>> + Send + '_ {
        self.call(
            "getblockheader",
            positional(vec![json!(hash), json!(false)]),
        )
    }

    /// Returns information about all known tips in the block tree, including
    /// the main chain as well as orphaned branches.
    fn get_chain_tips(&self) -> impl Future<Output = Result<Vec<ChainTip>>> + Send + '_ {
        self.call("getchaintips", positional(vec![]))
    }

    /// Returns the proof-of-work difficulty as a multiple of the minimum
    /// difficulty.
    fn get_difficulty(&self) -> impl Future<Output = Result<f64>> + Send + '_ {
        self.call("getdifficulty", positional(vec![]))
    }

    /// Returns an object containing various state info regarding deployments of
    /// consensus changes, at `hash` or at the current chain tip.
    fn get_deployment_info(
        &self,
        hash: Option<&str>,
    ) -> impl Future<Output = Result<DeploymentInfo>> + Send + '_ {
        self.call("getdeploymentinfo", positional(vec![json!(hash)]))
    }

    /// Returns details about the unspent transaction output `n` of `txid`, or
    /// `None` if it is not in the UTXO set.
    ///
    /// `include_mempool` defaults to `true` on the node; note that an output
    /// spent in the mempool then does not appear.
    fn get_tx_out(
        &self,
        txid: &str,
        n: u32,
        include_mempool: Option<bool>,
    ) -> impl Future<Output = Result<Option<TxOut>>> + Send + '_ {
        self.call(
            "gettxout",
            positional(vec![json!(txid), json!(n), json!(include_mempool)]),
        )
    }

    /// Waits for any new block and returns its hash and height.
    ///
    /// `timeout_ms` of `None` or `0` means no timeout. `current_tip` makes the
    /// node wait for the chain tip to differ from that hash, which is more
    /// reliable than letting it sample the tip itself.
    fn wait_for_new_block(
        &self,
        timeout_ms: Option<u64>,
        current_tip: Option<&str>,
    ) -> impl Future<Output = Result<BlockHashAndHeight>> + Send + '_ {
        self.call(
            "waitfornewblock",
            positional(vec![json!(timeout_ms), json!(current_tip)]),
        )
    }

    /// Waits for the chain to reach at least `height` and returns the hash and
    /// height of the current tip.
    ///
    /// `timeout_ms` of `None` or `0` means no timeout.
    fn wait_for_block_height(
        &self,
        height: u32,
        timeout_ms: Option<u64>,
    ) -> impl Future<Output = Result<BlockHashAndHeight>> + Send + '_ {
        self.call(
            "waitforblockheight",
            positional(vec![json!(height), json!(timeout_ms)]),
        )
    }
}

impl<T: RpcCallAsync + ?Sized> BlockchainRpc for T {}

/// Mempool RPCs.
pub trait MempoolRpc: RpcCallAsync {
    /// Returns details on the active state of the TX memory pool.
    fn get_mempool_info(&self) -> impl Future<Output = Result<GetMempoolInfo>> + Send + '_ {
        self.call("getmempoolinfo", positional(vec![]))
    }

    /// Returns all transaction ids in the mempool as a list of transaction
    /// ids.
    fn get_raw_mempool(&self) -> impl Future<Output = Result<Vec<String>>> + Send + '_ {
        self.call("getrawmempool", positional(vec![json!(false)]))
    }

    /// Returns all transactions in the mempool, keyed by transaction id, with
    /// their full mempool entry data.
    fn get_raw_mempool_verbose(
        &self,
    ) -> impl Future<Output = Result<BTreeMap<String, MempoolEntry>>> + Send + '_ {
        self.call("getrawmempool", positional(vec![json!(true)]))
    }

    /// Returns the transaction ids in the mempool together with the mempool
    /// sequence number, as of the moment the list was generated.
    fn get_raw_mempool_with_sequence(
        &self,
    ) -> impl Future<Output = Result<GetRawMempoolSequence>> + Send + '_ {
        self.call("getrawmempool", positional(vec![json!(false), json!(true)]))
    }

    /// Returns mempool data for the given transaction `txid`.
    fn get_mempool_entry(
        &self,
        txid: &str,
    ) -> impl Future<Output = Result<MempoolEntry>> + Send + '_ {
        self.call("getmempoolentry", positional(vec![json!(txid)]))
    }
}

impl<T: RpcCallAsync + ?Sized> MempoolRpc for T {}

/// Network RPCs.
pub trait NetworkRpc: RpcCallAsync {
    /// Returns an object containing various state info regarding P2P
    /// networking.
    fn get_network_info(&self) -> impl Future<Output = Result<GetNetworkInfo>> + Send + '_ {
        self.call("getnetworkinfo", positional(vec![]))
    }

    /// Returns data about each connected network peer as a json array of
    /// objects.
    fn get_peer_info(&self) -> impl Future<Output = Result<Vec<PeerInfo>>> + Send + '_ {
        self.call("getpeerinfo", positional(vec![]))
    }

    /// Returns the number of connections to other nodes.
    fn get_connection_count(&self) -> impl Future<Output = Result<u64>> + Send + '_ {
        self.call("getconnectioncount", positional(vec![]))
    }

    /// Returns information about network traffic, including bytes in, bytes
    /// out, and current system time.
    fn get_net_totals(&self) -> impl Future<Output = Result<GetNetTotals>> + Send + '_ {
        self.call("getnettotals", positional(vec![]))
    }

    /// Attempts to add or remove `node` from the addnode list, or try a
    /// connection to it once.
    ///
    /// `command` is one of `"add"`, `"remove"` or `"onetry"`.
    fn add_node(
        &self,
        node: &str,
        command: &str,
        v2transport: Option<bool>,
    ) -> impl Future<Output = Result<()>> + Send + '_ {
        self.call(
            "addnode",
            positional(vec![json!(node), json!(command), json!(v2transport)]),
        )
    }

    /// Immediately disconnects from the specified peer node.
    ///
    /// Strictly one of `address` and `node_id` can be provided to identify
    /// the node.
    fn disconnect_node(
        &self,
        address: Option<&str>,
        node_id: Option<u64>,
    ) -> impl Future<Output = Result<()>> + Send + '_ {
        self.call(
            "disconnectnode",
            positional(vec![json!(address), json!(node_id)]),
        )
    }
}

impl<T: RpcCallAsync + ?Sized> NetworkRpc for T {}

/// Mining RPCs.
pub trait MiningRpc: RpcCallAsync {
    /// Returns a json object containing mining-related information.
    fn get_mining_info(&self) -> impl Future<Output = Result<GetMiningInfo>> + Send + '_ {
        self.call("getmininginfo", positional(vec![]))
    }

    /// Returns data needed to construct a block to work on.
    ///
    /// `request` is sent as the single `template_request` object argument; see
    /// BIPs 22, 23, 9 and 145 for the full specification. Only the default
    /// `"template"` mode is modelled — `"proposal"` mode returns a different
    /// result shape.
    fn get_block_template(
        &self,
        request: &BlockTemplateRequest,
    ) -> impl Future<Output = Result<BlockTemplate>> + Send + '_ {
        // Serialize before entering the async block so only `self` (not
        // `request`, which may be dropped by the caller before the future
        // resolves in some usages) needs to outlive the returned future.
        let params = serde_json::to_value(request).map(|v| positional(vec![v]));
        async move { self.call("getblocktemplate", params?).await }
    }

    /// Attempts to submit new block `hex` to the network.
    ///
    /// Returns `None` if the block was accepted, or a rejection-reason string
    /// otherwise, per BIP 22.
    fn submit_block(&self, hex: &str) -> impl Future<Output = Result<Option<String>>> + Send + '_ {
        self.call("submitblock", positional(vec![json!(hex)]))
    }

    /// Decodes the given `hex` as a header and submits it as a candidate chain
    /// tip if valid. Throws when the header is invalid.
    fn submit_header(&self, hex: &str) -> impl Future<Output = Result<()>> + Send + '_ {
        self.call("submitheader", positional(vec![json!(hex)]))
    }

    /// Returns the estimated network hashes per second based on the last
    /// `nblocks` blocks, or since the last difficulty change if `-1`.
    ///
    /// `height` estimates the network speed at the time a certain block was
    /// found instead of using the current tip.
    fn get_network_hash_ps(
        &self,
        nblocks: Option<i64>,
        height: Option<i64>,
    ) -> impl Future<Output = Result<f64>> + Send + '_ {
        self.call(
            "getnetworkhashps",
            positional(vec![json!(nblocks), json!(height)]),
        )
    }
}

impl<T: RpcCallAsync + ?Sized> MiningRpc for T {}
