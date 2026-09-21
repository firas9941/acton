//! Parameters for the supported TON Center v3 operations.
//!
//! For URL queries, omit null values and repeat array parameters once per element.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parameters for retrieving the oldest and newest indexed masterchain blocks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MasterchainInfoQuery {}

/// Selects the shard state referenced by a masterchain block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MasterchainBlockShardStateQuery {
    /// Sequence number of the masterchain block.
    pub seqno: i32,
}

/// Selects shard blocks associated with a masterchain block, with pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MasterchainBlockShardsQuery {
    /// Sequence number of the masterchain block.
    pub seqno: i32,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Selects an account for the v3 address-information endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddressInformationQuery {
    /// Account or contract address to query, in raw or user-friendly format.
    pub address: String,
    /// Whether to obtain account information through the v2-compatible backend.
    pub use_v2: Option<bool>,
}

/// Selects addresses for address-book or token-metadata lookup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddressesQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
}

/// Selects an account for the v3 wallet-information endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WalletInformationQuery {
    /// Account or contract address to query, in raw or user-friendly format.
    pub address: String,
    /// Whether to obtain account information through the v2-compatible backend.
    pub use_v2: Option<bool>,
}

/// Selects indexed account states and optionally their serialized code and data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStatesQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    pub address: Vec<String>,
    /// Whether to include serialized contract code and data cells.
    pub include_boc: Option<bool>,
}

/// Filters transaction traces by participating account, hashes, or execution range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TracesQuery {
    /// Account address whose traces should be returned.
    pub account: Option<String>,
    /// Identifier of the transaction trace associated with the record. In a URL query, repeat
    /// the parameter for each value.
    #[serde(default)]
    pub trace_id: Vec<String>,
    /// Transaction hash used to identify or select a trace node. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub tx_hash: Vec<String>,
    /// Message hashes used to select matching records. In a URL query, repeat the parameter
    /// for each value.
    #[serde(default)]
    pub msg_hash: Vec<String>,
    /// Masterchain block sequence number associated with the records.
    pub mc_seqno: Option<i32>,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: Option<i32>,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: Option<i32>,
    /// Lower bound of the logical-time range.
    pub start_lt: Option<u64>,
    /// Upper bound of the logical-time range.
    pub end_lt: Option<u64>,
    /// Whether to include parsed actions in returned traces.
    pub include_actions: Option<bool>,
    /// Action type names requested for parsing or filtering. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub supported_action_types: Vec<String>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters actions in traces that have not yet completed indexing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PendingActionsQuery {
    /// Account address associated with the operation.
    pub account: Option<String>,
    /// External-message hashes used to select pending traces. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub ext_msg_hash: Vec<String>,
    /// Action type names requested for parsing or filtering. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub supported_action_types: Vec<String>,
    /// Whether to include full transactions in returned actions.
    pub include_transactions: Option<bool>,
}

/// Filters pending traces by account or external-message hash.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PendingTracesQuery {
    /// Account address associated with the operation.
    pub account: Option<String>,
    /// External-message hashes used to select pending traces. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub ext_msg_hash: Vec<String>,
}

/// Filters indexed DNS records by wallet or domain, with pagination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecordsQuery {
    /// Wallet address used to select DNS records.
    pub wallet: Option<String>,
    /// DNS domain name associated with the record.
    pub domain: Option<String>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Filters indexed jetton burns by owner, wallet, master, or execution range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonBurnsQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
    /// Jetton wallet contract storing the owner's token balance. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub jetton_wallet: Vec<String>,
    /// Jetton master contract identifying the token.
    pub jetton_master: Option<String>,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: Option<i32>,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: Option<i32>,
    /// Lower bound of the logical-time range.
    pub start_lt: Option<u64>,
    /// Upper bound of the logical-time range.
    pub end_lt: Option<u64>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters indexed jetton transfers by owner, wallet, master, or direction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonTransfersQuery {
    /// Owner account address associated with the contract. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub owner_address: Vec<String>,
    /// Jetton wallet contract storing the owner's token balance. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub jetton_wallet: Vec<String>,
    /// Jetton master contract identifying the token.
    pub jetton_master: Option<String>,
    /// Message or transfer direction relative to the selected account, such as `in` or `out`.
    pub direction: Option<String>,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: Option<i32>,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: Option<i32>,
    /// Lower bound of the logical-time range.
    pub start_lt: Option<u64>,
    /// Upper bound of the logical-time range.
    pub end_lt: Option<u64>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters indexed NFT collections by contract or owner address.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftCollectionsQuery {
    /// NFT collection contract address. In a URL query, repeat the parameter for each value.
    #[serde(default)]
    pub collection_address: Vec<String>,
    /// Owner account address associated with the contract. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub owner_address: Vec<String>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Selects NFT sale contracts by address.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftSalesQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
}

/// Filters indexed NFT ownership transfers by item, collection, or owner.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftTransfersQuery {
    /// Owner account address associated with the contract. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub owner_address: Vec<String>,
    /// NFT item addresses used to select transfers. In a URL query, repeat the parameter for
    /// each value.
    #[serde(default)]
    pub item_address: Vec<String>,
    /// NFT collection contract address.
    pub collection_address: Option<String>,
    /// Message or transfer direction relative to the selected account, such as `in` or `out`.
    pub direction: Option<String>,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: Option<i32>,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: Option<i32>,
    /// Lower bound of the logical-time range.
    pub start_lt: Option<u64>,
    /// Upper bound of the logical-time range.
    pub end_lt: Option<u64>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters multisig orders and optionally requests decoded order actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MultisigOrdersQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
    /// Parent multisig contract address. In a URL query, repeat the parameter for each value.
    #[serde(default)]
    pub multisig_address: Vec<String>,
    /// Whether to decode the messages contained in multisig order actions.
    pub parse_actions: Option<bool>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters multisig contracts and optionally includes their orders.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MultisigWalletsQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
    /// Wallet addresses used to select related contracts. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub wallet_address: Vec<String>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
    /// Whether to include orders associated with each multisig contract.
    pub include_orders: Option<bool>,
}

/// Filters vesting contracts and optionally checks their whitelists.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VestingQuery {
    /// Contract addresses used to select vesting schedules. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub contract_address: Vec<String>,
    /// Wallet addresses used to select related contracts. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub wallet_address: Vec<String>,
    /// Whether to also match vesting contracts whose whitelist contains the requested wallet.
    pub check_whitelist: Option<bool>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Filters indexed transactions by account, block, hash, or execution range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionsQuery {
    /// Workchain identifier; `-1` identifies the masterchain and `0` the basechain.
    pub workchain: Option<i32>,
    /// Shard identifier as text, preserving the server representation.
    pub shard: Option<String>,
    /// Block sequence number within the selected shard.
    pub seqno: Option<i32>,
    /// Masterchain block sequence number associated with the records.
    pub mc_seqno: Option<i32>,
    /// Account address associated with the operation. In a URL query, repeat the parameter
    /// for each value.
    #[serde(default)]
    pub account: Vec<String>,
    /// Account addresses whose transactions should be excluded. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub exclude_account: Vec<String>,
    /// Transaction hash used to select matching records. In a URL query, repeat the parameter for
    /// each value.
    #[serde(default)]
    pub hash: Vec<String>,
    /// Transaction logical time, which orders transactions within an account.
    pub lt: Option<u64>,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: Option<i32>,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: Option<i32>,
    /// Lower bound of the logical-time range.
    pub start_lt: Option<u64>,
    /// Upper bound of the logical-time range.
    pub end_lt: Option<u64>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters indexed blocks by identifiers, hashes, or execution range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlocksQuery {
    /// Workchain identifier; `-1` identifies the masterchain and `0` the basechain.
    pub workchain: Option<i32>,
    /// Shard identifier as text, preserving the server representation.
    pub shard: Option<String>,
    /// Block sequence number within the selected shard.
    pub seqno: Option<i32>,
    /// Hash of the block's root cell.
    pub root_hash: Option<String>,
    /// Hash of the serialized block file.
    pub file_hash: Option<String>,
    /// Masterchain block sequence number associated with the records.
    pub mc_seqno: Option<i32>,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: Option<i32>,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: Option<i32>,
    /// Lower bound of the logical-time range.
    pub start_lt: Option<u64>,
    /// Upper bound of the logical-time range.
    pub end_lt: Option<u64>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Finds transactions linked to messages matching the supplied filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionsByMessageQuery {
    /// Message hashes used to select matching records. In a URL query, repeat the parameter
    /// for each value.
    #[serde(default)]
    pub msg_hash: Vec<String>,
    /// Hash of the serialized message body cell.
    pub body_hash: Option<String>,
    /// Operation code extracted from the message body.
    pub opcode: Option<String>,
    /// Message or transfer direction relative to the selected account, such as `in` or `out`.
    pub direction: Option<String>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Selects transactions associated with a masterchain block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionsByMasterchainBlockQuery {
    /// Sequence number of the masterchain block.
    pub seqno: i32,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters indexed messages by hashes, participants, opcode, or execution range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MessagesQuery {
    /// Message hashes used to select matching records. In a URL query, repeat the parameter
    /// for each value.
    #[serde(default)]
    pub msg_hash: Vec<String>,
    /// Hash of the serialized message body cell.
    pub body_hash: Option<String>,
    /// Source account address of the message or transfer.
    pub source: Option<String>,
    /// Destination account address of the message or transfer.
    pub destination: Option<String>,
    /// Operation code extracted from the message body.
    pub opcode: Option<String>,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: Option<i32>,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: Option<i32>,
    /// Lower bound of the logical-time range.
    pub start_lt: Option<u64>,
    /// Upper bound of the logical-time range.
    pub end_lt: Option<u64>,
    /// Message or transfer direction relative to the selected account, such as `in` or `out`.
    pub direction: Option<String>,
    /// Whether to exclude external messages from the results.
    pub exclude_externals: Option<bool>,
    /// Whether to return only external messages.
    pub only_externals: Option<bool>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Finds transactions linked to a transaction through incoming or outgoing messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdjacentTransactionsQuery {
    /// Transaction hash used to select matching records.
    pub hash: String,
    /// Message or transfer direction relative to the selected account, such as `in` or `out`.
    pub direction: Option<String>,
}

/// Selects indexed wallet states for the supplied account addresses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WalletStatesQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
}

/// Paginates accounts ranked by their native-coin balance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TopAccountsByBalanceQuery {
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Supplies a message body and optional deployment data for fee estimation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EstimateFeeRequest {
    /// Account or contract address to query, in raw or user-friendly format.
    pub address: String,
    /// Base64-encoded `BoC` containing the message body cell.
    pub body: String,
    /// Base64-encoded contract code cell supplied for deployment fee estimation.
    pub init_code: Option<String>,
    /// Base64-encoded contract data cell supplied for deployment fee estimation.
    pub init_data: Option<String>,
    /// Whether to bypass signature checking during fee estimation.
    pub ignore_chksig: Option<bool>,
}

/// Selects pending transactions by account or trace identifier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PendingTransactionsQuery {
    /// Account addresses whose pending transactions are requested. Supply at least one address.
    #[serde(default)]
    pub account: Vec<String>,
    /// Identifier of the transaction trace associated with the record. In a URL query, repeat
    /// the parameter for each value.
    #[serde(default)]
    pub trace_id: Vec<String>,
}

/// Filters indexed jetton master contracts by contract or administrator address.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonMastersQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
    /// Administrator address authorized to manage the jetton master. In a URL query, repeat
    /// the parameter for each value.
    #[serde(default)]
    pub admin_address: Vec<String>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Filters indexed jetton wallets by wallet, owner, master, or balance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonWalletsQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
    /// Owner account address associated with the contract. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub owner_address: Vec<String>,
    /// Jetton master address used to select token wallets. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub jetton_address: Vec<String>,
    /// Whether to exclude jetton wallets with zero token balance.
    pub exclude_zero_balance: Option<bool>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
    /// Ordering direction, such as `asc` or `desc`, accepted by the endpoint.
    pub sort: Option<String>,
}

/// Filters indexed NFT items by contract, owner, collection, or item index.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftItemsQuery {
    /// Account or contract address to query, in raw or user-friendly format. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub address: Vec<String>,
    /// Owner account address associated with the contract. In a URL query, repeat the
    /// parameter for each value.
    #[serde(default)]
    pub owner_address: Vec<String>,
    /// NFT collection contract address. In a URL query, repeat the parameter for each value.
    #[serde(default)]
    pub collection_address: Vec<String>,
    /// NFT item index within its collection, encoded as decimal text. In a URL query, repeat
    /// the parameter for each value.
    #[serde(default)]
    pub index: Vec<String>,
    /// Whether to include items held by sale or auction contracts when filtering by owner.
    pub include_on_sale: Option<bool>,
    /// Whether to order NFT items by their last transaction's logical time.
    pub sort_by_last_transaction_lt: Option<bool>,
    /// Maximum number of records to return in this page.
    pub limit: Option<i32>,
    /// Number of matching records to skip before this page.
    pub offset: Option<i32>,
}

/// Submits a serialized external message for broadcast to the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendMessageRequest {
    /// Base64-encoded `BoC` containing the external message to submit.
    pub boc: String,
}

/// Selects a contract get method and supplies its ordered input stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RunGetMethodRequest {
    /// Account or contract address to query, in raw or user-friendly format.
    pub address: String,
    /// Name of the contract get method to execute.
    pub method: String,
    /// Ordered input stack entries, encoded as a JSON array.
    pub stack: Vec<StackEntry>,
}

/// Input stack value; the type label determines the interpretation of its JSON payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StackEntry {
    /// TVM input value type, such as `num`, `cell`, or `slice`.
    #[serde(rename = "type")]
    pub kind: String,
    /// JSON payload interpreted according to the stack entry type.
    pub value: Value,
}
