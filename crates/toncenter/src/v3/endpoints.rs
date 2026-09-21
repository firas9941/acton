//! Typed routes for the supported TON Center v3 operations.
//!
//! Transports use these markers to pair requests with responses and select the
//! HTTP method. Authentication, rate limits, and retries belong to the transport.

use serde::{Serialize, de::DeserializeOwned};

use super::{requests, responses};

/// HTTP method accepted by a v3 route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// Parameters are encoded in the URL query.
    Get,
    /// Parameters are encoded as a JSON request body.
    Post,
}

/// Associates a v3 route with its parameters and successful JSON response.
pub trait Endpoint {
    /// Query parameters for GET, or the JSON body for POST.
    type Request: Serialize + DeserializeOwned;
    /// Successful JSON body, without a v2 response envelope.
    type Response: Serialize + DeserializeOwned;
    /// HTTP method accepted by the route.
    const METHOD: HttpMethod;
    /// Absolute route, including the `/api/v3` prefix.
    const PATH: &'static str;
    /// Stable operation name used in the generated `OpenAPI` document.
    const OPERATION_ID: &'static str;
    /// Purpose of the operation and relevant usage constraints.
    const DESCRIPTION: &'static str;
}

macro_rules! endpoints {
    ($($name:ident, $method:ident, $path:tt, $request:ty, $response:ty, $description:literal;)+) => {
        $(
            #[doc = $description]
            #[derive(Debug, Clone, Copy)]
            pub struct $name;

            impl Endpoint for $name {
                type Request = $request;
                type Response = $response;
                const METHOD: HttpMethod = HttpMethod::$method;
                const PATH: &'static str = concat!("/api/v3/", $path);
                const OPERATION_ID: &'static str = stringify!($name);
                const DESCRIPTION: &'static str = $description;
            }
        )+

        /// Supported routes and their HTTP methods, including the `/api/v3` prefix.
        pub const ROUTES: &[(HttpMethod, &str)] = &[$(($name::METHOD, $name::PATH)),+];

        #[cfg(feature = "openapi")]
        pub(super) fn register(document: &mut super::openapi::Document) {
            $(document.endpoint::<$name>();)+
        }
    };
}

endpoints! {
    GetMasterchainInfo, Get, "masterchainInfo",
        requests::MasterchainInfoQuery, responses::MasterchainInfo,
        "Returns the oldest and newest masterchain blocks available in the index.";
    GetMasterchainBlockShardState, Get, "masterchainBlockShardState",
        requests::MasterchainBlockShardStateQuery, responses::BlocksResponse,
        "Returns the shard blocks referenced by the state of a masterchain block.";
    GetMasterchainBlockShards, Get, "masterchainBlockShards",
        requests::MasterchainBlockShardsQuery, responses::BlocksResponse,
        "Returns shard blocks committed by a masterchain block, with pagination.";
    GetAddressInformation, Get, "addressInformation",
        requests::AddressInformationQuery, responses::V2AddressInformation,
        "Returns an account's balance, lifecycle state, code, data, and last transaction reference.";
    GetAddressBook, Get, "addressBook",
        requests::AddressesQuery, responses::AddressBook,
        "Returns address presentation and detected contract interfaces, keyed by address.";
    GetMetadata, Get, "metadata",
        requests::AddressesQuery, responses::Metadata,
        "Returns indexing status and token or NFT metadata, keyed by address.";
    GetWalletInformation, Get, "walletInformation",
        requests::WalletInformationQuery, responses::V2WalletInformation,
        "Returns wallet detection and version-specific state, including the sequence counter and wallet identifier.";
    GetAccountStates, Get, "accountStates",
        requests::AccountStatesQuery, responses::AccountStatesResponse,
        "Returns indexed account states, with optional serialized contract code and data.";
    GetTraces, Get, "traces",
        requests::TracesQuery, responses::TracesResponse,
        "Returns transaction traces selected by account, hash, block, or execution range. Trace status indicates whether indexing is complete.";
    GetPendingActions, Get, "pendingActions",
        requests::PendingActionsQuery, responses::ActionsResponse,
        "Returns parsed actions in pending traces. Pending results may change before transactions are included on chain.";
    GetPendingTraces, Get, "pendingTraces",
        requests::PendingTracesQuery, responses::TracesResponse,
        "Returns pending traces selected by account or external-message hash. Pending results do not prove on-chain inclusion.";
    GetDnsRecords, Get, "dns/records",
        requests::DnsRecordsQuery, responses::DnsRecordsResponse,
        "Returns indexed DNS records selected by domain or wallet address.";
    GetJettonBurns, Get, "jetton/burns",
        requests::JettonBurnsQuery, responses::JettonBurnsResponse,
        "Returns indexed jetton burns with their amounts, owners, and originating transactions.";
    GetJettonTransfers, Get, "jetton/transfers",
        requests::JettonTransfersQuery, responses::JettonTransfersResponse,
        "Returns indexed jetton transfers with participants, amounts, and notification payloads.";
    GetNftCollections, Get, "nft/collections",
        requests::NftCollectionsQuery, responses::NftCollectionsResponse,
        "Returns indexed NFT collection states and content metadata.";
    GetNftSales, Get, "nft/sales",
        requests::NftSalesQuery, responses::NftSalesResponse,
        "Returns NFT sale and auction contracts with their type-specific details.";
    GetNftTransfers, Get, "nft/transfers",
        requests::NftTransfersQuery, responses::NftTransfersResponse,
        "Returns indexed NFT ownership transfers and their notification payloads.";
    GetMultisigOrders, Get, "multisig/orders",
        requests::MultisigOrdersQuery, responses::MultisigOrdersResponse,
        "Returns multisig orders, approval state, and optionally decoded outgoing actions.";
    GetMultisigWallets, Get, "multisig/wallets",
        requests::MultisigWalletsQuery, responses::MultisigsResponse,
        "Returns multisig contracts and their authorized participants, with optional orders.";
    GetVesting, Get, "vesting",
        requests::VestingQuery, responses::VestingContractsResponse,
        "Returns vesting schedules and participants. The whitelist filter can include contracts that permit transfers to a wallet.";
    GetTransactions, Get, "transactions",
        requests::TransactionsQuery, responses::TransactionsResponse,
        "Returns indexed transactions selected by account, block, hash, or execution range.";
    GetBlocks, Get, "blocks",
        requests::BlocksQuery, responses::BlocksResponse,
        "Returns indexed block headers, predecessor references, and transaction counts.";
    GetTransactionsByMessage, Get, "transactionsByMessage",
        requests::TransactionsByMessageQuery, responses::TransactionsResponse,
        "Returns transactions linked to messages matching the supplied hashes, body hash, or opcode.";
    GetTransactionsByMasterchainBlock, Get, "transactionsByMasterchainBlock",
        requests::TransactionsByMasterchainBlockQuery, responses::TransactionsResponse,
        "Returns transactions associated with a masterchain block, with pagination and ordering.";
    GetMessages, Get, "messages",
        requests::MessagesQuery, responses::MessagesResponse,
        "Returns indexed messages, transferred values, and serialized or decoded content.";
    GetAdjacentTransactions, Get, "adjacentTransactions",
        requests::AdjacentTransactionsQuery, responses::TransactionsResponse,
        "Returns transactions connected to a transaction through incoming or outgoing messages.";
    GetWalletStates, Get, "walletStates",
        requests::WalletStatesQuery, responses::WalletStatesResponse,
        "Returns indexed wallet detection results and version-specific wallet state for the requested accounts.";
    GetTopAccountsByBalance, Get, "topAccountsByBalance",
        requests::TopAccountsByBalanceQuery, Vec<responses::AccountBalance>,
        "Returns a page of accounts ordered by descending GRAM balance, with amounts in nanograms.";
    EstimateFee, Post, "estimateFee",
        requests::EstimateFeeRequest, responses::EstimateFeeResult,
        "Estimates message-processing fees for the sender and destination contracts without broadcasting the message.";
    GetPendingTransactions, Get, "pendingTransactions",
        requests::PendingTransactionsQuery, responses::TransactionsResponse,
        "Returns pending transactions for the selected accounts and traces. Pending results do not prove on-chain inclusion.";
    GetJettonMasters, Get, "jetton/masters",
        requests::JettonMastersQuery, responses::JettonMastersResponse,
        "Returns indexed jetton master states, supply, administration, and token metadata.";
    GetJettonWallets, Get, "jetton/wallets",
        requests::JettonWalletsQuery, responses::JettonWalletsResponse,
        "Returns indexed jetton wallets, their owners, and balances in the token's smallest units.";
    GetNftItems, Get, "nft/items",
        requests::NftItemsQuery, responses::NftItemsResponse,
        "Returns indexed NFT ownership, collection membership, content, and sale status.";
    SendMessage, Post, "message",
        requests::SendMessageRequest, responses::SendMessageResult,
        "Submits an external message for broadcast and returns its hash. Acceptance does not prove on-chain inclusion.";
    RunGetMethod, Post, "runGetMethod",
        requests::RunGetMethodRequest, responses::RunGetMethodResult,
        "Executes a contract get method with the supplied stack and returns its exit code, gas usage, and output stack.";
}
