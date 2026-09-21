//! Endpoint metadata connects each request with its result type.
//!
//! These zero-sized markers do not perform I/O. Transports may use `Endpoint` to
//! construct typed calls while retaining ownership of authentication and retries.

use super::{requests, responses, stack};
use serde::{Serialize, de::DeserializeOwned};

/// One REST operation and the corresponding JSON-RPC method.
/// The associated response is the result inside `TonlibResponse`, not the envelope.
pub trait Endpoint {
    /// Parameters serialized as a POST body or individual GET query parameters.
    type Request: Serialize + DeserializeOwned;
    /// Successful result before the common transport envelope is added.
    type Response: Serialize + DeserializeOwned;
    /// Exact case-sensitive method name used by the JSON-RPC proxy.
    const METHOD: &'static str;
    /// Absolute route, including the `/api/v2` prefix.
    const PATH: &'static str;
    /// Whether GET is supported in addition to POST.
    const SUPPORTS_GET: bool;
    /// User-facing explanation of the operation and its relevant limitations.
    const DESCRIPTION: &'static str;
}

macro_rules! endpoints {
    ($($name:ident, $method:literal, $request:ty, $response:ty, $get:literal, $description:literal;)+) => {
        $(
            #[doc = $description]
            #[derive(Debug, Clone, Copy)]
            pub struct $name;

            impl Endpoint for $name {
                type Request = $request;
                type Response = $response;
                const METHOD: &'static str = $method;
                const PATH: &'static str = concat!("/api/v2/", $method);
                const SUPPORTS_GET: bool = $get;
                const DESCRIPTION: &'static str = $description;
            }
        )+

        /// Case-sensitive JSON-RPC method names, including `shards` and `sendBocReturnHashNoError`.
        pub const METHODS: &[&str] = &[$($method),+];

        #[cfg(feature = "openapi")]
        pub(super) fn register(document: &mut super::openapi::Document) {
            $(document.endpoint::<$name>();)+
        }
    };
}

endpoints! {
    DetectAddress, "detectAddress",
        requests::DetectAddressRequest, responses::DetectAddress, true,
        "Validates an address and returns it in all standard formats. Use this to convert between address formats or to validate user input. Returns raw format (0:abc), base64 bounceable (EQ), base64 non-bounceable (UQ), and URL-safe variants.";
    DetectHash, "detectHash",
        requests::DetectHashRequest, responses::DetectHash, true,
        "Validates a hash and returns it in all standard formats. Use this to convert between hex (64 chars) and base64 (44 chars) representations. Works with any 256-bit hash including transaction hashes, block hashes, and message hashes.";
    PackAddress, "packAddress",
        requests::PackAddressRequest, String, true,
        "Converts a raw address to user-friendly base64 format. Raw addresses use the format `workchain:hex` (e.g., `0:abc...`). The packed format is shorter and includes a checksum for error detection.";
    UnpackAddress, "unpackAddress",
        requests::UnpackAddressRequest, String, true,
        "Converts a user-friendly base64 address to a raw address string in `workchain:hex` format.";
    GetAddressInformation, "getAddressInformation",
        requests::AddressInformationRequest, responses::AddressInformation, true,
        "Returns the current state of any account on the TON blockchain. Includes the balance (in nanograms), smart contract code and data (if deployed), account status, and a reference to the last transaction. This is the primary endpoint for checking if an address exists and what's deployed there.";
    GetExtendedAddressInformation, "getExtendedAddressInformation",
        requests::ExtendedAddressInformationRequest, responses::ExtendedAddressInformation, true,
        "Returns detailed account information with parsed contract state. For recognized contract types, returns type-specific state fields. For other contracts, returns the raw state.";
    GetShardAccountCell, "getShardAccountCell",
        requests::ShardAccountCellRequest, stack::TvmCell, true,
        "Get raw TVM cell with shard account";
    GetWalletInformation, "getWalletInformation",
        requests::WalletInformationRequest, responses::WalletInformation, true,
        "Returns wallet-specific information for an address. If the address is a known wallet contract, returns the wallet type, current `seqno` (needed for sending transactions), and `wallet_id`. Always check `wallet: true` before using wallet-specific fields. Call this before sending any transaction to get the current `seqno`.";
    GetAddressBalance, "getAddressBalance",
        requests::AddressBalanceRequest, String, true,
        "Returns the GRAM balance of an account in nanograms. 1 GRAM = 1,000,000,000 nanograms. A lightweight endpoint that returns only the balance without contract code, data, or other account details. Returns \"0\" for addresses that have never received any funds.";
    GetAddressState, "getAddressState",
        requests::AddressStateRequest, responses::AccountStateEnum, true,
        "Returns the account lifecycle state: `uninitialized` (no contract deployed), `active` (contract deployed), or `frozen` (contract state frozen).";
    GetTokenData, "getTokenData",
        requests::TokenDataRequest, responses::TokenData, true,
        "Returns metadata for Jetton or NFT contracts. Automatically detects the contract type and returns appropriate fields. For Jetton masters: total supply, admin, metadata. For Jetton wallets: balance, owner. For NFT items: collection, owner, content. For NFT collections: item count, metadata.";
    DnsResolve, "dnsResolve",
        requests::DnsResolveRequest, responses::DnsResolved, true,
        "Resolve TON DNS contract";
    GetMasterchainInfo, "getMasterchainInfo",
        requests::MasterchainInfoRequest, responses::MasterchainInfo, true,
        "Returns the current state of the TON masterchain. The `last` field contains the latest block used for querying current state. The `seqno` in `last` is the current block height. Use this endpoint to obtain the latest block reference for other queries.";
    GetMasterchainBlockSignatures, "getMasterchainBlockSignatures",
        requests::MasterchainBlockSignaturesRequest, responses::MasterchainBlockSignatures, true,
        "Returns validator signatures for a specific masterchain block. Each signature proves that a validator approved this block. Use this for building cryptographic proofs or verifying block authenticity in trustless applications.";
    GetShardBlockProof, "getShardBlockProof",
        requests::ShardBlockProofRequest, responses::ShardBlockProof, true,
        "Returns a Merkle proof that links a shardchain block to a masterchain block. This proof cryptographically verifies that the shard block is part of the canonical chain. Used by light clients and cross-chain bridges to verify shard data without trusting the API.";
    GetConsensusBlock, "getConsensusBlock",
        requests::ConsensusBlockRequest, responses::ConsensusBlock, true,
        "Get block that was confirmed by consensus";
    LookupBlock, "lookupBlock",
        requests::LookupBlockRequest, responses::TonBlockIdExt, true,
        "Finds a block by position or time. Specify workchain and shard, then provide exactly one of `seqno` (block sequence number), `lt` (logical time), or `unixtime` (Unix timestamp). Returns the full block identifier including hashes.";
    GetShards, "getShards",
        requests::ShardsRequest, responses::Shards, true,
        "Returns the active shardchain block identifiers at a given masterchain block height. Each shard processes a subset of accounts in parallel. The response shows how the basechain is currently partitioned and which block each shard is at.";
    GetBlock, "getBlock",
        requests::BlockDataRequest, responses::BlockData, true,
        "Get raw block data as a base64-encoded BOC";
    GetBlockHeader, "getBlockHeader",
        requests::BlockHeaderRequest, responses::BlockHeader, true,
        "Returns block metadata without the full transaction list. Includes timestamps, validator info, and references to previous blocks. Intended for block explorers and other use cases that require block information without transactions.";
    GetOutMsgQueueSize, "getOutMsgQueueSize",
        requests::OutMsgQueueSizeRequest, responses::OutMsgQueueSizes, true,
        "Returns the current size of the outbound message queue for each shard. A growing queue indicates network congestion. If the queue is large, transactions may take longer to process. Monitor this to detect network issues.";
    GetBlockTransactions, "getBlockTransactions",
        requests::BlockTransactionsRequest, responses::BlockTransactions, true,
        "Returns a summary of transactions in a specific block. Each item contains the account address and transaction ID, but not full transaction details. Use `count` to limit results and `after_lt`/`after_hash` for pagination. Call getTransactions with each transaction ID to get full details.";
    GetBlockTransactionsExt, "getBlockTransactionsExt",
        requests::BlockTransactionsExtRequest, responses::BlockTransactionsExt, true,
        "Returns full transaction objects for transactions in a specific block. Each transaction includes complete data: inbound and outbound messages, fees, and BoC-encoded raw data. Use `count` to limit results and `after_lt`/`after_hash` for pagination when `incomplete` is true.";
    GetTransactions, "getTransactions",
        requests::TransactionsRequest, responses::Transactions, true,
        "Returns transaction history for an account. Transactions are returned newest-first. Each transaction shows the incoming message that triggered it, all outgoing messages, and fees paid. For pagination: use the `lt` and `hash` from the oldest transaction as the starting point for the next request.";
    GetTransactionsStd, "getTransactionsStd",
        requests::TransactionsRequest, responses::TransactionsStd, true,
        "Returns transaction history for an account in a standardized format. Transactions are returned newest-first. Each transaction includes the triggering inbound message, all outbound messages, and fees paid. The response includes a `previous_transaction_id` cursor for paginating through older transactions.";
    TryLocateTx, "tryLocateTx",
        requests::TryLocateTxRequest, responses::Transaction, true,
        "Finds a transaction by message parameters. Given a source address, destination address, and message creation time (`created_lt`), returns the transaction that processed this message. Useful for locating when a previously sent message was executed.";
    TryLocateResultTx, "tryLocateResultTx",
        requests::TryLocateResultTxRequest, responses::Transaction, true,
        "Finds the transaction that received a specific message. Given message parameters, returns the transaction on the destination account that processed the incoming message. Use this to trace message delivery across accounts.";
    TryLocateSourceTx, "tryLocateSourceTx",
        requests::TryLocateSourceTxRequest, responses::Transaction, true,
        "Finds the transaction that sent a specific message. Given message parameters, returns the transaction on the source account that created this outgoing message. Useful for tracing where a message originated from.";
    GetConfigParam, "getConfigParam",
        requests::ConfigParamRequest, responses::ConfigInfo, true,
        "Returns a specific blockchain configuration parameter. TON stores all network settings on-chain as numbered parameters. Common ones: 0 (config contract), 1 (elector), 15 (election timing), 17 (stake limits), 20-21 (gas prices), 34 (current validators). Check TON documentation for the full list.";
    GetConfigAll, "getConfigAll",
        requests::ConfigAllRequest, responses::ConfigInfo, true,
        "Returns all blockchain configuration parameters at once. Includes gas prices, validator settings, workchain configs, and governance rules. Use the optional `seqno` to get historical configuration at a specific block height.";
    GetLibraries, "getLibraries",
        requests::LibrariesRequest, responses::LibraryResult, true,
        "Returns smart contract library code by hash. Some contracts reference shared libraries instead of including all code directly. When a library reference appears in contract code, this endpoint fetches the actual library implementation.";
    RunGetMethod, "runGetMethod",
        requests::RunGetMethodRequest, responses::RunGetMethodResult, false,
        "Executes a read-only method on a smart contract. Get methods query contract state without sending a transaction. Common methods include `seqno` (wallet sequence number), `get_wallet_data` (wallet info), and `get_jetton_data` (token info). Method arguments are provided in the `stack` array.";
    RunGetMethodStd, "runGetMethodStd",
        requests::RunGetMethodStdRequest, responses::RunGetMethodStdResult, false,
        "Executes a read-only method on a smart contract using typed stack entries. Input and output stack entries use explicit types (`TvmStackEntryNumber`, `TvmStackEntryCell`, etc.) for structured input/output handling. Common methods: `seqno` (wallet sequence number), `get_wallet_data` (wallet info), `get_jetton_data` (token info).";
    SendBoc, "sendBoc",
        requests::SendBocRequest, responses::ResultOk, false,
        "Broadcasts a signed message to the TON network. The `boc` parameter must contain a complete, signed external message in base64 format. The API validates the message and forwards it to validators. Returns immediately after acceptance; use getTransactions to confirm the transaction was processed.";
    SendBocReturnHash, "sendBocReturnHash",
        requests::SendBocRequest, responses::ExtMessageInfo, false,
        "Broadcasts a signed message to the TON network and returns the message hash. The `boc` parameter must contain a complete, signed external message in base64 format. The API validates the message and forwards it to validators. The returned hash can be used to track the message's processing status.";
    EstimateFee, "estimateFee",
        requests::EstimateFeeRequest, responses::QueryFees, false,
        "Calculates the fees required to send a message. Provide the destination address and message body. For new contract deployments, also include `init_code` and `init_data`. Set `ignore_chksig` to true when estimating before signing. Returns a breakdown of storage, gas, and forwarding fees.";
    ShardsAlias, "shards",
        requests::ShardsRequest, responses::Shards, true,
        "C++ compatibility spelling of getShards, present in the server route and JSON-RPC allowlist.";
    SendBocReturnHashNoError, "sendBocReturnHashNoError",
        requests::SendBocRequest, responses::ExtMessageInfo, false,
        "C++ broadcast route returning message hashes. Despite its name, validation and `TONLib` failures still return errors; a successful response does not prove inclusion.";
}
