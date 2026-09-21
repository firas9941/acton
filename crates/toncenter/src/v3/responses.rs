//! Indexed blockchain data and execution results returned by TON Center v3.

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use super::StringOrNumber;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// Address-keyed presentation and interface information for accounts in a response.
pub type AddressBook = HashMap<String, AddressBookRow>;
/// Address-keyed indexing status and token metadata for accounts in a response.
pub type Metadata = HashMap<String, AddressMetadata>;

/// Address presentation and contract-interface information supplied by the indexer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddressBookRow {
    /// DNS domain name associated with the record.
    #[serde(default)]
    pub domain: Option<String>,
    /// Contract interfaces detected by the indexer.
    #[serde(default)]
    pub interfaces: Option<Vec<String>>,
    /// User-friendly address representation supplied by the indexer.
    #[serde(default)]
    pub user_friendly: Option<String>,
}

/// Indexing status and token metadata associated with an address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddressMetadata {
    /// Whether the indexer has processed the address.
    pub is_indexed: bool,
    /// Token and NFT metadata records associated with the address.
    #[serde(default)]
    pub token_info: Vec<TokenInfo>,
}

/// Display metadata and validity flags for a token or NFT.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TokenInfo {
    /// Whether the indexer considers the token metadata valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
    /// Asset category, such as a jetton, NFT item, or NFT collection.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Display name supplied in token metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Ticker symbol supplied in token metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Human-readable description supplied in token metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Image URI supplied in token metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// NFT item index within its collection, encoded as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nft_index: Option<String>,
    /// Whether the metadata is marked as unsuitable for general audiences.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_nsfw: Option<bool>,
    /// Whether the token or NFT is flagged as a scam.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_scam: Option<bool>,
    /// Additional metadata attributes keyed by their original names.
    #[serde(default)]
    pub extra: HashMap<String, Value>,
}

/// Indexed account states with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStatesResponse {
    /// Indexed states of the requested accounts.
    pub accounts: Vec<AccountStateFull>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed traces with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TracesResponse {
    /// Matching transaction traces. Missing or null values decode as an empty collection.
    #[serde(default, deserialize_with = "deserialize_null_default")]
    #[cfg_attr(feature = "openapi", schema(nullable))]
    pub traces: Vec<Trace>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Parsed blockchain actions with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionsResponse {
    /// Parsed actions associated with the containing object.
    pub actions: Vec<Action>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed DNS records attached to a domain NFT.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecord {
    /// Address of the NFT item representing the domain.
    pub nft_item_address: String,
    /// Owner address of the domain NFT.
    pub nft_item_owner: Option<String>,
    /// DNS domain name associated with the record.
    pub domain: String,
    /// Contract address used to resolve the remaining domain labels.
    pub dns_next_resolver: Option<String>,
    /// Wallet address stored in the domain's DNS records.
    pub dns_wallet: Option<String>,
    /// ADNL address stored for the domain's site.
    pub dns_site_adnl: Option<String>,
    /// TON Storage bag identifier stored for the domain.
    pub dns_storage_bag_id: Option<String>,
}

/// Matching DNS records and their address-book entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecordsResponse {
    /// Matching DNS records.
    pub records: Vec<DnsRecord>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
}

/// Indexed jetton transfer with its transaction and notification payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonTransfer {
    /// Application request identifier carried by the token message, encoded as decimal text.
    pub query_id: String,
    /// Source account address of the message or transfer.
    pub source: String,
    /// Destination account address of the message or transfer.
    pub destination: String,
    /// Jetton amount in the token's smallest units, encoded as decimal text.
    pub amount: String,
    /// Jetton wallet contract that sent the tokens.
    pub source_wallet: String,
    /// Jetton master contract identifying the token.
    pub jetton_master: String,
    /// Hash of the transaction that processed the token operation.
    pub transaction_hash: String,
    /// Logical time of the transaction that processed the token operation.
    pub transaction_lt: String,
    /// Creation time of the transaction, in Unix seconds.
    pub transaction_now: i64,
    /// Whether the transaction aborted during execution.
    pub transaction_aborted: bool,
    /// Address designated to receive excess funds from the operation.
    pub response_destination: Option<String>,
    /// Base64-encoded custom payload cell attached to the token operation.
    pub custom_payload: Option<String>,
    /// Decoded custom payload supplied by the indexer, when available.
    pub decoded_custom_payload: Option<Value>,
    /// Native-coin amount forwarded with the token notification, in nanograms as decimal
    /// text.
    pub forward_ton_amount: Option<String>,
    /// Base64-encoded payload forwarded to the recipient.
    pub forward_payload: Option<String>,
    /// Decoded forwarded payload supplied by the indexer, when available.
    pub decoded_forward_payload: Option<Value>,
    /// Identifier of the transaction trace associated with the record.
    pub trace_id: Option<String>,
}

/// Matching jetton transfers with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonTransfersResponse {
    /// Matching indexed jetton transfers.
    pub jetton_transfers: Vec<JettonTransfer>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed jetton burn with the owner, amount, and originating transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonBurn {
    /// Application request identifier carried by the token message, encoded as decimal text.
    pub query_id: String,
    /// Address of the owner of the token wallet.
    pub owner: String,
    /// Jetton wallet contract storing the owner's token balance.
    pub jetton_wallet: String,
    /// Jetton master contract identifying the token.
    pub jetton_master: String,
    /// Hash of the transaction that processed the token operation.
    pub transaction_hash: String,
    /// Logical time of the transaction that processed the token operation.
    pub transaction_lt: String,
    /// Creation time of the transaction, in Unix seconds.
    pub transaction_now: i64,
    /// Whether the transaction aborted during execution.
    pub transaction_aborted: bool,
    /// Jetton amount in the token's smallest units, encoded as decimal text.
    pub amount: String,
    /// Address designated to receive excess funds from the operation.
    pub response_destination: Option<String>,
    /// Base64-encoded custom payload cell attached to the token operation.
    pub custom_payload: Option<String>,
    /// Decoded custom payload supplied by the indexer, when available.
    pub decoded_custom_payload: Option<Value>,
    /// Identifier of the transaction trace associated with the record.
    pub trace_id: Option<String>,
}

/// Matching jetton burns with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonBurnsResponse {
    /// Matching indexed jetton burns.
    pub jetton_burns: Vec<JettonBurn>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed NFT collection state and its content metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftCollection {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Owner account address associated with the contract.
    pub owner_address: Option<String>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    pub last_transaction_lt: String,
    /// Next NFT item index available to the collection, encoded as decimal text.
    pub next_item_index: String,
    /// Collection metadata keyed by content attribute name.
    pub collection_content: HashMap<String, Value>,
    /// Hash of the persistent contract data cell.
    pub data_hash: String,
    /// Hash of the contract code cell.
    pub code_hash: String,
}

/// Matching NFT collections with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftCollectionsResponse {
    /// Matching indexed NFT collections.
    pub nft_collections: Vec<NftCollection>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed NFT ownership transfer with its transaction and notification payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftTransfer {
    /// Application request identifier carried by the token message, encoded as decimal text.
    pub query_id: String,
    /// Address of the NFT item involved in the operation.
    pub nft_address: String,
    /// Collection address associated with the transferred NFT.
    pub nft_collection: String,
    /// Hash of the transaction that processed the token operation.
    pub transaction_hash: String,
    /// Logical time of the transaction that processed the token operation.
    pub transaction_lt: String,
    /// Creation time of the transaction, in Unix seconds.
    pub transaction_now: i64,
    /// Whether the transaction aborted during execution.
    pub transaction_aborted: bool,
    /// NFT owner address before the transfer.
    pub old_owner: String,
    /// NFT owner address after the transfer.
    pub new_owner: String,
    /// Address designated to receive excess funds from the operation.
    pub response_destination: Option<String>,
    /// Base64-encoded custom payload cell attached to the token operation.
    pub custom_payload: Option<String>,
    /// Decoded custom payload supplied by the indexer, when available.
    pub decoded_custom_payload: Option<Value>,
    /// Native-coin amount forwarded with the NFT notification, in nanograms as decimal text.
    pub forward_amount: Option<String>,
    /// Base64-encoded payload forwarded to the recipient.
    pub forward_payload: Option<String>,
    /// Decoded forwarded payload supplied by the indexer, when available.
    pub decoded_forward_payload: Option<Value>,
    /// Identifier of the transaction trace associated with the record.
    pub trace_id: Option<String>,
}

/// Matching NFT transfers with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftTransfersResponse {
    /// Matching indexed NFT ownership transfers.
    pub nft_transfers: Vec<NftTransfer>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed NFT sale or auction contract, with sale-specific details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftSale {
    /// Sale or auction contract type; determines the structure of `details`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Address of the NFT item involved in the operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nft_address: Option<String>,
    /// Owner address recorded by the NFT sale contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nft_owner_address: Option<String>,
    /// Marketplace contract associated with the sale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketplace_address: Option<String>,
    /// Creation time of the object, in Unix seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_transaction_lt: Option<String>,
    /// Hash of the contract code cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_hash: Option<String>,
    /// Hash of the persistent contract data cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_hash: Option<String>,
    /// Type-specific parsed data supplied by the indexer.
    pub details: Value,
    /// NFT item state associated with the sale contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nft_item: Option<NftItem>,
}

/// Matching sale contracts with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftSalesResponse {
    /// Matching indexed NFT sale or auction contracts.
    pub nft_sales: Vec<NftSale>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// An outgoing action in a multisig order, with optional decoded message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MultisigOrderAction {
    /// Destination account address of the message or transfer.
    pub destination: Option<String>,
    /// Native-coin value in nanograms, encoded as decimal text.
    pub value: Option<String>,
    /// Serialized or raw message body supplied for the order action.
    pub body_raw: Value,
    /// Whether the indexer successfully decoded the action's message body.
    pub parsed: bool,
    /// Diagnostic message describing the failed request or parsing operation.
    pub error: Option<String>,
    /// Decoded message body, when parsing succeeded.
    pub parsed_body: Option<Value>,
    /// Type label identifying the decoded message body.
    pub parsed_body_type: String,
    /// TVM message send-mode bitmask used by the order action.
    pub send_mode: u8,
}

/// Multisig order state, approvals, expiration, and requested actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MultisigOrder {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Parent multisig contract address.
    pub multisig_address: String,
    /// Order sequence number within the multisig, encoded as decimal text.
    pub order_seqno: Option<String>,
    /// Number of signer approvals required to execute an order.
    pub threshold: Option<i32>,
    /// Whether the approved order has been submitted for execution.
    pub sent_for_execution: Option<bool>,
    /// Bitmask of collected signer approvals, encoded as decimal text.
    pub approvals_mask: Option<String>,
    /// Number of approvals collected for the order.
    pub approvals_num: Option<i32>,
    /// Order expiration time, in Unix seconds.
    pub expiration_date: Option<u64>,
    /// Base64-encoded `BoC` containing the order's actions.
    pub order_boc: Option<String>,
    /// Signer addresses in the order used by the approval bitmask.
    #[serde(default)]
    pub signers: Vec<String>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    pub last_transaction_lt: String,
    /// Hash of the contract code cell.
    pub code_hash: Option<String>,
    /// Hash of the persistent contract data cell.
    pub data_hash: Option<String>,
    /// Parsed actions associated with the containing object. Missing or null values decode as
    /// an empty collection.
    #[serde(default, deserialize_with = "deserialize_null_default")]
    #[cfg_attr(feature = "openapi", schema(nullable))]
    pub actions: Vec<MultisigOrderAction>,
}

/// Indexed multisig contract state, authorized participants, and optional orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Multisig {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Next sequence number available for a multisig order, encoded as decimal text.
    pub next_order_seqno: Option<String>,
    /// Number of signer approvals required to execute an order.
    pub threshold: Option<i32>,
    /// Signer addresses in the order used by the approval bitmask.
    #[serde(default)]
    pub signers: Vec<String>,
    /// Addresses authorized to propose multisig orders.
    #[serde(default)]
    pub proposers: Vec<String>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    pub last_transaction_lt: String,
    /// Hash of the contract code cell.
    pub code_hash: Option<String>,
    /// Hash of the persistent contract data cell.
    pub data_hash: Option<String>,
    /// Orders associated with the selected multisig contracts.
    #[serde(default)]
    pub orders: Vec<MultisigOrder>,
}

/// Matching multisig orders and their address-book entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MultisigOrdersResponse {
    /// Orders associated with the selected multisig contracts.
    pub orders: Vec<MultisigOrder>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
}

/// Matching multisig contracts and their address-book entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MultisigsResponse {
    /// Matching indexed multisig contracts.
    pub multisigs: Vec<Multisig>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
}

/// Vesting schedule, participants, and transfer restrictions for a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VestingInfo {
    /// Contract or account address in the representation returned by the indexer.
    pub address: Option<String>,
    /// Time when the vesting schedule starts, in Unix seconds.
    pub start_time: Option<i64>,
    /// Total duration of the vesting schedule, in seconds.
    pub total_duration: Option<i64>,
    /// Interval between scheduled releases, in seconds.
    pub unlock_period: Option<i64>,
    /// Initial period before funds can be released, in seconds.
    pub cliff_duration: Option<i64>,
    /// Address funding the vesting schedule.
    pub sender_address: Option<String>,
    /// Owner account address associated with the contract.
    pub owner_address: Option<String>,
    /// Total native-coin amount in the vesting schedule, in nanograms as decimal text.
    pub total_amount: Option<String>,
    /// Addresses to which the vesting contract permits restricted transfers.
    #[serde(default)]
    pub whitelist: Vec<String>,
}

/// Matching vesting schedules and their address-book entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VestingContractsResponse {
    /// Matching indexed vesting schedules.
    pub vesting_contracts: Vec<VestingInfo>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
}

/// Indexed transactions and address-book entries for their participants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionsResponse {
    /// Transactions associated with or selected by the request.
    pub transactions: Vec<Transaction>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
}

/// Indexed messages with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MessagesResponse {
    /// Messages associated with or selected by the request.
    pub messages: Vec<Message>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed wallet states with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WalletStatesResponse {
    /// Wallet states selected by the requested addresses.
    pub wallets: Vec<WalletState>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Wallet detection result and version-specific state for an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WalletState {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Whether the account is recognized as a wallet contract.
    pub is_wallet: bool,
    /// Detected wallet implementation and version label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_type: Option<String>,
    /// Wallet sequence counter reported as a signed 64-bit integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<i64>,
    /// Wallet or subwallet identifier reported by the contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<i64>,
    /// Native-coin balance in nanograms, encoded as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    /// Extra-currency balances keyed by currency identifier, with decimal-string amounts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_currencies: Option<HashMap<String, String>>,
    /// Whether the wallet currently accepts signature-authorized requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_signature_allowed: Option<bool>,
    /// Account lifecycle status reported by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Hash of the contract code cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_hash: Option<String>,
    /// Hash of the account's latest indexed transaction.
    #[serde(default)]
    pub last_transaction_hash: Option<String>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    #[serde(default)]
    pub last_transaction_lt: Option<String>,
}

/// Account address and native-coin balance in a balance ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountBalance {
    /// Account address associated with the operation.
    pub account: String,
    /// Native-coin balance in nanograms, encoded as decimal text.
    pub balance: String,
}

/// Estimated fees charged to the sender and destination contracts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EstimateFeeResult {
    /// Estimated fees charged to the source contract.
    pub source_fees: EstimatedFee,
    /// Estimated fees charged to the destination contracts.
    pub destination_fees: Vec<EstimatedFee>,
}

/// Fee components in nanograms; 1 GRAM equals 1,000,000,000 nanograms.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EstimatedFee {
    /// Incoming-message forwarding fee in nanograms.
    pub in_fwd_fee: u64,
    /// Storage fee in nanograms.
    pub storage_fee: u64,
    /// TVM execution fee in nanograms.
    pub gas_fee: u64,
    /// Message forwarding fee in nanograms.
    pub fwd_fee: u64,
}

/// Block position identified by workchain, shard, and sequence number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockId {
    /// Workchain identifier; `-1` identifies the masterchain and `0` the basechain.
    pub workchain: i32,
    /// Shard identifier as text, preserving the server representation.
    pub shard: String,
    /// Block sequence number within the selected shard.
    pub seqno: u32,
}

/// Indexed block header, references, and transaction count.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Block {
    /// Workchain identifier; `-1` identifies the masterchain and `0` the basechain.
    pub workchain: i32,
    /// Shard identifier as text, preserving the server representation.
    pub shard: String,
    /// Block sequence number within the selected shard.
    pub seqno: u32,
    /// Hash of the block's root cell.
    pub root_hash: String,
    /// Hash of the serialized block file.
    pub file_hash: String,
    /// Lower bound of the logical-time range.
    pub start_lt: String,
    /// Upper bound of the logical-time range.
    pub end_lt: String,
    /// Block creation time in Unix seconds, retaining its string or integer representation.
    pub gen_utime: StringOrNumber,
    /// Masterchain block referenced by this block.
    pub masterchain_block_ref: BlockId,
    /// Immediate predecessor blocks; a merge can have multiple predecessors.
    #[serde(default)]
    pub prev_blocks: Vec<BlockId>,
    /// Whether the block immediately follows a shard merge.
    pub after_merge: bool,
    /// Whether the block immediately follows a shard split.
    pub after_split: bool,
    /// Whether a shard split is scheduled after this block.
    pub before_split: bool,
    /// Identifier of the validator that created the block.
    pub created_by: String,
    /// Block-header flags supplied by the protocol.
    pub flags: i32,
    /// Catchain sequence number used to generate the block.
    pub gen_catchain_seqno: i32,
    /// Global network identifier recorded in the block.
    pub global_id: i32,
    /// Whether this is a masterchain key block.
    pub key_block: bool,
    /// Sequence number of the referenced masterchain block.
    pub master_ref_seqno: i32,
    /// Minimum masterchain sequence number referenced by the block's state.
    pub min_ref_mc_seqno: i32,
    /// Sequence number of the preceding masterchain key block.
    pub prev_key_block_seqno: i32,
    /// Random seed recorded in the block header.
    pub rand_seed: String,
    /// Number of transactions included in the block.
    pub tx_count: i64,
    /// Short validator-list hash recorded in the block header.
    pub validator_list_hash_short: i32,
    /// Protocol version recorded in the block header.
    pub version: i64,
    /// Vertical block sequence number.
    pub vert_seqno: i32,
    /// Whether the block increments the vertical sequence number.
    pub vert_seqno_incr: bool,
    /// Whether the block signals a requested shard merge.
    pub want_merge: bool,
    /// Whether the block signals a requested shard split.
    pub want_split: bool,
}

/// Indexed blocks matching the requested filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlocksResponse {
    /// Blocks matching the requested filters.
    pub blocks: Vec<Block>,
}

/// Oldest and newest masterchain blocks available in the index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MasterchainInfo {
    /// Oldest masterchain block available in the index.
    pub first: Block,
    /// Newest masterchain block available in the index.
    pub last: Block,
}

/// Indexed jetton master state, supply, administration, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonMaster {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Administrator address authorized to manage the jetton master.
    #[serde(default)]
    pub admin_address: Option<String>,
    /// Hash of the contract code cell.
    pub code_hash: String,
    /// Hash of the persistent contract data cell.
    pub data_hash: String,
    /// Jetton metadata keyed by content attribute name.
    #[serde(default)]
    pub jetton_content: HashMap<String, Value>,
    /// Hash of the wallet code used by this jetton master.
    pub jetton_wallet_code_hash: String,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    pub last_transaction_lt: String,
    /// Whether the jetton master allows additional tokens to be minted.
    pub mintable: bool,
    /// Total minted jetton amount in the token's smallest units, as decimal text.
    pub total_supply: String,
}

/// Matching jetton masters with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonMastersResponse {
    /// Matching indexed jetton master contracts.
    pub jetton_masters: Vec<JettonMaster>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Matching jetton wallets with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonWalletsResponse {
    /// Matching indexed jetton wallet contracts.
    pub jetton_wallets: Vec<JettonWallet>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Indexed NFT ownership, collection membership, metadata, and sale status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftItem {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Auction contract associated with the NFT item.
    #[serde(default)]
    pub auction_contract_address: Option<String>,
    /// Hash of the contract code cell.
    pub code_hash: String,
    /// Collection reference associated with the NFT item.
    #[serde(default)]
    pub collection: Option<NftCollectionRef>,
    /// NFT collection contract address.
    #[serde(default)]
    pub collection_address: Option<String>,
    /// NFT metadata keyed by content attribute name.
    #[serde(default)]
    pub content: HashMap<String, Value>,
    /// Hash of the persistent contract data cell.
    pub data_hash: String,
    /// NFT item index within its collection, encoded as decimal text.
    pub index: String,
    /// Whether the NFT item has been initialized.
    pub init: bool,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    pub last_transaction_lt: String,
    /// Whether the indexer identifies an active sale for the NFT item.
    pub on_sale: bool,
    /// Owner account address associated with the contract.
    #[serde(default)]
    pub owner_address: Option<String>,
    /// Beneficial owner identified behind an NFT sale or custody contract.
    #[serde(default)]
    pub real_owner: Option<String>,
    /// Sale contract associated with the NFT item.
    #[serde(default)]
    pub sale_contract_address: Option<String>,
}

/// Address identifying the collection associated with an NFT item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftCollectionRef {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
}

/// Matching NFT items with address-book entries and token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftItemsResponse {
    /// Matching indexed NFT items.
    pub nft_items: Vec<NftItem>,
    /// Address-keyed presentation and contract-interface information.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressBookRow>))]
    pub address_book: AddressBook,
    /// Address-keyed token and NFT metadata.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = HashMap<String, AddressMetadata>))]
    pub metadata: Metadata,
}

/// Hashes identifying the submitted external message; acceptance does not confirm execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendMessageResult {
    /// Hash of the submitted external message.
    pub message_hash: String,
    /// Normalized external-message hash used to track the submitted message.
    pub message_hash_norm: String,
}

/// Get-method execution outcome, gas usage, and ordered result stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RunGetMethodResult {
    /// TVM gas units consumed during execution.
    pub gas_used: StringOrNumber,
    /// TVM exit code; `0` and `1` indicate successful execution.
    pub exit_code: i32,
    /// Ordered TVM stack entries supplied to or returned by the get method.
    pub stack: Vec<StackEntity>,
    /// VM execution trace when supplied by the server.
    #[serde(default)]
    pub vm_log: Option<String>,
}

/// Output stack value with a type label and a scalar or nested payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StackEntity {
    /// TVM value type, such as `num`, `cell`, `slice`, `tuple`, or `list`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Scalar payload or nested stack entries, interpreted according to `type`.
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub value: StackValue,
}

/// Nested stack entries or a JSON payload interpreted according to the entry type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StackValue {
    /// Ordered entries of a tuple or list; matching arrays decode as this variant first.
    Entries(Vec<StackEntity>),
    /// Scalar, null, or other JSON value not decoded as a list of stack entries.
    Json(Value),
}

#[cfg(feature = "openapi")]
impl utoipa::PartialSchema for StackValue {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::AnyOfBuilder::new()
            .description(Some("Nested stack entries or any JSON payload. Matching arrays decode as stack entries first."))
            .item(Vec::<StackEntity>::schema())
            .item(Value::schema())
            .into()
    }
}

#[cfg(feature = "openapi")]
impl utoipa::ToSchema for StackValue {
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.push((
            StackEntity::name().into_owned(),
            <StackEntity as utoipa::PartialSchema>::schema(),
        ));
        StackEntity::schemas(schemas);
    }
}

/// Account state returned by the v3 address-information compatibility endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct V2AddressInformation {
    /// Native-coin balance in nanograms, encoded as decimal text.
    pub balance: String,
    /// Base64-encoded contract code cell.
    #[serde(default)]
    pub code: Option<String>,
    /// Base64-encoded persistent contract data cell.
    #[serde(default)]
    pub data: Option<String>,
    /// Hash identifying the state retained for a frozen account.
    #[serde(default)]
    pub frozen_hash: Option<String>,
    /// Hash of the account's latest indexed transaction.
    #[serde(default)]
    pub last_transaction_hash: Option<String>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    #[serde(default)]
    pub last_transaction_lt: Option<String>,
    /// Account lifecycle status reported by the server.
    pub status: String,
}

/// Wallet state returned by the v3 wallet-information compatibility endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct V2WalletInformation {
    /// Native-coin balance in nanograms, encoded as decimal text.
    pub balance: String,
    /// Detected wallet implementation and version label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_type: Option<String>,
    /// Wallet sequence counter reported as a signed 64-bit integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<i64>,
    /// Wallet or subwallet identifier reported by the contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<i64>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    pub last_transaction_lt: String,
    /// Hash of the account's latest indexed transaction.
    pub last_transaction_hash: String,
    /// Account lifecycle status reported by the server.
    pub status: String,
}

/// API or request-validation error returned instead of a successful result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RequestError {
    /// Present for API errors, but omitted by request-body validation errors (HTTP 422).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    /// Diagnostic message describing the failed request or parsing operation.
    pub error: String,
}

/// Transaction-tree node linking an incoming message to its descendant transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TraceNode {
    /// Child transactions reached through this node's outgoing messages.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub children: Vec<TraceNode>,
    /// Incoming message that triggered the transaction.
    #[serde(default)]
    pub in_msg: Option<Message>,
    /// Hash of the incoming message associated with the node.
    #[serde(default)]
    pub in_msg_hash: Option<String>,
    /// Full transaction associated with this trace node, when included.
    #[serde(default)]
    pub transaction: Option<Transaction>,
    /// Transaction hash used to identify or select a trace node.
    #[serde(default)]
    pub tx_hash: Option<String>,
}

/// Parsed operation spanning one or more transactions within a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Action {
    /// Accounts participating in the operation.
    #[serde(default)]
    pub accounts: Vec<String>,
    /// Identifier of the parsed action within its trace.
    pub action_id: String,
    /// Type-specific parsed data supplied by the indexer.
    pub details: Value,
    /// Upper bound of the logical-time range.
    pub end_lt: String,
    /// Latest execution time in the range, in Unix seconds.
    pub end_utime: u32,
    /// Finality label assigned to the transaction or action by the indexer.
    pub finality: String,
    /// Identifier of the parent gasless action, when applicable.
    #[serde(default)]
    pub parent_gasless_action: Option<String>,
    /// Lower bound of the logical-time range.
    pub start_lt: String,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: u32,
    /// Whether the operation or execution phase completed successfully.
    #[serde(default)]
    pub success: Option<bool>,
    /// Logical time at which the containing trace ends.
    pub trace_end_lt: String,
    /// End time of the containing trace, in Unix seconds.
    pub trace_end_utime: u32,
    /// Hash of the external message that initiated the trace.
    #[serde(default)]
    pub trace_external_hash: Option<String>,
    /// Normalized hash of the external message that initiated the trace.
    #[serde(default)]
    pub trace_external_hash_norm: Option<String>,
    /// Identifier of the transaction trace associated with the record.
    #[serde(default)]
    pub trace_id: Option<String>,
    /// Masterchain sequence number at the end of the containing trace.
    pub trace_mc_seqno_end: u32,
    /// Hashes of the transactions participating in the parsed action.
    #[serde(default)]
    pub transactions: Vec<String>,
    /// Full transaction records included for the parsed action.
    #[serde(default)]
    pub transactions_full: Vec<Transaction>,
    /// Parsed operation type; determines the structure of `details`.
    #[serde(rename = "type")]
    pub kind: String,
}

/// Indexed token balance and ownership of a jetton wallet contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonWallet {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Jetton balance in the token's smallest units, encoded as decimal text.
    pub balance: String,
    /// Hash of the contract code cell.
    #[serde(default)]
    pub code_hash: Option<String>,
    /// Hash of the persistent contract data cell.
    #[serde(default)]
    pub data_hash: Option<String>,
    /// Jetton master contract identifying the wallet's token.
    pub jetton: String,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    pub last_transaction_lt: String,
    /// Mintless-jetton claim information supplied by the indexer.
    #[serde(default)]
    pub mintless_info: Option<Value>,
    /// Address of the owner of the token wallet.
    pub owner: String,
}

/// Indexed account state, contract data, and detected interfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateFull {
    /// Contract or account address in the representation returned by the indexer.
    pub address: String,
    /// Hash of the serialized account state.
    pub account_state_hash: String,
    /// Native-coin balance in nanograms, encoded as decimal text.
    #[serde(default)]
    pub balance: Option<String>,
    /// Base64-encoded `BoC` containing the contract code cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_boc: Option<String>,
    /// Hash of the contract code cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_hash: Option<String>,
    /// Get-method identifiers detected in the contract code. Missing or null values decode as
    /// an empty collection.
    #[serde(default, deserialize_with = "deserialize_null_default")]
    #[cfg_attr(feature = "openapi", schema(nullable))]
    pub contract_methods: Vec<i32>,
    /// Base64-encoded `BoC` containing the persistent contract data cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_boc: Option<String>,
    /// Hash of the persistent contract data cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_hash: Option<String>,
    /// Extra-currency balances keyed by currency identifier, with decimal-string amounts.
    #[serde(default)]
    pub extra_currencies: HashMap<String, String>,
    /// Hash identifying the state retained for a frozen account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_hash: Option<String>,
    /// Contract interfaces detected by the indexer.
    #[serde(default)]
    pub interfaces: Option<Vec<String>>,
    /// Hash of the account's latest indexed transaction.
    #[serde(default)]
    pub last_transaction_hash: Option<String>,
    /// Logical time of the account's latest indexed transaction, encoded as decimal text.
    #[serde(default)]
    pub last_transaction_lt: Option<String>,
    /// Account lifecycle status reported by the server.
    pub status: String,
}

/// Transaction returned by `/transactions`, `/transactionsByMessage`, and traces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Transaction {
    /// Account address associated with the operation.
    pub account: String,
    /// Hash of the serialized transaction cell.
    pub hash: String,
    /// Transaction logical time, which orders transactions within an account.
    pub lt: String,
    /// Block containing the transaction.
    pub block_ref: BlockId,
    /// Transaction creation time, in Unix seconds.
    #[serde(default)]
    pub now: u32,
    /// Masterchain sequence number associated with the transaction.
    pub mc_block_seqno: u32,
    /// Whether the transaction was produced by emulation rather than an indexed block.
    pub emulated: bool,
    /// Finality label assigned to the transaction or action by the indexer.
    pub finality: String,
    /// Hash of the preceding transaction on this account.
    pub prev_trans_hash: String,
    /// Logical time of the preceding transaction on this account.
    pub prev_trans_lt: String,
    /// Account lifecycle status before the transaction.
    pub orig_status: String,
    /// Account lifecycle status after the transaction.
    pub end_status: String,
    /// Total native-coin fees charged by the transaction, in nanograms as decimal text.
    pub total_fees: String,
    /// Transaction fees in extra currencies, keyed by currency identifier.
    #[serde(default)]
    pub total_fees_extra_currencies: HashMap<String, String>,
    /// Hash of the external message that initiated the trace.
    #[serde(default)]
    pub trace_external_hash: Option<String>,
    /// Identifier of the transaction trace associated with the record.
    #[serde(default)]
    pub trace_id: Option<String>,
    /// Hashes of transactions caused by this transaction's outgoing messages.
    #[serde(default)]
    pub child_transactions: Vec<String>,
    /// Transaction kind and its execution phases.
    pub description: TransactionDescr,
    /// Incoming message that triggered the transaction.
    #[serde(default)]
    pub in_msg: Option<Message>,
    /// Messages emitted by the transaction.
    #[serde(default)]
    pub out_msgs: Vec<Message>,
    /// Account state immediately before execution.
    pub account_state_before: AccountState,
    /// Account state immediately after execution.
    pub account_state_after: AccountState,
}

/// Account state referenced by a transaction before or after execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountState {
    /// Hash of the serialized account state.
    pub hash: String,
    /// Lifecycle status of the account represented by this state.
    #[serde(default)]
    pub account_status: Option<String>,
    /// Native-coin balance in nanograms, encoded as decimal text.
    #[serde(default)]
    pub balance: Option<String>,
    /// Base64-encoded `BoC` containing the contract code cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_boc: Option<String>,
    /// Hash of the contract code cell.
    #[serde(default)]
    pub code_hash: Option<String>,
    /// Base64-encoded `BoC` containing the persistent contract data cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_boc: Option<String>,
    /// Hash of the persistent contract data cell.
    #[serde(default)]
    pub data_hash: Option<String>,
    /// Extra-currency balances keyed by currency identifier, with decimal-string amounts.
    #[serde(default)]
    pub extra_currencies: Option<HashMap<String, String>>,
    /// Hash identifying the state retained for a frozen account.
    #[serde(default)]
    pub frozen_hash: Option<String>,
}

/// Transaction kind and the execution phases applicable to that kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionDescr {
    /// Transaction kind, such as ordinary, tick-tock, split, or merge.
    #[serde(rename = "type")]
    pub kind: String,
    /// Whether the transaction aborted.
    #[serde(default)]
    pub aborted: Option<bool>,
    /// Whether the account was destroyed during execution.
    #[serde(default)]
    pub destroyed: Option<bool>,
    /// Whether crediting the incoming value preceded storage-fee collection.
    #[serde(default)]
    pub credit_first: Option<bool>,
    /// TVM execution or skipped-computation phase, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute_ph: Option<ComputePhase>,
    /// Outcome of the action phase, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionPhase>,
    /// Storage-fee collection phase, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_ph: Option<StoragePhase>,
    /// Incoming-value credit phase, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit_ph: Option<CreditPhase>,
    /// Bounce-phase details supplied for this transaction kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounce: Option<Value>,
    /// Whether the transaction installed the state used by a shard split or merge.
    #[serde(default)]
    pub installed: Option<bool>,
    /// Whether the transaction is a tock rather than a tick invocation.
    #[serde(default)]
    pub is_tock: Option<bool>,
    /// Shard split or merge information supplied for the transaction kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split_info: Option<Value>,
}

/// Value credited to the account and any previously owed fees collected from it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreditPhase {
    /// Previously owed fees collected during the credit phase, in nanograms as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_fees_collected: Option<String>,
    /// Native-coin amount credited to the account, in nanograms as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit: Option<String>,
    /// Extra-currency amounts credited, keyed by currency identifier.
    #[serde(default)]
    pub credit_extra_currencies: HashMap<String, String>,
}

/// TVM execution or skipped-computation details for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ComputePhase {
    /// Whether TVM computation was skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped: Option<bool>,
    /// Whether the operation or execution phase completed successfully.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    /// Whether the incoming message's state was used during computation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg_state_used: Option<bool>,
    /// Whether computation activated the account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_activated: Option<bool>,
    /// Fee charged for TVM computation, in nanograms as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_fees: Option<String>,
    /// TVM gas units consumed during execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<String>,
    /// Gas-unit limit applied to contract execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<String>,
    /// Initial gas credit available before message acceptance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_credit: Option<String>,
    /// Execution mode recorded in the compute phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<i8>,
    /// TVM exit code; `0` and `1` indicate successful execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Additional argument accompanying the TVM exit code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_arg: Option<i32>,
    /// Number of TVM instructions executed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vm_steps: Option<u32>,
    /// Hash of the VM state at the beginning of computation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vm_init_state_hash: Option<String>,
    /// Hash of the VM state at the end of computation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vm_final_state_hash: Option<String>,
    /// Protocol reason explaining why computation was skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Outcome of applying the actions produced by contract execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionPhase {
    /// Whether the operation or execution phase completed successfully.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    /// Whether the computed action list was structurally valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
    /// Whether insufficient funds prevented action execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_funds: Option<bool>,
    /// Account status transition produced by the phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_change: Option<String>,
    /// Action-phase result code; zero indicates success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_code: Option<i32>,
    /// Additional argument accompanying the action-phase result code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_arg: Option<i32>,
    /// Total number of actions in the computed action list; also accepts `total_actions` on
    /// input.
    #[serde(
        default,
        alias = "total_actions",
        skip_serializing_if = "Option::is_none"
    )]
    pub tot_actions: Option<u32>,
    /// Number of special actions in the computed action list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_actions: Option<u32>,
    /// Number of actions skipped during execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped_actions: Option<u32>,
    /// Number of outgoing messages created by the action phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msgs_created: Option<u32>,
    /// Total forwarding fees for generated messages, in nanograms as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_fwd_fees: Option<String>,
    /// Total fees charged for action processing, in nanograms as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_action_fees: Option<String>,
    /// Hash of the computed action list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_list_hash: Option<String>,
    /// Combined serialized size of the generated messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tot_msg_size: Option<MsgSize>,
}

/// Total cell and bit counts of serialized outgoing messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MsgSize {
    /// Number of cells in the serialized messages, encoded as decimal text.
    #[serde(default)]
    pub cells: Option<String>,
    /// Number of bits in the serialized messages, encoded as decimal text.
    #[serde(default)]
    pub bits: Option<String>,
}

/// Storage fees charged to an account and the resulting status change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StoragePhase {
    /// Storage fees collected in this phase, in nanograms as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_fees_collected: Option<String>,
    /// Unpaid storage fees remaining after this phase, in nanograms as decimal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_fees_due: Option<String>,
    /// Account status transition produced by the phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_change: Option<String>,
}

/// Indexed message with participants, transferred value, fees, and serialized content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Message {
    /// Hash of the serialized message cell.
    pub hash: String,
    /// Normalized message hash used to identify equivalent external messages.
    #[serde(default)]
    pub hash_norm: Option<String>,
    /// Source account address of the message or transfer.
    #[serde(default)]
    pub source: Option<String>,
    /// Destination account address of the message or transfer.
    #[serde(default)]
    pub destination: Option<String>,
    /// Native-coin value in nanograms, encoded as decimal text.
    #[serde(default)]
    pub value: Option<String>,
    /// Extra-currency values carried by the message, keyed by currency identifier.
    #[serde(default)]
    pub value_extra_currencies: Option<HashMap<String, String>>,
    /// Message forwarding fee in nanograms.
    #[serde(default)]
    pub fwd_fee: Option<String>,
    /// Instant hypercube routing fee in nanograms, encoded as decimal text.
    #[serde(default)]
    pub ihr_fee: Option<String>,
    /// Message creation logical time, encoded as decimal text.
    #[serde(default)]
    pub created_lt: Option<String>,
    /// Creation time of the object, in Unix seconds.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Human-readable operation label decoded from the message body.
    #[serde(default)]
    pub decoded_opcode: Option<String>,
    /// Additional protocol flags carried by the message, encoded as text.
    #[serde(default)]
    pub extra_flags: Option<String>,
    /// Whether instant hypercube routing is disabled for the message.
    #[serde(default)]
    pub ihr_disabled: Option<bool>,
    /// Whether the message requests a bounce on failed delivery.
    #[serde(default)]
    pub bounce: Option<bool>,
    /// Whether this message is a bounce generated after a failed delivery.
    #[serde(default)]
    pub bounced: Option<bool>,
    /// Fee for importing an external message, in nanograms as decimal text.
    #[serde(default)]
    pub import_fee: Option<String>,
    /// Hash of the transaction that received this message.
    #[serde(default)]
    pub in_msg_tx_hash: Option<String>,
    /// Operation code extracted from the message body.
    #[serde(default)]
    pub opcode: Option<StringOrNumber>,
    /// Hash of the transaction that emitted this message.
    #[serde(default)]
    pub out_msg_tx_hash: Option<String>,
    /// Serialized message body and its decoded representation.
    #[serde(default)]
    pub message_content: Option<MessageContent>,
    /// Serialized deployment state and its decoded representation.
    #[serde(default)]
    pub init_state: Option<MessageContent>,
}

/// Serialized cell content and any decoded representation supplied by the indexer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MessageContent {
    /// Hash of the content cell serialized in `body`.
    #[serde(default)]
    pub hash: Option<String>,
    /// Base64-encoded `BoC` containing the message body cell.
    #[serde(default)]
    pub body: Option<String>,
    /// Decoded cell content supplied by the indexer.
    #[serde(default)]
    pub decoded: Option<Value>,
}

/// Related transactions and messages forming a trace, with indexing and classification
/// status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Trace {
    /// Identifier of the transaction trace associated with the record.
    pub trace_id: String,
    /// Transaction hashes in the traversal order supplied for this trace.
    pub transactions_order: Vec<String>,
    /// Full transaction records keyed by transaction hash.
    pub transactions: HashMap<String, Transaction>,
    /// Whether the trace is missing transactions or messages.
    #[serde(default)]
    pub is_incomplete: bool,
    /// Parsed actions associated with the containing object.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Upper bound of the logical-time range.
    #[serde(default)]
    pub end_lt: Option<String>,
    /// Latest execution time in the range, in Unix seconds.
    #[serde(default)]
    pub end_utime: Option<u32>,
    /// Hash of the external message that initiated the trace.
    #[serde(default)]
    pub external_hash: Option<String>,
    /// Masterchain sequence number at the end of the trace, encoded as decimal text.
    pub mc_seqno_end: String,
    /// Masterchain sequence number at the beginning of the trace, encoded as decimal text.
    pub mc_seqno_start: String,
    /// Lower bound of the logical-time range.
    pub start_lt: String,
    /// Earliest execution time in the range, in Unix seconds.
    pub start_utime: u32,
    /// Transaction tree for the trace, when included.
    #[serde(default)]
    pub trace: Option<TraceNode>,
    /// Counts and indexing/classification status for the trace.
    pub trace_info: TraceInfo,
    /// Diagnostic information supplied by the indexer about the trace.
    #[serde(default)]
    pub warning: Option<String>,
}

/// Transaction/message counts and completion status for a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TraceInfo {
    /// Number of transactions included in the trace.
    pub transactions: usize,
    /// Number of messages associated with the trace.
    pub messages: usize,
    /// Number of messages whose receiving transactions are not yet included.
    pub pending_messages: usize,
    /// Completion state assigned to the trace by the indexer.
    pub trace_state: String,
    /// Status of action classification for the trace.
    pub classification_state: String,
}
