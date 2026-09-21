//! Responses for the TON Center v2 wire contract.

use super::stack::{LegacyStackEntry, TvmCell, TvmStackEntry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// An account address inside standard `TONLib` objects. An empty address denotes the missing
/// side of an external message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountAddress {
    /// Fixed `TONLib` discriminator `accountAddress`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountAddress,
    /// The account address in raw format (`workchain:hex`), e.g. `0:abc123...`. Use the pack
    /// address endpoint to convert to user-friendly base64 format.
    pub account_address: String,
}

/// An ADNL network address carried by a resolved DNS record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdnlAddress {
    /// Fixed `TONLib` discriminator `adnlAddress`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AdnlAddress,
    /// ADNL network address encoded as base64, separate from a smart-contract account address.
    pub adnl_address: String,
}

/// A complete block identifier with cryptographic hashes. Contains `workchain`, `shard`, `seqno`
/// (position) plus `root_hash` and `file_hash` (verification).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TonBlockIdExt {
    /// Fixed `TONLib` discriminator `ton.blockIdExt`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TonBlockIdExt,
    /// Workchain ID: `-1` for masterchain, `0` for basechain.
    pub workchain: i64,
    /// Shard identifier as a signed 64-bit integer string. Masterchain uses
    /// `-9223372036854775808`.
    pub shard: String,
    /// Block sequence number within its workchain and shard. For the masterchain (workchain `-1`),
    /// this equals the global block height. For basechain shards (workchain `0`), this is the
    /// sequence number local to that specific shard, not a global height.
    pub seqno: i64,
    /// Root hash of the block. Together with `file_hash`, uniquely identifies a block.
    pub root_hash: String,
    /// Hash of the serialized block file. Together with `root_hash`, uniquely identifies a block.
    pub file_hash: String,
}

/// User-friendly address in both standard base64 and URL-safe base64 encodings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DetectAddressBase64Variant {
    /// Fixed `TONLib` discriminator `ext.utils.detectedAddressVariant`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DetectAddressBase64Variant,
    /// Address in standard base64 encoding (48 characters, uses `+` and `/`).
    pub b64: String,
    /// Address in URL-safe base64 encoding (48 characters, uses `-` and `_` instead of `+` and
    /// `/`).
    pub b64url: String,
}

/// Balance of one extra currency. Its identifier selects the currency; the amount is decimal
/// text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExtraCurrencyBalance {
    /// Fixed `TONLib` discriminator `extraCurrency`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ExtraCurrencyBalance,
    /// 32-bit extra-currency identifier, represented by `TONLib` as a signed integer.
    pub id: i32,
    /// Currency balance amount as a decimal string.
    pub amount: String,
}

/// A reference to a specific transaction. The combination of `lt` (logical time) and `hash`
/// uniquely identifies any transaction. Use these values for pagination and lookups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InternalTransactionId {
    /// Fixed `TONLib` discriminator `internal.transactionId`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::InternalTransactionId,
    /// Logical time of the transaction. Use together with its hash for pagination and lookups.
    pub lt: String,
    /// SHA-256 hash of this transaction, encoded in base64.
    pub hash: String,
}

/// Code, data, and frozen-state hash for an account without a recognized specialized state
/// representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateRaw {
    /// Fixed `TONLib` discriminator `raw.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateRaw,
    /// Smart contract code in `BoC` format, base64 encoded. Empty if account is uninitialized.
    pub code: String,
    /// Smart contract persistent storage in `BoC` format, base64 encoded.
    pub data: String,
    /// Frozen account state hash as base64, or an empty string when the account is not frozen.
    pub frozen_hash: String,
}

/// Persistent state of a Wallet V3 account, including its subwallet ID and next sequence
/// number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateWalletV3 {
    /// Fixed `TONLib` discriminator `wallet.v3.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateWalletV3,
    /// Subwallet identifier. Allows creating multiple wallets from one key.
    pub wallet_id: i64,
    /// Wallet contract sequence number. Used for replay protection: each outgoing transaction must
    /// include the current `seqno` and increments it by 1. Fetch the current value before sending
    /// a transaction.
    pub seqno: i32,
}

/// Persistent state of a Wallet V4 account, including its subwallet ID and next sequence
/// number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateWalletV4 {
    /// Fixed `TONLib` discriminator `wallet.v4.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateWalletV4,
    /// Subwallet identifier. Allows creating multiple wallets from one key.
    pub wallet_id: i64,
    /// Wallet contract sequence number. Used for replay protection: each outgoing transaction must
    /// include the current `seqno` and increments it by 1. Fetch the current value before sending
    /// a transaction.
    pub seqno: i32,
}

/// Persistent state of a legacy Highload V1 wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateWalletHighloadV1 {
    /// Fixed `TONLib` discriminator `wallet.highload.v1.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateWalletHighloadV1,
    /// Subwallet identifier. Allows creating multiple wallets from one key.
    pub wallet_id: i64,
    /// Wallet contract sequence number. Used for replay protection: each outgoing transaction must
    /// include the current `seqno` and increments it by 1. Fetch the current value before sending
    /// a transaction.
    pub seqno: i32,
}

/// Subwallet identifier reported for a legacy Highload V2 wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateWalletHighloadV2 {
    /// Fixed `TONLib` discriminator `wallet.highload.v2.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateWalletHighloadV2,
    /// Subwallet identifier. Allows creating multiple wallets from one key.
    pub wallet_id: i64,
}

/// Wallet identifier reported for a legacy `TONLib` DNS account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateDns {
    /// Fixed `TONLib` discriminator `dns.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateDns,
    /// Wallet subwallet identifier stored in contract data, distinct from the account address.
    pub wallet_id: i64,
}

/// One timed spending limit in a restricted-wallet release schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RWalletLimit {
    /// Fixed `TONLib` discriminator `rwallet.limit`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::RWalletLimit,
    /// Duration of this spending limit window in seconds.
    pub seconds: i32,
    /// Maximum spendable amount within this time period, in nanograms.
    pub value: i64,
}

/// Start time and spending limits for a restricted wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RWalletConfig {
    /// Fixed `TONLib` discriminator `rwallet.config`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::RWalletConfig,
    /// Unix timestamp when the spending limits become active.
    pub start_at: i64,
    /// Array of spending limit rules. Each entry defines a time window in seconds and the maximum
    /// spendable amount in nanograms within that window.
    pub limits: Vec<RWalletLimit>,
}

/// Restricted-wallet state, including the amount unlocked under its spending schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateRWallet {
    /// Fixed `TONLib` discriminator `rwallet.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateRWallet,
    /// Subwallet identifier. Allows creating multiple wallets from one key.
    pub wallet_id: i64,
    /// Wallet contract sequence number. Used for replay protection: each outgoing transaction must
    /// include the current `seqno` and increments it by 1. Fetch the current value before sending
    /// a transaction.
    pub seqno: i32,
    /// Balance available for immediate withdrawal, in nanograms.
    pub unlocked_balance: i64,
    /// Spending limits and restrictions configuration.
    pub config: RWalletConfig,
}

/// Participants, public keys, and timeouts governing a payment channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PChanConfig {
    /// Fixed `TONLib` discriminator `pchan.config`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::PChanConfig,
    /// Alice's Ed25519 public key for signing channel operations.
    pub alice_public_key: String,
    /// Alice's on-chain wallet address.
    pub alice_address: AccountAddress,
    /// Bob's Ed25519 public key for signing channel operations.
    pub bob_public_key: String,
    /// Bob's on-chain wallet address.
    pub bob_address: AccountAddress,
    /// Maximum seconds allowed to complete channel initialization before it expires.
    pub init_timeout: i32,
    /// Maximum seconds allowed to complete cooperative channel closure.
    pub close_timeout: i32,
    /// Unique numeric identifier for this payment channel.
    pub channel_id: i64,
}

/// Payment-channel initialization state, including each party's signatures and committed
/// amounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PChanStateInit {
    /// Fixed `TONLib` discriminator `pchan.stateInit`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::PChanStateInit,
    /// Returns `true` if Alice has signed the initialization; otherwise `false`.
    #[serde(rename = "signed_A")]
    pub signed_a: bool,
    /// Returns `true` if Bob has signed the initialization; otherwise `false`.
    #[serde(rename = "signed_B")]
    pub signed_b: bool,
    /// Minimum deposit required from Alice, in nanograms.
    #[serde(rename = "min_A")]
    pub min_a: i64,
    /// Minimum deposit required from Bob, in nanograms.
    #[serde(rename = "min_B")]
    pub min_b: i64,
    /// Unix timestamp when the initialization offer expires.
    pub expire_at: i64,
    /// Alice's deposited amount in nanograms.
    #[serde(rename = "A")]
    pub a: i64,
    /// Bob's deposited amount in nanograms.
    #[serde(rename = "B")]
    pub b: i64,
}

/// Payment-channel closing state while signatures, minimum payouts, and the expiry are tracked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PChanStateClose {
    /// Fixed `TONLib` discriminator `pchan.stateClose`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::PChanStateClose,
    /// Returns `true` if Alice has signed the close proposal; otherwise `false`.
    #[serde(rename = "signed_A")]
    pub signed_a: bool,
    /// Returns `true` if Bob has signed the close proposal; otherwise `false`.
    #[serde(rename = "signed_B")]
    pub signed_b: bool,
    /// Minimum guaranteed payout for Alice, in nanograms.
    #[serde(rename = "min_A")]
    pub min_a: i64,
    /// Minimum guaranteed payout for Bob, in nanograms.
    #[serde(rename = "min_B")]
    pub min_b: i64,
    /// Unix timestamp when the close proposal expires.
    pub expire_at: i64,
    /// Alice's final balance in nanograms.
    #[serde(rename = "A")]
    pub a: i64,
    /// Bob's final balance in nanograms.
    #[serde(rename = "B")]
    pub b: i64,
}

/// Final amounts allocated to the two payment-channel participants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PChanStatePayout {
    /// Fixed `TONLib` discriminator `pchan.statePayout`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::PChanStatePayout,
    /// Alice's payout amount in nanograms.
    #[serde(rename = "A")]
    pub a: i64,
    /// Bob's payout amount in nanograms.
    #[serde(rename = "B")]
    pub b: i64,
}

/// Payment-channel state selected by its `TONLib` discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum PChanState {
    /// Payload in the `PChanStateInit` wire representation.
    PChanStateInit(Box<PChanStateInit>),
    /// Payload in the `PChanStateClose` wire representation.
    PChanStateClose(Box<PChanStateClose>),
    /// Payload in the `PChanStatePayout` wire representation.
    PChanStatePayout(Box<PChanStatePayout>),
}

/// Payment-channel configuration and its current initialization, closing, or payout state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStatePChan {
    /// Fixed `TONLib` discriminator `pchan.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStatePChan,
    /// Payment channel configuration (parties, timeouts, keys).
    pub config: PChanConfig,
    /// Current payment channel state (init, close, or payout phase).
    pub state: PChanState,
    /// Human-readable payment channel description.
    pub description: String,
}

/// State of an uninitialized or frozen account. The frozen hash is empty for an unfrozen
/// account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountStateUninited {
    /// Fixed `TONLib` discriminator `uninited.accountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AccountStateUninited,
    /// Frozen account state hash as base64, or an empty string when the account is not frozen.
    pub frozen_hash: String,
}

/// Specialized account state returned by `getExtendedAddressInformation`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum AccountState {
    /// Payload in the `AccountStateRaw` wire representation.
    AccountStateRaw(Box<AccountStateRaw>),
    /// Payload in the `AccountStateWalletV3` wire representation.
    AccountStateWalletV3(Box<AccountStateWalletV3>),
    /// Payload in the `AccountStateWalletV4` wire representation.
    AccountStateWalletV4(Box<AccountStateWalletV4>),
    /// Payload in the `AccountStateWalletHighloadV1` wire representation.
    AccountStateWalletHighloadV1(Box<AccountStateWalletHighloadV1>),
    /// Payload in the `AccountStateWalletHighloadV2` wire representation.
    AccountStateWalletHighloadV2(Box<AccountStateWalletHighloadV2>),
    /// Payload in the `AccountStateDns` wire representation.
    AccountStateDns(Box<AccountStateDns>),
    /// Payload in the `AccountStateRWallet` wire representation.
    AccountStateRWallet(Box<AccountStateRWallet>),
    /// Payload in the `AccountStatePChan` wire representation.
    AccountStatePChan(Box<AccountStatePChan>),
    /// Payload in the `AccountStateUninited` wire representation.
    AccountStateUninited(Box<AccountStateUninited>),
}

/// On-chain metadata keys and JSON values, preserved without imposing a token-specific vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(transparent)]
pub struct TokenContentDict(
    /// Metadata values preserved as JSON, including application-specific keys.
    pub BTreeMap<String, Value>,
);

/// A TON Storage bag referenced by token DNS metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecordStorageAddress {
    /// Fixed `TONLib` discriminator `dns_storage_address`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsRecordStorageAddress,
    /// TON Storage bag identifier in hex, pointing to distributed file storage.
    pub bag_id: String,
}

/// An ADNL hosting address referenced by token DNS metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecordAdnlAddress {
    /// Fixed `TONLib` discriminator `dns_adnl_address`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsRecordAdnlAddress,
    /// ADNL (Abstract Datagram Network Layer) address in hex, used for TON Sites and services.
    pub adnl_addr: String,
}

/// A standard internal address split into workchain and 256-bit account ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmcAddr {
    /// Fixed `TONLib` discriminator `addr_std`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::SmcAddr,
    /// Workchain ID (`0` for basechain, `-1` for masterchain).
    pub workchain_id: i32,
    /// Account identifier as a 64-character hex string.
    pub address: String,
}

/// A smart-contract address referenced by token DNS metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecordSmcAddress {
    /// Fixed `TONLib` discriminator `dns_smc_address`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsRecordSmcAddress,
    /// Smart contract address associated with this DNS record.
    pub smc_addr: SmcAddr,
}

/// A contract responsible for the next step in domain resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecordNextResolver {
    /// Fixed `TONLib` discriminator `dns_next_resolver`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsRecordNextResolver,
    /// Smart contract address of the next DNS resolver in the chain.
    pub resolver: SmcAddr,
}

/// DNS metadata record selected by its discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum DnsRecord {
    /// Payload in the `DnsRecordStorageAddress` wire representation.
    DnsRecordStorageAddress(Box<DnsRecordStorageAddress>),
    /// Payload in the `DnsRecordSmcAddress` wire representation.
    DnsRecordSmcAddress(Box<DnsRecordSmcAddress>),
    /// Payload in the `DnsRecordAdnlAddress` wire representation.
    DnsRecordAdnlAddress(Box<DnsRecordAdnlAddress>),
    /// Payload in the `DnsRecordNextResolver` wire representation.
    DnsRecordNextResolver(Box<DnsRecordNextResolver>),
}

/// Named DNS metadata records, including additional categories returned by the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsRecordSet {
    /// Next resolver contract for subdomain lookups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_next_resolver: Option<DnsRecord>,
    /// Wallet address associated with this domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet: Option<DnsRecord>,
    /// Site hosting address (ADNL or TON Storage).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<DnsRecord>,
    /// TON Storage bag ID for files hosted under this domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<DnsRecord>,
    /// Additional DNS categories returned by the server, preserved for round trips.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Domain name and DNS records exposed by an NFT item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsContent {
    /// The resolved domain name.
    pub domain: String,
    /// Parsed DNS records for this domain.
    pub data: DnsRecordSet,
}

/// Token metadata as an on-chain dictionary or an off-chain URI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TokenContent {
    /// Storage method for token (jetton) metadata content: `onchain` (stored in contract data) or
    /// `offchain` (external URI).
    #[serde(rename = "type")]
    pub kind: TokenContentType,
    /// Token metadata key-value pairs (name, symbol, decimals, image, description).
    pub data: TokenContentData,
}

/// Jetton master data, including supply, minting permissions, and metadata.
///
/// Query this contract for token-wide settings; individual holder balances belong
/// to jetton wallet contracts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonMasterData {
    /// Fixed `TONLib` discriminator `ext.tokens.jettonMasterData`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::JettonMasterData,
    /// Contract address of this Jetton master.
    pub address: String,
    /// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
    pub contract_type: JettonMasterDataContractType,
    /// Total tokens in circulation, in the smallest unit. Divide by 10^decimals for human-readable
    /// amount.
    pub total_supply: String,
    /// Whether new tokens can currently be minted.
    pub mintable: bool,
    /// Admin address that can mint tokens or update metadata. Empty if admin rights were revoked.
    /// Omitted when the contract stores no address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_address: Option<String>,
    /// Token metadata: name, symbol, decimals, description, and image URL.
    pub jetton_content: TokenContent,
    /// Code used to deploy individual user wallets for this token. Used to verify wallet
    /// authenticity.
    pub jetton_wallet_code: String,
}

/// Jetton wallet contract data. Each user has a separate wallet contract for each token they hold.
/// Contains the token balance, owner address, and reference to the master contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JettonWalletData {
    /// Fixed `TONLib` discriminator `ext.tokens.jettonWalletData`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::JettonWalletData,
    /// Contract address of this Jetton wallet.
    pub address: String,
    /// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
    pub contract_type: JettonWalletDataContractType,
    /// The token balance in the Jetton's base units (smallest denomination). To get the human-
    /// readable amount, divide by 10^decimals, where `decimals` is a metadata field on the Jetton
    /// master contract (commonly 9, but varies per token). Refer to the Jetton overview for how
    /// Jetton decimals are defined.
    pub balance: String,
    /// Wallet owner's address (the user who holds these tokens).
    pub owner: String,
    /// Address of the Jetton master contract this wallet belongs to.
    pub jetton: String,
    /// Returns `true` if the mintless jetton allocation has been claimed; otherwise `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mintless_is_claimed: Option<bool>,
    /// Wallet contract code. Should match the master contract's wallet code to verify
    /// authenticity.
    pub jetton_wallet_code: String,
}

/// NFT collection contract data. Contains the number of items minted, collection owner, and
/// collection metadata (name, description, image).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftCollectionData {
    /// Fixed `TONLib` discriminator `ext.tokens.nftCollectionData`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::NftCollectionData,
    /// Contract address of this NFT collection.
    pub address: String,
    /// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
    pub contract_type: NftCollectionDataContractType,
    /// Index that will be assigned to the next minted NFT. Also indicates total items if minted
    /// sequentially.
    pub next_item_index: String,
    /// Collection owner's address. Controls minting and metadata updates. Omitted when the
    /// contract stores no address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_address: Option<String>,
    /// Collection metadata: name, description, and cover image.
    pub collection_content: TokenContent,
}

/// Individual NFT data. Contains the item index, collection reference, current owner, and item-
/// specific content/metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftItemData {
    /// Fixed `TONLib` discriminator `ext.tokens.nftItemData`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::NftItemData,
    /// Contract address of this NFT item.
    pub address: String,
    /// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
    pub contract_type: NftItemDataContractType,
    /// Returns `true` if this NFT item has been initialized (deployed); otherwise `false`.
    pub init: bool,
    /// This item's sequential index within its collection.
    pub index: String,
    /// Address of the NFT collection this item belongs to. Empty if standalone NFT. Omitted when
    /// the contract stores no address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_address: Option<String>,
    /// Current owner's address. Changes on each transfer. Omitted when the contract stores no
    /// address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_address: Option<String>,
    /// NFT item metadata and content (name, description, image URL).
    pub content: NftItemDataContent,
}

/// Data returned by `getTokenData` for a jetton master, jetton wallet, NFT collection, or NFT
/// item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum TokenData {
    /// Payload in the `JettonMasterData` wire representation.
    JettonMasterData(Box<JettonMasterData>),
    /// Payload in the `JettonWalletData` wire representation.
    JettonWalletData(Box<JettonWalletData>),
    /// Payload in the `NftCollectionData` wire representation.
    NftCollectionData(Box<NftCollectionData>),
    /// Payload in the `NftItemData` wire representation.
    NftItemData(Box<NftItemData>),
}

/// A DNS record whose payload has no specialized `TONLib` representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsEntryDataUnknown {
    /// Fixed `TONLib` discriminator `dns.entryDataUnknown`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsEntryDataUnknown,
    /// Unrecognized DNS record payload serialized as base64 bytes.
    pub bytes: String,
}

/// Plain text returned by a DNS resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsEntryDataText {
    /// Fixed `TONLib` discriminator `dns.entryDataText`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsEntryDataText,
    /// Text stored in a `TONLib` DNS text record; this is plain text, not base64.
    pub text: String,
}

/// Resolver contract for the remaining domain labels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsEntryDataNextResolver {
    /// Fixed `TONLib` discriminator `dns.entryDataNextResolver`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsEntryDataNextResolver,
    /// Contract responsible for resolving the remaining domain labels.
    pub resolver: AccountAddress,
}

/// Smart-contract address returned by a DNS resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsEntryDataSmcAddress {
    /// Fixed `TONLib` discriminator `dns.entryDataSmcAddress`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsEntryDataSmcAddress,
    /// Smart-contract account referenced by the DNS record.
    pub smc_address: AccountAddress,
}

/// ADNL endpoint returned by a DNS resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsEntryDataAdnlAddress {
    /// Fixed `TONLib` discriminator `dns.entryDataAdnlAddress`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsEntryDataAdnlAddress,
    /// ADNL endpoint referenced by the DNS record.
    pub adnl_address: AdnlAddress,
}

/// TON Storage bag returned by a DNS resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsEntryDataStorageAddress {
    /// Fixed `TONLib` discriminator `dns.entryDataStorageAddress`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsEntryDataStorageAddress,
    /// TON Storage bag identifier as a 256-bit hexadecimal hash.
    pub bag_id: String,
}

/// Resolved DNS payload selected by its `TONLib` discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum DnsEntryData {
    /// Payload in the `DnsEntryDataUnknown` wire representation.
    DnsEntryDataUnknown(Box<DnsEntryDataUnknown>),
    /// Payload in the `DnsEntryDataText` wire representation.
    DnsEntryDataText(Box<DnsEntryDataText>),
    /// Payload in the `DnsEntryDataNextResolver` wire representation.
    DnsEntryDataNextResolver(Box<DnsEntryDataNextResolver>),
    /// Payload in the `DnsEntryDataSmcAddress` wire representation.
    DnsEntryDataSmcAddress(Box<DnsEntryDataSmcAddress>),
    /// Payload in the `DnsEntryDataAdnlAddress` wire representation.
    DnsEntryDataAdnlAddress(Box<DnsEntryDataAdnlAddress>),
    /// Payload in the `DnsEntryDataStorageAddress` wire representation.
    DnsEntryDataStorageAddress(Box<DnsEntryDataStorageAddress>),
}

/// One resolved domain and category, with its typed record payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsEntry {
    /// Fixed `TONLib` discriminator `dns.entry`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsEntry,
    /// Domain name attached to this resolved entry.
    pub name: String,
    /// DNS category key as a base64-encoded 256-bit hash.
    pub category: String,
    /// Typed DNS record payload; the @type discriminator determines the variant.
    pub entry: DnsEntryData,
}

/// DNS lookup result. An empty entries array means the resolver found no matching records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsResolved {
    /// Fixed `TONLib` discriminator `dns.resolved`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DnsResolved,
    /// Resolved DNS entries. An empty array means no matching record was found.
    pub entries: Vec<DnsEntry>,
}

/// Account lifecycle state reported by the network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum AccountStateEnum {
    /// Wire value `uninitialized`.
    #[serde(rename = "uninitialized")]
    Uninitialized,
    /// Wire value `active`.
    #[serde(rename = "active")]
    Active,
    /// Wire value `frozen`.
    #[serde(rename = "frozen")]
    Frozen,
}

/// One validator signature over a block; use its node identifier to select the verification
/// key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockSignature {
    /// Fixed `TONLib` discriminator `blocks.signature`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockSignature,
    /// Short public key hash of the validator node that produced this signature.
    pub node_id_short: String,
    /// Ed25519 signature of the block by this validator, base64 encoded.
    pub signature: String,
}

/// Proof link connecting a shard block to the masterchain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ShardBlockLink {
    /// Fixed `TONLib` discriminator `blocks.shardBlockLink`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ShardBlockLink,
    /// Block identifier for this link in the proof chain.
    pub id: TonBlockIdExt,
    /// Merkle proof data for this shard block link.
    pub proof: String,
}

/// Backward masterchain proof link, including destination, block, and state proofs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockLinkBack {
    /// Fixed `TONLib` discriminator `blocks.blockLinkBack`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockLinkBack,
    /// Returns `true` if the destination block is a key block; otherwise `false`.
    pub to_key_block: bool,
    /// Source block in the proof chain.
    pub from: TonBlockIdExt,
    /// Destination block in the proof chain.
    pub to: TonBlockIdExt,
    /// Cryptographic proof for the destination block.
    pub dest_proof: String,
    /// Merkle proof linking the two blocks.
    pub proof: String,
    /// State proof validating the block link.
    pub state_proof: String,
}

/// Outbound message queue length for one shard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OutMsgQueueSize {
    /// Fixed `TONLib` discriminator `blocks.outMsgQueueSize`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::OutMsgQueueSize,
    /// Full block identifier for the shard.
    pub id: TonBlockIdExt,
    /// Number of outgoing messages waiting in this shard's queue.
    pub size: i64,
}

/// Shared library cell retrieved by its hash, with its base64 `BoC`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibraryEntry {
    /// Fixed `TONLib` discriminator `smc.libraryEntry`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::LibraryEntry,
    /// Library cell hash.
    pub hash: String,
    /// Library cell code in `BoC` format, base64 encoded.
    pub data: String,
}

/// Transaction cursor within a block. The account, logical time, and hash identify its full
/// transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ShortTxId {
    /// Fixed `TONLib` discriminator `blocks.shortTxId`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ShortTxId,
    /// Bitmask indicating which optional fields are present.
    pub mode: i64,
    /// Account address that executed this transaction.
    pub account: String,
    /// Logical time of the transaction. Use together with its hash for pagination and lookups.
    pub lt: String,
    /// SHA-256 hash of this transaction, encoded in base64.
    pub hash: String,
}

/// Binary message body and optional deployment state, represented as base64 `BoC`s.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MsgDataRaw {
    /// Fixed `TONLib` discriminator `msg.dataRaw`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::MsgDataRaw,
    /// Raw message body in `BoC` format, base64 encoded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Base64 `BoC` containing the message `StateInit`, or an empty string when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init_state: Option<String>,
}

/// Text message payload encoded as base64 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MsgDataText {
    /// Fixed `TONLib` discriminator `msg.dataText`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::MsgDataText,
    /// UTF-8 text comment, base64 encoded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Decrypted text message payload encoded as base64 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MsgDataDecryptedText {
    /// Fixed `TONLib` discriminator `msg.dataDecryptedText`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::MsgDataDecryptedText,
    /// Decrypted UTF-8 text content, base64 encoded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Encrypted text message payload encoded as base64 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MsgDataEncryptedText {
    /// Fixed `TONLib` discriminator `msg.dataEncryptedText`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::MsgDataEncryptedText,
    /// Encrypted message payload, base64 encoded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Message body representation selected by its `TONLib` discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum MsgData {
    /// Payload in the `MsgDataRaw` wire representation.
    MsgDataRaw(Box<MsgDataRaw>),
    /// Payload in the `MsgDataText` wire representation.
    MsgDataText(Box<MsgDataText>),
    /// Payload in the `MsgDataDecryptedText` wire representation.
    MsgDataDecryptedText(Box<MsgDataDecryptedText>),
    /// Payload in the `MsgDataEncryptedText` wire representation.
    MsgDataEncryptedText(Box<MsgDataEncryptedText>),
}

/// Message with structured sender and recipient addresses, transferred value, fees, and payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MessageStd {
    /// Fixed `TONLib` discriminator `raw.message`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::MessageStd,
    /// SHA-256 hash of this message, encoded in base64.
    pub hash: String,
    /// Sender account address. Empty for external inbound messages.
    pub source: AccountAddress,
    /// Recipient account address.
    pub destination: AccountAddress,
    /// GRAM amount transferred with this message, in nanograms.
    pub value: String,
    /// Array of extra currencies (native blockchain-level tokens, distinct from Jettons)
    /// transferred in this message. Each entry contains a currency ID (integer) and amount
    /// (decimal string). Empty array if no extra currencies were transferred.
    pub extra_currencies: Vec<ExtraCurrencyBalance>,
    /// Forwarding fee deducted from the message value, in nanograms.
    pub fwd_fee: String,
    /// Instant Hypercube Routing (IHR) fee in nanograms.
    pub ihr_fee: String,
    /// Logical time when this message was created.
    pub created_lt: String,
    /// Hash of the message body.
    pub body_hash: String,
    /// Message body payload (raw, text comment, or encrypted).
    pub msg_data: MsgData,
}

/// Standard `TONLib` transaction with account-address objects in its messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionStd {
    /// Fixed `TONLib` discriminator `raw.transaction`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TransactionStd,
    /// Account address object for this transaction.
    pub address: AccountAddress,
    /// Unix timestamp when this transaction was executed.
    pub utime: i64,
    /// Full transaction data serialized as `BoC`, base64 encoded.
    pub data: String,
    /// The transaction identifier, represented by logical time and transaction hash.
    pub transaction_id: InternalTransactionId,
    /// Total fees paid for this transaction, in nanograms.
    pub fee: String,
    /// Storage fees in nanograms deducted from the account balance during the storage phase of this
    /// transaction. Covers the cost of storing the contract state on-chain since the last
    /// transaction.
    pub storage_fee: String,
    /// Fees in nanograms deducted from the account balance during the computation and action phases
    /// of this transaction. Covers the cost of TVM execution and processing outgoing messages.
    pub other_fee: String,
    /// The inbound message that triggered this transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_msg: Option<MessageStd>,
    /// Array of outbound messages produced by this transaction. Each message contains sender and
    /// recipient address objects, GRAM amount (nanograms as string), forwarding fee, and raw message
    /// body.
    pub out_msgs: Vec<MessageStd>,
}

/// Block transaction with a direct account address and standard `TONLib` message objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionExt {
    /// Fixed `TONLib` discriminator `raw.transactionExt`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TransactionExt,
    /// Account address object for this transaction.
    pub address: AccountAddress,
    /// Unix timestamp when this transaction was executed.
    pub utime: i64,
    /// Full transaction data serialized as `BoC`, base64 encoded.
    pub data: String,
    /// The transaction identifier, represented by logical time and transaction hash.
    pub transaction_id: InternalTransactionId,
    /// Total fees paid for this transaction, in nanograms.
    pub fee: String,
    /// Storage fees in nanograms deducted from the account balance during the storage phase of this
    /// transaction. Covers the cost of storing the contract state on-chain since the last
    /// transaction.
    pub storage_fee: String,
    /// Fees in nanograms deducted from the account balance during the computation and action phases
    /// of this transaction. Covers the cost of TVM execution and processing outgoing messages.
    pub other_fee: String,
    /// The inbound message that triggered this transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_msg: Option<MessageStd>,
    /// Array of outbound messages produced by this transaction. Each message contains sender and
    /// recipient address objects, GRAM amount (nanograms as string), forwarding fee, and raw message
    /// body.
    pub out_msgs: Vec<MessageStd>,
    /// The account address that this transaction belongs to.
    pub account: String,
}

/// A message between accounts. Contains sender, recipient, GRAM amount transferred, message
/// creation logical time, and the message body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Message {
    /// Fixed `TONLib` discriminator `ext.message`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::Message,
    /// SHA-256 hash of this message, encoded in base64.
    pub hash: String,
    /// Sender address. Empty for external inbound messages.
    pub source: String,
    /// Recipient address.
    pub destination: String,
    /// GRAM amount transferred with this message, in nanograms.
    pub value: String,
    /// Array of extra currencies (native blockchain-level tokens, distinct from Jettons)
    /// transferred in this message. Each entry contains a currency ID (integer) and amount
    /// (decimal string). Empty array if no extra currencies were transferred.
    pub extra_currencies: Vec<ExtraCurrencyBalance>,
    /// Forwarding fee deducted from the message value, in nanograms.
    pub fwd_fee: String,
    /// Instant Hypercube Routing (IHR) fee in nanograms.
    pub ihr_fee: String,
    /// Logical time when this message was created.
    pub created_lt: String,
    /// Hash of the message body. Useful for verifying content without the full payload.
    pub body_hash: String,
    /// Message body payload (raw, text comment, or encrypted).
    pub msg_data: MsgData,
    /// Decoded UTF-8 text comment from the message body. Present only when the message body is a
    /// plain text comment (opcode `0x00000000`). Empty string otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Error description if the message body could not be decoded as a text comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_decode_error: Option<String>,
}

/// Account transaction with decoded messages, fees, and its complete serialized `BoC`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Transaction {
    /// Fixed `TONLib` discriminator `ext.transaction`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::Transaction,
    /// Account address object for this transaction.
    pub address: AccountAddress,
    /// Account address in raw format.
    pub account: String,
    /// Unix timestamp when this transaction was executed.
    pub utime: i64,
    /// Full transaction data serialized as `BoC`, base64 encoded.
    pub data: String,
    /// The transaction identifier, represented by logical time and transaction hash.
    pub transaction_id: InternalTransactionId,
    /// Total fees paid for this transaction, in nanograms.
    pub fee: String,
    /// Storage fees in nanograms deducted from the account balance during the storage phase of this
    /// transaction. Covers the cost of storing the contract state on-chain since the last
    /// transaction.
    pub storage_fee: String,
    /// Fees in nanograms deducted from the account balance during the computation and action phases
    /// of this transaction. Covers the cost of TVM execution and processing outgoing messages.
    pub other_fee: String,
    /// The inbound message that triggered this transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_msg: Option<Message>,
    /// Array of outbound messages produced by this transaction. Each message contains sender,
    /// recipient, GRAM amount (nanograms as string), forwarding fee, message body, and optional
    /// decoded text comment.
    pub out_msgs: Vec<Message>,
}

/// An address converted to raw, bounceable, and non-bounceable forms.
///
/// Includes standard and URL-safe base64 encodings, the original input format,
/// and its testnet-only flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DetectAddress {
    /// Fixed `TONLib` discriminator `ext.utils.detectedAddress`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DetectAddress,
    /// Raw address in `workchain_id:account_id` hex format (e.g., `0:abc...def`). This is the
    /// canonical on-chain representation without flags or checksum.
    pub raw_form: String,
    /// Bounceable form of the address in standard base64 and URL-safe base64. Bounceable addresses
    /// (prefix `E...`) instruct wallet software to set the bounce flag on outgoing messages, so
    /// funds are returned if the destination contract cannot process them.
    pub bounceable: DetectAddressBase64Variant,
    /// Non-bounceable form of the address in standard base64 and URL-safe base64. Non-bounceable
    /// addresses (prefix `U...`) instruct wallet software to clear the bounce flag, ensuring funds
    /// are credited even if the recipient has no deployed contract.
    pub non_bounceable: DetectAddressBase64Variant,
    /// The encoding format of the address as provided in the request (e.g., `raw_form`, `dns`,
    /// `friendly_bounceable`, `friendly_non_bounceable`).
    pub given_type: DetectAddressGivenType,
    /// Returns `true` if the user-friendly address has the testnet-only flag set (`0x80`);
    /// otherwise `false`. This is a flag in the address encoding that tells wallet software to
    /// reject this address on mainnet. The underlying account itself can exist on both networks.
    pub test_only: bool,
}

/// Detected hash in all supported encoding formats: hex (64 characters), standard base64 (44
/// characters), and URL-safe base64.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DetectHash {
    /// Fixed `TONLib` discriminator `ext.utils.detectedHash`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::DetectHash,
    /// Hash in standard base64 encoding (44 characters, uses `+` and `/`).
    pub b64: String,
    /// Hash in URL-safe base64 encoding (44 characters, uses `-` and `_` instead of `+` and `/`).
    pub b64url: String,
    /// Hash in lowercase hexadecimal encoding (64 characters).
    pub hex: String,
}

/// Typed alternatives for `PackAddress`. Deserialization preserves the wire variant and its
/// protocol-specific fields.
pub type PackAddress = String;

/// Typed alternatives for `UnpackAddress`. Deserialization preserves the wire variant and its
/// protocol-specific fields.
pub type UnpackAddress = String;

/// Raw account state including balance, code, data, and status. The code and data fields contain
/// the smart contract in `BoC` format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddressInformation {
    /// Fixed `TONLib` discriminator `raw.fullAccountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::AddressInformation,
    /// Account balance in nanograms as a decimal string. 1 GRAM = 1,000,000,000 nanograms.
    pub balance: String,
    /// Array of non-GRAM currency balances held by this account. Each entry contains a currency ID
    /// (integer) and balance amount (decimal string). Empty array if the account holds no extra
    /// currencies.
    pub extra_currencies: Vec<ExtraCurrencyBalance>,
    /// Reference to the most recent transaction. Use as starting point for getTransactions.
    pub last_transaction_id: InternalTransactionId,
    /// Full block identifier where this event occurred.
    pub block_id: TonBlockIdExt,
    /// Smart contract code in `BoC` format, base64 encoded.
    pub code: String,
    /// Smart contract persistent storage in `BoC` format, base64 encoded.
    pub data: String,
    /// Frozen account state hash as base64, or an empty string when the account is not frozen.
    pub frozen_hash: String,
    /// Unix timestamp of the block from which this data was read.
    pub sync_utime: i64,
    /// Account state: uninitialized, active, or frozen.
    pub state: AccountStateEnum,
    /// Whether the account is suspended by the network. May be omitted by older server builds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspended: Option<bool>,
}

/// Account balance, last transaction, and parsed state for recognized contract types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExtendedAddressInformation {
    /// Fixed `TONLib` discriminator `fullAccountState`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ExtendedAddressInformation,
    /// Account address object.
    pub address: AccountAddress,
    /// Account balance in nanograms as a decimal string. 1 GRAM = 1,000,000,000 nanograms.
    pub balance: String,
    /// Array of non-GRAM currency balances held by this account. Each entry contains a currency ID
    /// (integer) and balance amount (decimal string). Empty array if the account holds no extra
    /// currencies.
    pub extra_currencies: Vec<ExtraCurrencyBalance>,
    /// Reference to the most recent transaction. Use as starting point for getTransactions.
    pub last_transaction_id: InternalTransactionId,
    /// Full block identifier where this event occurred.
    pub block_id: TonBlockIdExt,
    /// Unix timestamp of the block from which this data was read.
    pub sync_utime: i64,
    /// Detailed account state including contract type and internal data.
    pub account_state: AccountState,
    /// Contract revision number. Different revisions may have different behavior.
    pub revision: i64,
}

/// Information about a wallet account. Check `wallet` before using wallet-specific fields such as
/// `wallet_type`, `seqno`, and `wallet_id`; available fields depend on the wallet type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WalletInformation {
    /// Fixed `TONLib` discriminator `ext.accounts.walletInformation`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::WalletInformation,
    /// Returns `true` if this address is a recognized wallet contract type; otherwise `false`.
    pub wallet: bool,
    /// Account balance in nanograms as a decimal string.
    pub balance: String,
    /// Account lifecycle state: `uninitialized`, `active`, or `frozen`.
    pub account_state: AccountStateEnum,
    /// Reference to the most recent transaction. Use as starting point for getTransactions.
    pub last_transaction_id: InternalTransactionId,
    /// Recognized wallet contract type, such as `wallet v4 r2`, `wallet v5 r1`, or `tg-wallet`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_type: Option<WalletInformationWalletType>,
    /// Wallet contract sequence number. Used for replay protection: each outgoing transaction must
    /// include the current `seqno` and increments it by 1. Fetch the current value before sending
    /// a transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seqno: Option<i64>,
    /// Subwallet identifier. Allows creating multiple wallets from one key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<i64>,
    /// Whether signature authentication is enabled for this wallet (v5 wallets only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_signature_allowed: Option<bool>,
}

/// Shared wire representation for `AddressBalance`; see the underlying type for field semantics.
pub type AddressBalance = String;

/// Shared wire representation for `AddressState`; see the underlying type for field semantics.
pub type AddressState = AccountStateEnum;

/// Information about the latest masterchain block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MasterchainInfo {
    /// Fixed `TONLib` discriminator `blocks.masterchainInfo`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::MasterchainInfo,
    /// The most recent masterchain block.
    pub last: TonBlockIdExt,
    /// Base64 hash of the masterchain state referenced by this response.
    pub state_root_hash: String,
    /// Zero-state block identifier used to identify the network; it is not the current head.
    pub init: TonBlockIdExt,
}

/// Validator signatures for a masterchain block under catchain consensus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockSignatures {
    /// Fixed `TONLib` discriminator `blocks.blockSignatures`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockSignatures,
    /// Exact masterchain block covered by these validator signatures.
    pub id: TonBlockIdExt,
    /// Validator signatures for this exact block. Applications own cryptographic verification.
    pub signatures: Vec<BlockSignature>,
}

/// Validator signatures and candidate data for a masterchain block under Simplex consensus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockSignaturesSimplex {
    /// Fixed `TONLib` discriminator `blocks.blockSignatures.simplex`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockSignaturesSimplex,
    /// Exact masterchain block covered by these Simplex signatures.
    pub id: TonBlockIdExt,
    /// Validator signatures for this exact block. Applications own cryptographic verification.
    pub signatures: Vec<BlockSignature>,
    /// Base64 identifier of the Simplex consensus session which produced the signatures.
    pub session_id: String,
    /// Slot number within the Simplex consensus session.
    pub slot: i32,
    /// Base64-encoded serialized Simplex candidate data needed to verify the signatures.
    pub candidate: String,
}

/// Catchain or Simplex block signatures, selected by the response discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum MasterchainBlockSignatures {
    /// Payload in the `BlockSignatures` wire representation.
    BlockSignatures(Box<BlockSignatures>),
    /// Payload in the `BlockSignaturesSimplex` wire representation.
    BlockSignaturesSimplex(Box<BlockSignaturesSimplex>),
}

/// Proof connecting a shard block to a masterchain block and any requested earlier masterchain
/// height.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ShardBlockProof {
    /// Fixed `TONLib` discriminator `blocks.shardBlockProof`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ShardBlockProof,
    /// The shard block being proven.
    pub from: TonBlockIdExt,
    /// Masterchain block used as trust anchor for the proof.
    pub mc_id: TonBlockIdExt,
    /// Array of cryptographic links forming a proof chain from the shard block to the masterchain.
    /// Each link contains a block identifier and a Merkle proof (base64).
    pub links: Vec<ShardBlockLink>,
    /// Array of masterchain block link proofs. Each entry contains source and destination block
    /// identifiers with corresponding Merkle proofs (base64) and state proofs (base64).
    pub mc_proof: Vec<BlockLinkBack>,
}

/// Masterchain sequence number agreed on by the available liteservers, with observation time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConsensusBlock {
    /// Fixed `TONLib` discriminator `ext.blocks.consensusBlock`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ConsensusBlock,
    /// Sequence number of the latest consensus block.
    pub consensus_block: i32,
    /// Unix time in seconds when the server observed the consensus height.
    pub timestamp: i32,
}

/// Shared wire representation for `LookupBlock`; see the underlying type for field semantics.
pub type LookupBlock = TonBlockIdExt;

/// Shard blocks referenced by a masterchain block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Shards {
    /// Fixed `TONLib` discriminator `blocks.shards`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::Shards,
    /// Array of active shard block identifiers. Each entry contains `workchain` (integer), `shard`
    /// ID (string), `seqno` (integer), `root_hash` (base64/hex), and `file_hash` (base64/hex).
    pub shards: Vec<TonBlockIdExt>,
}

/// Complete serialized block `BoC` and the identifier of its exact bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockData {
    /// Fixed `TONLib` discriminator `blocks.blockData`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockData,
    /// Extended identifier of the returned block.
    pub id: TonBlockIdExt,
    /// Full block BOC encoded as base64.
    pub data: String,
}

/// Block metadata and links to previous blocks, without the serialized block body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockHeader {
    /// Fixed `TONLib` discriminator `blocks.header`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockHeader,
    /// Full block identifier including workchain, shard, `seqno`, and hashes.
    pub id: TonBlockIdExt,
    /// Global network identifier.
    pub global_id: i64,
    /// Block format version number.
    pub version: i64,
    /// Returns `true` if this block was created immediately after a shard merge; otherwise
    /// `false`.
    pub after_merge: bool,
    /// Returns `true` if this block was created immediately after a shard split; otherwise
    /// `false`.
    pub after_split: bool,
    /// Returns `true` if this shard will split after this block; otherwise `false`.
    pub before_split: bool,
    /// Returns `true` if validators have signaled a preference to merge this shard; otherwise
    /// `false`.
    pub want_merge: bool,
    /// Returns `true` if validators have signaled a preference to split this shard; otherwise
    /// `false`.
    pub want_split: bool,
    /// Short hash of the validator set active during this block.
    pub validator_list_hash_short: i64,
    /// Catchain sequence number used for validator consensus.
    pub catchain_seqno: i64,
    /// Minimum masterchain block `seqno` referenced by this block.
    pub min_ref_mc_seqno: i64,
    /// Returns `true` if this is a key block containing validator set changes or config updates;
    /// otherwise `false`.
    pub is_key_block: bool,
    /// Sequence number of the previous key block.
    pub prev_key_block_seqno: i64,
    /// Logical time at the start of this block.
    pub start_lt: String,
    /// Ending logical time.
    pub end_lt: String,
    /// Unix timestamp when this block was generated.
    pub gen_utime: i64,
    /// Array of previous block identifiers (`workchain`, `shard`, `seqno`, `root_hash`,
    /// `file_hash`). Usually contains one entry, but two after a shard merge.
    pub prev_blocks: Vec<TonBlockIdExt>,
}

/// Outbound queue lengths across shards and the external queue size limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OutMsgQueueSizes {
    /// Fixed `TONLib` discriminator `blocks.outMsgQueueSizes`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::OutMsgQueueSizes,
    /// Array of per-shard queue sizes. Each entry contains the shard's block identifier and the
    /// number of outgoing messages waiting in its queue (integer).
    pub shards: Vec<OutMsgQueueSize>,
    /// Limit for the external message queue size.
    pub ext_msg_queue_size_limit: i64,
}

/// Blockchain configuration encoded in a TVM cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConfigInfo {
    /// Fixed `TONLib` discriminator `configInfo`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ConfigInfo,
    /// The requested configuration parameter value as a TVM cell.
    pub config: TvmCell,
}

/// Shared libraries found for the requested hashes. Missing libraries are absent from the
/// result array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibraryResult {
    /// Fixed `TONLib` discriminator `smc.libraryResult`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::LibraryResult,
    /// Array of library entries. Each entry contains a hash (base64 or hex) identifying the
    /// library and a data field with the library code in `BoC` format (base64).
    pub result: Vec<LibraryEntry>,
}

/// Paginated transaction identifiers for a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockTransactions {
    /// Fixed `TONLib` discriminator `blocks.transactions`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockTransactions,
    /// Full block identifier for the queried block.
    pub id: TonBlockIdExt,
    /// The maximum number of transactions requested, as specified by the `count` request
    /// parameter. If `incomplete` is `true`, more transactions exist beyond this limit.
    pub req_count: i64,
    /// Returns `true` if there are more transactions in this block; otherwise `false`. Use the
    /// last transaction as cursor for pagination.
    pub incomplete: bool,
    /// Array of compact transaction references. Each entry contains the account address (string),
    /// logical time (string), and transaction hash (base64 or hex).
    pub transactions: Vec<ShortTxId>,
}

/// Paginated full transactions for a block, with standard `TONLib` messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BlockTransactionsExt {
    /// Fixed `TONLib` discriminator `blocks.transactionsExt`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::BlockTransactionsExt,
    /// Full block identifier for the queried block.
    pub id: TonBlockIdExt,
    /// The maximum number of transactions requested, as specified by the `count` request
    /// parameter. If `incomplete` is `true`, more transactions exist beyond this limit.
    pub req_count: i64,
    /// Returns `true` if there are more transactions in this block; otherwise `false`. Use the
    /// last transaction as cursor for pagination.
    pub incomplete: bool,
    /// Array of full transaction objects. Each entry contains the account address, timestamps,
    /// inbound/outbound messages, fees (in nanograms), and the raw transaction `BoC` (base64).
    pub transactions: Vec<TransactionExt>,
}

/// Typed alternatives for Transactions. Deserialization preserves the wire variant and its
/// protocol-specific fields.
pub type Transactions = Vec<Transaction>;

/// Account transaction history and the cursor for the preceding transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionsStd {
    /// Fixed `TONLib` discriminator `raw.transactions`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TransactionsStd,
    /// Array of transaction objects in standardized format. Each transaction contains address,
    /// timestamps, inbound/outbound messages, fees in nanograms, and raw `BoC` (base64).
    pub transactions: Vec<TransactionStd>,
    /// Use this as cursor to fetch the next page of older transactions.
    pub previous_transaction_id: InternalTransactionId,
}

/// External-message hashes returned after submission. Query the chain to establish inclusion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExtMessageInfo {
    /// Fixed `TONLib` discriminator `raw.extMessageInfo`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ExtMessageInfo,
    /// SHA-256 hash of the external message as accepted by the network.
    pub hash: String,
    /// Base64 hash of the normalized external message, useful for matching on-chain inclusion.
    pub hash_norm: String,
}

/// Acknowledgment of successful message submission. It does not establish on-chain inclusion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResultOk {
    /// Fixed `TONLib` discriminator `ok`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::ResultOk,
}

/// Estimated forwarding, storage, and execution fees in nanograms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Fees {
    /// Fixed `TONLib` discriminator `fees`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::Fees,
    /// Fee for importing this inbound message, in nanograms.
    pub in_fwd_fee: i64,
    /// Storage fee charged for keeping the contract state on-chain, in nanograms.
    pub storage_fee: i64,
    /// Computation gas fee for executing contract code, in nanograms.
    pub gas_fee: i64,
    /// Fee for forwarding outbound messages created by this transaction, in nanograms.
    pub fwd_fee: i64,
}

/// Fee estimate for the source transaction and any destination transactions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueryFees {
    /// Fixed `TONLib` discriminator `query.fees`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::QueryFees,
    /// Fees charged on the sending account.
    pub source_fees: Fees,
    /// Fee estimates for destination transactions; may be empty when only the source is emulated.
    pub destination_fees: Vec<Fees>,
}

/// Get-method execution result using the standard `TONLib` stack format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct RunGetMethodStdResult {
    /// Fixed `TONLib` discriminator `smc.runResult`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::RunGetMethodStdResult,
    /// Gas consumed during execution. Useful for cost estimation.
    pub gas_used: i64,
    /// Output values as a TVM stack. Each entry is a typed object with `@type` discriminator:
    /// `tvm.stackEntryNumber` (decimal string), `tvm.stackEntryCell` (base64 `BoC`),
    /// `tvm.stackEntrySlice` (base64 `BoC`), `tvm.stackEntryTuple`, or `tvm.stackEntryList`.
    pub stack: Vec<TvmStackEntry>,
    /// TVM exit code. `0` or `1` means success; other values indicate errors. Refer to the TVM
    /// exit code reference for all values.
    pub exit_code: i32,
}

/// Get-method execution result using the legacy stack format, with the queried block and
/// transaction cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RunGetMethodResult {
    /// Fixed `TONLib` discriminator `smc.runResult`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::RunGetMethodResult,
    /// Gas consumed during execution. Useful for cost estimation.
    pub gas_used: i64,
    /// Ordered result stack in legacy two-element array format. Nested tuples and lists contain
    /// standard `TONLib` stack entries.
    pub stack: Vec<LegacyStackEntry>,
    /// TVM exit code. `0` or `1` means success; other values indicate errors. Refer to the TVM
    /// exit code reference for all values.
    pub exit_code: i32,
    /// Block at which the get method was executed.
    pub block_id: TonBlockIdExt,
    /// Most recent transaction at the time of execution.
    pub last_transaction_id: InternalTransactionId,
}

/// Recognized wallet contract type, such as `wallet v4 r2`, `wallet v5 r1`, or `tg-wallet`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum WalletInformationWalletType {
    /// Wire value `wallet v1 r1`.
    #[serde(rename = "wallet v1 r1")]
    WalletV1R1,
    /// Wire value `wallet v1 r2`.
    #[serde(rename = "wallet v1 r2")]
    WalletV1R2,
    /// Wire value `wallet v1 r3`.
    #[serde(rename = "wallet v1 r3")]
    WalletV1R3,
    /// Wire value `wallet v2 r1`.
    #[serde(rename = "wallet v2 r1")]
    WalletV2R1,
    /// Wire value `wallet v2 r2`.
    #[serde(rename = "wallet v2 r2")]
    WalletV2R2,
    /// Wire value `wallet v3 r1`.
    #[serde(rename = "wallet v3 r1")]
    WalletV3R1,
    /// Wire value `wallet v3 r2`.
    #[serde(rename = "wallet v3 r2")]
    WalletV3R2,
    /// Wire value `wallet v4 r1`.
    #[serde(rename = "wallet v4 r1")]
    WalletV4R1,
    /// Wire value `wallet v4 r2`.
    #[serde(rename = "wallet v4 r2")]
    WalletV4R2,
    /// Wire value `wallet v5 beta`.
    #[serde(rename = "wallet v5 beta")]
    WalletV5Beta,
    /// Wire value `wallet v5 r1`.
    #[serde(rename = "wallet v5 r1")]
    WalletV5R1,
    /// Wire value `tg-wallet`.
    #[serde(rename = "tg-wallet")]
    TgWallet,
}

/// The encoding format of the address as provided in the request (e.g., `raw_form`, `dns`,
/// `friendly_bounceable`, `friendly_non_bounceable`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum DetectAddressGivenType {
    /// Wire value `raw_form`.
    #[serde(rename = "raw_form")]
    RawForm,
    /// Wire value `friendly_bounceable`.
    #[serde(rename = "friendly_bounceable")]
    FriendlyBounceable,
    /// Wire value `friendly_non_bounceable`.
    #[serde(rename = "friendly_non_bounceable")]
    FriendlyNonBounceable,
}

/// NFT item metadata and content (name, description, image URL).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum NftItemDataContent {
    /// Payload in the `TokenContent` wire representation.
    TokenContent(Box<TokenContent>),
    /// Payload in the `DnsContent` wire representation.
    DnsContent(Box<DnsContent>),
}

/// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum NftItemDataContractType {
    /// Wire value `nft_item`.
    #[serde(rename = "nft_item")]
    NftItem,
}

/// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum NftCollectionDataContractType {
    /// Wire value `nft_collection`.
    #[serde(rename = "nft_collection")]
    NftCollection,
}

/// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum JettonWalletDataContractType {
    /// Wire value `jetton_wallet`.
    #[serde(rename = "jetton_wallet")]
    JettonWallet,
}

/// Type of token contract: `jetton_master`, `jetton_wallet`, `nft_collection`, or `nft_item`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum JettonMasterDataContractType {
    /// Wire value `jetton_master`.
    #[serde(rename = "jetton_master")]
    JettonMaster,
}

/// Token metadata key-value pairs (name, symbol, decimals, image, description).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum TokenContentData {
    /// Payload in the `String` wire representation.
    String(String),
    /// Payload in the `TokenContentDict` wire representation.
    TokenContentDict(Box<TokenContentDict>),
}

/// Storage method for token (jetton) metadata content: `onchain` (stored in contract data) or
/// `offchain` (external URI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum TokenContentType {
    /// Wire value `onchain`.
    #[serde(rename = "onchain")]
    Onchain,
    /// Wire value `offchain`.
    #[serde(rename = "offchain")]
    Offchain,
}

/// Result union for callers dispatching arbitrary JSON-RPC methods. Prefer the
/// endpoint's associated response type when the method is known in advance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum RpcResult {
    /// Result returned by the `DetectAddress` endpoint family.
    DetectAddress(Box<DetectAddress>),
    /// Result returned by the `DetectHash` endpoint family.
    DetectHash(Box<DetectHash>),
    /// Result returned by the `String` endpoint family.
    String(String),
    /// Result returned by the `AddressInformation` endpoint family.
    AddressInformation(Box<AddressInformation>),
    /// Result returned by the `ExtendedAddressInformation` endpoint family.
    ExtendedAddressInformation(Box<ExtendedAddressInformation>),
    /// Result returned by the `TvmCell` endpoint family.
    TvmCell(Box<TvmCell>),
    /// Result returned by the `WalletInformation` endpoint family.
    WalletInformation(Box<WalletInformation>),
    /// Result returned by the `TokenData` endpoint family.
    TokenData(Box<TokenData>),
    /// Result returned by the `DnsResolved` endpoint family.
    DnsResolved(Box<DnsResolved>),
    /// Result returned by the `MasterchainInfo` endpoint family.
    MasterchainInfo(Box<MasterchainInfo>),
    /// Result returned by the `MasterchainBlockSignatures` endpoint family.
    MasterchainBlockSignatures(Box<MasterchainBlockSignatures>),
    /// Result returned by the `ShardBlockProof` endpoint family.
    ShardBlockProof(Box<ShardBlockProof>),
    /// Result returned by the `ConsensusBlock` endpoint family.
    ConsensusBlock(Box<ConsensusBlock>),
    /// Result returned by the `TonBlockIdExt` endpoint family.
    TonBlockIdExt(Box<TonBlockIdExt>),
    /// Result returned by the `Shards` endpoint family.
    Shards(Box<Shards>),
    /// Result returned by the `BlockData` endpoint family.
    BlockData(Box<BlockData>),
    /// Result returned by the `BlockHeader` endpoint family.
    BlockHeader(Box<BlockHeader>),
    /// Result returned by the `OutMsgQueueSizes` endpoint family.
    OutMsgQueueSizes(Box<OutMsgQueueSizes>),
    /// Result returned by the `BlockTransactions` endpoint family.
    BlockTransactions(Box<BlockTransactions>),
    /// Result returned by the `BlockTransactionsExt` endpoint family.
    BlockTransactionsExt(Box<BlockTransactionsExt>),
    /// Result returned by the `Transactions` endpoint family.
    Transactions(Transactions),
    /// Result returned by the `TransactionsStd` endpoint family.
    TransactionsStd(Box<TransactionsStd>),
    /// Result returned by the `Transaction` endpoint family.
    Transaction(Box<Transaction>),
    /// Result returned by the `ConfigInfo` endpoint family.
    ConfigInfo(Box<ConfigInfo>),
    /// Result returned by the `LibraryResult` endpoint family.
    LibraryResult(Box<LibraryResult>),
    /// Result returned by the `RunGetMethodResult` endpoint family.
    RunGetMethodResult(Box<RunGetMethodResult>),
    /// Result returned by the `RunGetMethodStdResult` endpoint family.
    RunGetMethodStdResult(Box<RunGetMethodStdResult>),
    /// Result returned by the `ResultOk` endpoint family.
    ResultOk(Box<ResultOk>),
    /// Result returned by the `ExtMessageInfo` endpoint family.
    ExtMessageInfo(Box<ExtMessageInfo>),
    /// Result returned by the `QueryFees` endpoint family.
    QueryFees(Box<QueryFees>),
}
