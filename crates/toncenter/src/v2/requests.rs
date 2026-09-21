//! Requests for the TON Center v2 wire contract.

use super::stack::{LegacyStackEntry, TvmStackEntry};
use super::{BoolInput, Int32Input, Int64Input};
use serde::{Deserialize, Serialize};

/// Empty request body for endpoints that require no parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EmptyRequest {}

/// Request containing a single address parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct AddressRequest {
    /// Account to query, in raw workchain:hex or user-friendly base64 form.
    pub address: String,
}

/// Parameters for the `AddressWithSeqno` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct AddressWithSeqnoRequest {
    /// Account to query, in raw workchain:hex or user-friendly base64 form.
    pub address: String,
    /// Masterchain block sequence number. Query the account state as it was at this block height.
    /// If omitted, returns the current state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<Int32Input>,
}

/// Parameters for the Seqno API operation. Optional fields are omitted rather than sent as null;
/// the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SeqnoRequest {
    /// Block sequence number. For account and configuration queries this is a masterchain height;
    /// omit to query the latest state.
    pub seqno: Int32Input,
}

/// Parameters for the `DetectAddress` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
pub type DetectAddressRequest = AddressRequest;

/// Parameters for the `DetectHash` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct DetectHashRequest {
    /// Hash value to detect encoding formats for.
    pub hash: String,
}

/// Parameters for the `PackAddress` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
pub type PackAddressRequest = AddressRequest;

/// Parameters for the `UnpackAddress` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
pub type UnpackAddressRequest = AddressRequest;

/// Parameters for the `AddressInformation` API operation. Optional fields are omitted rather than
/// sent as null; the server owns semantic validation.
pub type AddressInformationRequest = AddressWithSeqnoRequest;

/// Parameters for the `ShardAccountCell` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
pub type ShardAccountCellRequest = AddressWithSeqnoRequest;

/// Parameters for the `ExtendedAddressInformation` API operation. Optional fields are omitted rather
/// than sent as null; the server owns semantic validation.
pub type ExtendedAddressInformationRequest = AddressWithSeqnoRequest;

/// Parameters for the `WalletInformation` API operation. Optional fields are omitted rather than
/// sent as null; the server owns semantic validation.
pub type WalletInformationRequest = AddressWithSeqnoRequest;

/// Parameters for the `AddressBalance` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
pub type AddressBalanceRequest = AddressWithSeqnoRequest;

/// Parameters for the `AddressState` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
pub type AddressStateRequest = AddressWithSeqnoRequest;

/// Parameters for the `TokenData` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
pub type TokenDataRequest = AddressWithSeqnoRequest;

/// Parameters for the `DnsResolve` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct DnsResolveRequest {
    /// Account to query, in raw workchain:hex or user-friendly base64 form.
    pub address: String,
    /// Domain name to resolve; omission uses the empty name at the supplied resolver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// DNS category selector. The pinned C++ implementation accepts but ignores this
    /// field and passes the zero category (all records) to `TONLib`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Maximum resolver recursion depth, not a cache lifetime in seconds. Must be non-negative;
    /// omission passes zero to `TONLib`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i32>,
    /// Masterchain block sequence number for historical resolution; must be positive when
    /// supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<i32>,
}

/// Parameters for the `MasterchainInfo` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
pub type MasterchainInfoRequest = EmptyRequest;

/// Parameters for the `MasterchainBlockSignatures` API operation. Optional fields are omitted rather
/// than sent as null; the server owns semantic validation.
pub type MasterchainBlockSignaturesRequest = SeqnoRequest;

/// Parameters for the `ShardBlockProof` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ShardBlockProofRequest {
    /// Workchain ID: `-1` for masterchain, `0` for basechain. Most user transactions are on
    /// workchain `0`.
    pub workchain: Int32Input,
    /// Shard identifier. A signed 64-bit integer. Masterchain uses -9223372036854775808.
    pub shard: Int64Input,
    /// Shardchain block sequence number. Identifies a specific block within the given workchain
    /// and shard. Use together with `workchain` and `shard` to uniquely identify a block.
    pub seqno: Int32Input,
    /// Starting masterchain block for proof generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_seqno: Option<Int32Input>,
}

/// Parameters for the `ConsensusBlock` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
pub type ConsensusBlockRequest = EmptyRequest;

/// Request to find a block by workchain, shard, and either `seqno`, lt, or unixtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct LookupBlockRequest {
    /// Workchain ID: `-1` for masterchain, `0` for basechain. Most user transactions are on
    /// workchain `0`.
    pub workchain: Int32Input,
    /// Shard identifier. A signed 64-bit integer. Masterchain uses -9223372036854775808.
    pub shard: Int64Input,
    /// Block sequence number. For account and configuration queries this is a masterchain height;
    /// omit to query the latest state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<Int32Input>,
    /// Logical time of the transaction. Use together with its hash for pagination and lookups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lt: Option<Int64Input>,
    /// Unix timestamp to look up.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unixtime: Option<Int32Input>,
}

/// Parameters for the Shards API operation. Optional fields are omitted rather than sent as null;
/// the server owns semantic validation.
pub type ShardsRequest = SeqnoRequest;

/// Parameters for the `BlockData` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct BlockDataRequest {
    /// Workchain ID: `-1` for masterchain, `0` for basechain. Most user transactions are on
    /// workchain `0`.
    pub workchain: Int32Input,
    /// Shard identifier. A signed 64-bit integer. Masterchain uses -9223372036854775808.
    pub shard: Int64Input,
    /// Block sequence number. For account and configuration queries this is a masterchain height;
    /// omit to query the latest state.
    pub seqno: Int32Input,
    /// Block root cell hash, accepted as 64 hexadecimal characters or base64.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_hash: Option<String>,
    /// Hash of the serialized block `BoC`, accepted as hexadecimal or base64.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_hash: Option<String>,
    /// Select archival liteservers for old data. Omission lets the server choose its default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archival: Option<BoolInput>,
}

/// Parameters for the `BlockHeader` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct BlockHeaderRequest {
    /// Workchain identifier: -1 for masterchain, 0 for the base workchain.
    pub workchain: Int32Input,
    /// Signed 64-bit shard identifier. The full shard uses -9223372036854775808; preserve the
    /// decimal string representation.
    pub shard: Int64Input,
    /// Block sequence number. For account and configuration queries this is a masterchain height;
    /// omit to query the latest state.
    pub seqno: Int32Input,
    /// Root hash of the block. Together with `file_hash`, uniquely identifies a block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_hash: Option<String>,
    /// Hash of the serialized block data. Together with `root_hash`, uniquely identifies a block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_hash: Option<String>,
}

/// Parameters for the `OutMsgQueueSize` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
pub type OutMsgQueueSizeRequest = EmptyRequest;

/// Request to fetch transactions from a specific block with optional pagination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct BlockTransactionsRequest {
    /// Workchain ID: `-1` for masterchain, `0` for basechain. Most user transactions are on
    /// workchain `0`.
    pub workchain: Int32Input,
    /// Shard identifier. A signed 64-bit integer. Masterchain uses -9223372036854775808.
    pub shard: Int64Input,
    /// Block sequence number. For account and configuration queries this is a masterchain height;
    /// omit to query the latest state.
    pub seqno: Int32Input,
    /// Root hash of the block. Together with `file_hash`, uniquely identifies a block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_hash: Option<String>,
    /// Hash of the serialized block data. Together with `root_hash`, uniquely identifies a block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_hash: Option<String>,
    /// Return items after this logical time (for pagination).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_lt: Option<Int64Input>,
    /// Return items after this hash (for pagination within same `lt`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_hash: Option<String>,
    /// Maximum number of items to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<Int64Input>,
}

/// Parameters for the `BlockTransactionsExt` API operation. Optional fields are omitted rather than
/// sent as null; the server owns semantic validation.
pub type BlockTransactionsExtRequest = BlockTransactionsRequest;

/// Request to fetch transaction history for an account with optional pagination and filtering
/// parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct TransactionsRequest {
    /// Account to query, in raw workchain:hex or user-friendly base64 form.
    pub address: String,
    /// Logical time of the transaction. Use together with its hash for pagination and lookups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lt: Option<Int64Input>,
    /// Hash of the starting transaction for pagination. Use together with `lt` from a previous
    /// response's `transaction_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Stop returning items at this logical time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_lt: Option<Int64Input>,
    /// Whether to use archival nodes for old data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archival: Option<BoolInput>,
    /// Maximum number of items to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<Int64Input>,
}

/// Parameters for the `TryLocateTx` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct TryLocateTxRequest {
    /// Source account of the internal message, in raw or user-friendly address format.
    pub source: String,
    /// Recipient address.
    pub destination: String,
    /// Logical time when this message was created. Use with source and destination to uniquely
    /// identify a message.
    pub created_lt: Int64Input,
}

/// Parameters for the `TryLocateResultTx` API operation. Optional fields are omitted rather than
/// sent as null; the server owns semantic validation.
pub type TryLocateResultTxRequest = TryLocateTxRequest;

/// Parameters for the `TryLocateSourceTx` API operation. Optional fields are omitted rather than
/// sent as null; the server owns semantic validation.
pub type TryLocateSourceTxRequest = TryLocateTxRequest;

/// Parameters for the `ConfigParam` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ConfigParamRequest {
    /// Configuration parameter number. Each number controls different blockchain settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<Int32Input>,
    /// Configuration parameter number; alternative spelling of `config_id`. Supply at least one of
    /// them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<Int32Input>,
    /// Masterchain block sequence number (block height). Used to query blockchain config at a
    /// specific block. If omitted, the latest masterchain block is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<Int32Input>,
}

/// Parameters for the `ConfigAll` API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ConfigAllRequest {
    /// Masterchain block sequence number (block height). Used to query blockchain config at a
    /// specific block. If omitted, the latest masterchain block is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<Int32Input>,
}

/// Parameters for the Libraries API operation. Optional fields are omitted rather than sent as
/// null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct LibrariesRequest {
    /// Array of library cell hashes to retrieve. Each hash is a 256-bit value in hex (64 chars) or
    /// base64 (44 chars) format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libraries: Option<Vec<String>>,
}

/// Request to broadcast a signed message. The boc field contains the base64-encoded external
/// message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SendBocRequest {
    /// Serialized external message in `BoC` format, base64 encoded.
    pub boc: String,
}

/// Request to estimate transaction fees. Include the target address, message body, and optionally
/// init code/data for contract deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EstimateFeeRequest {
    /// Account to query, in raw workchain:hex or user-friendly base64 form.
    pub address: String,
    /// Message body in `BoC` format, base64 encoded.
    pub body: String,
    /// Contract code for deployment messages, `BoC` format, base64 encoded. Omit for regular
    /// transfers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init_code: Option<String>,
    /// Initial contract storage for deployment, `BoC` format, base64 encoded. Omit for regular
    /// transfers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init_data: Option<String>,
    /// Set to `true` to skip signature verification during fee estimation; otherwise `false`.
    /// Useful when a real signature is not yet available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_chksig: Option<bool>,
}

/// Parameters for the `RunGetMethodStd` API operation. Optional fields are omitted rather than sent
/// as null; the server owns semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct RunGetMethodStdRequest {
    /// Account to query, in raw workchain:hex or user-friendly base64 form.
    pub address: String,
    /// The get method name (e.g., `seqno`, `get_wallet_data`) or its numeric ID.
    pub method: Int32Input,
    /// Input arguments as a TVM stack. Each entry is a typed object with `@type` discriminator:
    /// `tvm.stackEntryNumber` (decimal string), `tvm.stackEntryCell` (base64 `BoC`),
    /// `tvm.stackEntrySlice` (base64 `BoC`), `tvm.stackEntryTuple`, or `tvm.stackEntryList`.
    pub stack: Vec<TvmStackEntry>,
    /// Masterchain block sequence number. Run the get method against the contract state at this
    /// specific block height. If omitted, uses the current state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<i64>,
}

/// Request to execute a smart contract get method. Specify address, method name/ID, and input
/// stack. `S` defaults to TON Center stack entries; custom servers can use an extended entry type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct RunGetMethodRequest<S = LegacyStackEntry> {
    /// Account to query, in raw workchain:hex or user-friendly base64 form.
    pub address: String,
    /// The get method name (e.g., `seqno`, `get_wallet_data`) or its numeric ID.
    pub method: Int32Input,
    /// Input arguments as `[type, value]` pairs. Numeric values accept decimal or hexadecimal
    /// strings and signed 64-bit integers. Use `cell` or `slice` with a `{ "bytes": "..." }`
    /// object, or `tvm.Cell` or `tvm.Slice` with a base64 `BoC` string. Nested tuple and list
    /// elements use standard `TONLib` stack entries.
    pub stack: Vec<S>,
    /// Masterchain block sequence number. Run the get method against the contract state at this
    /// specific block height. If omitted, uses the current state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<i32>,
}

/// A typed call through `/jsonRPC`. The C++ proxy ignores JSON-RPC metadata and
/// returns an ordinary `TONLib` envelope; do not require the ID to be echoed.
///
/// `S` defaults to TON Center stack entries and can represent custom server extensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JsonRpcRequest<S = LegacyStackEntry> {
    /// Method and its matching object-shaped parameters.
    #[serde(flatten)]
    pub call: JsonRpcCall<S>,
    /// Optional protocol metadata. C++ ignores its contents; `"2.0"` is conventional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<serde_json::Value>,
    /// Optional correlation metadata. C++ ignores this value and does not echo it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

/// Case-sensitive JSON-RPC method paired with its concrete parameter object.
///
/// This models canonical calls. The server also tolerates omitted or non-object
/// params for parameterless methods; transports should send an empty object.
/// `S` selects the legacy get-method stack entry type and defaults to the v2 contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "method", content = "params")]
pub enum JsonRpcCall<S = LegacyStackEntry> {
    /// Validates an address and returns it in all standard formats. Use this to convert between address formats or to validate user input. Returns raw format (0:abc), base64 bounceable (EQ), base64 non-bounceable (UQ), and URL-safe variants.
    #[serde(rename = "detectAddress")]
    DetectAddress(DetectAddressRequest),
    /// Validates a hash and returns it in all standard formats. Use this to convert between hex (64 chars) and base64 (44 chars) representations. Works with any 256-bit hash including transaction hashes, block hashes, and message hashes.
    #[serde(rename = "detectHash")]
    DetectHash(DetectHashRequest),
    /// Converts a raw address to user-friendly base64 format. Raw addresses use the format `workchain:hex` (e.g., `0:abc...`). The packed format is shorter and includes a checksum for error detection.
    #[serde(rename = "packAddress")]
    PackAddress(PackAddressRequest),
    /// Converts a user-friendly base64 address to a raw address string in `workchain:hex` format.
    #[serde(rename = "unpackAddress")]
    UnpackAddress(UnpackAddressRequest),
    /// Returns the current state of any account on the TON blockchain. Includes the balance (in nanograms), smart contract code and data (if deployed), account status, and a reference to the last transaction. This is the primary endpoint for checking if an address exists and what's deployed there.
    #[serde(rename = "getAddressInformation")]
    GetAddressInformation(AddressInformationRequest),
    /// Returns detailed account information with parsed contract state. For recognized contract types, returns type-specific state fields. For other contracts, returns the raw state.
    #[serde(rename = "getExtendedAddressInformation")]
    GetExtendedAddressInformation(ExtendedAddressInformationRequest),
    /// Get raw TVM cell with shard account
    #[serde(rename = "getShardAccountCell")]
    GetShardAccountCell(ShardAccountCellRequest),
    /// Returns wallet-specific information for an address. If the address is a known wallet contract, returns the wallet type, current `seqno` (needed for sending transactions), and `wallet_id`. Always check `wallet: true` before using wallet-specific fields. Call this before sending any transaction to get the current `seqno`.
    #[serde(rename = "getWalletInformation")]
    GetWalletInformation(WalletInformationRequest),
    /// Returns the GRAM balance of an account in nanograms. 1 GRAM = 1,000,000,000 nanograms. A lightweight endpoint that returns only the balance without contract code, data, or other account details. Returns "0" for addresses that have never received any funds.
    #[serde(rename = "getAddressBalance")]
    GetAddressBalance(AddressBalanceRequest),
    /// Returns the account lifecycle state: `uninitialized` (no contract deployed), `active` (contract deployed), or `frozen` (contract state frozen).
    #[serde(rename = "getAddressState")]
    GetAddressState(AddressStateRequest),
    /// Returns metadata for Jetton or NFT contracts. Automatically detects the contract type and returns appropriate fields. For Jetton masters: total supply, admin, metadata. For Jetton wallets: balance, owner. For NFT items: collection, owner, content. For NFT collections: item count, metadata.
    #[serde(rename = "getTokenData")]
    GetTokenData(TokenDataRequest),
    /// Resolve TON DNS contract
    #[serde(rename = "dnsResolve")]
    DnsResolve(DnsResolveRequest),
    /// Returns the current state of the TON masterchain. The `last` field contains the latest block used for querying current state. The `seqno` in `last` is the current block height. Use this endpoint to obtain the latest block reference for other queries.
    #[serde(rename = "getMasterchainInfo")]
    GetMasterchainInfo(MasterchainInfoRequest),
    /// Returns validator signatures for a specific masterchain block. Each signature proves that a validator approved this block. Use this for building cryptographic proofs or verifying block authenticity in trustless applications.
    #[serde(rename = "getMasterchainBlockSignatures")]
    GetMasterchainBlockSignatures(MasterchainBlockSignaturesRequest),
    /// Returns a Merkle proof that links a shardchain block to a masterchain block. This proof cryptographically verifies that the shard block is part of the canonical chain. Used by light clients and cross-chain bridges to verify shard data without trusting the API.
    #[serde(rename = "getShardBlockProof")]
    GetShardBlockProof(ShardBlockProofRequest),
    /// Get block that was confirmed by consensus
    #[serde(rename = "getConsensusBlock")]
    GetConsensusBlock(ConsensusBlockRequest),
    /// Finds a block by position or time. Specify workchain and shard, then provide exactly one of `seqno` (block sequence number), `lt` (logical time), or `unixtime` (Unix timestamp). Returns the full block identifier including hashes.
    #[serde(rename = "lookupBlock")]
    LookupBlock(LookupBlockRequest),
    /// Returns the active shardchain block identifiers at a given masterchain block height. Each shard processes a subset of accounts in parallel. The response shows how the basechain is currently partitioned and which block each shard is at.
    #[serde(rename = "getShards")]
    GetShards(ShardsRequest),
    /// Get raw block data as a base64-encoded BOC
    #[serde(rename = "getBlock")]
    GetBlock(BlockDataRequest),
    /// Returns block metadata without the full transaction list. Includes timestamps, validator info, and references to previous blocks. Intended for block explorers and other use cases that require block information without transactions.
    #[serde(rename = "getBlockHeader")]
    GetBlockHeader(BlockHeaderRequest),
    /// Returns the current size of the outbound message queue for each shard. A growing queue indicates network congestion. If the queue is large, transactions may take longer to process. Monitor this to detect network issues.
    #[serde(rename = "getOutMsgQueueSize")]
    GetOutMsgQueueSize(OutMsgQueueSizeRequest),
    /// Returns a summary of transactions in a specific block. Each item contains the account address and transaction ID, but not full transaction details. Use `count` to limit results and `after_lt`/`after_hash` for pagination. Call getTransactions with each transaction ID to get full details.
    #[serde(rename = "getBlockTransactions")]
    GetBlockTransactions(BlockTransactionsRequest),
    /// Returns full transaction objects for transactions in a specific block. Each transaction includes complete data: inbound and outbound messages, fees, and BoC-encoded raw data. Use `count` to limit results and `after_lt`/`after_hash` for pagination when `incomplete` is true.
    #[serde(rename = "getBlockTransactionsExt")]
    GetBlockTransactionsExt(BlockTransactionsExtRequest),
    /// Returns transaction history for an account. Transactions are returned newest-first. Each transaction shows the incoming message that triggered it, all outgoing messages, and fees paid. For pagination: use the `lt` and `hash` from the oldest transaction as the starting point for the next request.
    #[serde(rename = "getTransactions")]
    GetTransactions(TransactionsRequest),
    /// Returns transaction history for an account in a standardized format. Transactions are returned newest-first. Each transaction includes the triggering inbound message, all outbound messages, and fees paid. The response includes a `previous_transaction_id` cursor for paginating through older transactions.
    #[serde(rename = "getTransactionsStd")]
    GetTransactionsStd(TransactionsRequest),
    /// Finds a transaction by message parameters. Given a source address, destination address, and message creation time (`created_lt`), returns the transaction that processed this message. Useful for locating when a previously sent message was executed.
    #[serde(rename = "tryLocateTx")]
    TryLocateTx(TryLocateTxRequest),
    /// Finds the transaction that received a specific message. Given message parameters, returns the transaction on the destination account that processed the incoming message. Use this to trace message delivery across accounts.
    #[serde(rename = "tryLocateResultTx")]
    TryLocateResultTx(TryLocateResultTxRequest),
    /// Finds the transaction that sent a specific message. Given message parameters, returns the transaction on the source account that created this outgoing message. Useful for tracing where a message originated from.
    #[serde(rename = "tryLocateSourceTx")]
    TryLocateSourceTx(TryLocateSourceTxRequest),
    /// Returns a specific blockchain configuration parameter. TON stores all network settings on-chain as numbered parameters. Common ones: 0 (config contract), 1 (elector), 15 (election timing), 17 (stake limits), 20-21 (gas prices), 34 (current validators). Check TON documentation for the full list.
    #[serde(rename = "getConfigParam")]
    GetConfigParam(ConfigParamRequest),
    /// Returns all blockchain configuration parameters at once. Includes gas prices, validator settings, workchain configs, and governance rules. Use the optional `seqno` to get historical configuration at a specific block height.
    #[serde(rename = "getConfigAll")]
    GetConfigAll(ConfigAllRequest),
    /// Returns smart contract library code by hash. Some contracts reference shared libraries instead of including all code directly. When a library reference appears in contract code, this endpoint fetches the actual library implementation.
    #[serde(rename = "getLibraries")]
    GetLibraries(LibrariesRequest),
    /// Executes a read-only method on a smart contract. Get methods query contract state without sending a transaction. Common methods include `seqno` (wallet sequence number), `get_wallet_data` (wallet info), and `get_jetton_data` (token info). Method arguments are provided in the `stack` array.
    #[serde(rename = "runGetMethod")]
    RunGetMethod(RunGetMethodRequest<S>),
    /// Executes a read-only method on a smart contract using typed stack entries. Input and output stack entries use explicit types (`TvmStackEntryNumber`, `TvmStackEntryCell`, etc.) for structured input/output handling. Common methods: `seqno` (wallet sequence number), `get_wallet_data` (wallet info), `get_jetton_data` (token info).
    #[serde(rename = "runGetMethodStd")]
    RunGetMethodStd(RunGetMethodStdRequest),
    /// Broadcasts a signed message to the TON network. The `boc` parameter must contain a complete, signed external message in base64 format. The API validates the message and forwards it to validators. Returns immediately after acceptance; use getTransactions to confirm the transaction was processed.
    #[serde(rename = "sendBoc")]
    SendBoc(SendBocRequest),
    /// Broadcasts a signed message to the TON network and returns the message hash. The `boc` parameter must contain a complete, signed external message in base64 format. The API validates the message and forwards it to validators. The returned hash can be used to track the message's processing status.
    #[serde(rename = "sendBocReturnHash")]
    SendBocReturnHash(SendBocRequest),
    /// Calculates the fees required to send a message. Provide the destination address and message body. For new contract deployments, also include `init_code` and `init_data`. Set `ignore_chksig` to true when estimating before signing. Returns a breakdown of storage, gas, and forwarding fees.
    #[serde(rename = "estimateFee")]
    EstimateFee(EstimateFeeRequest),
    /// C++ compatibility spelling of getShards, present in the server route and JSON-RPC allowlist.
    #[serde(rename = "shards")]
    ShardsAlias(ShardsRequest),
    /// C++ broadcast route returning message hashes. Despite its name, validation and `TONLib` failures still return errors; a successful response does not prove inclusion.
    #[serde(rename = "sendBocReturnHashNoError")]
    SendBocReturnHashNoError(SendBocRequest),
}
