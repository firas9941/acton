//! Exact `TONLib` discriminator values.
//!
//! Single-variant enums prevent structurally similar objects (wallet states,
//! stack entries, and transactions) from being decoded as the wrong variant.
//! Use `Default::default()` when constructing a wire object.

use serde::{Deserialize, Serialize};

macro_rules! tags {
    ($($name:ident => $wire:tt),+ $(,)?) => {
        $(
            #[doc = concat!("Exact `@type` discriminator for `", $wire, "` objects.")]
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
            #[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
            #[cfg_attr(feature = "openapi", schema(as = tags::$name))]
            pub enum $name {
                /// The only valid wire value for this object.
                #[default]
                #[serde(rename = $wire)]
                Value,
            }
        )+
    };
}

tags! {
    AccountAddress => "accountAddress",
    AdnlAddress => "adnlAddress",
    TonBlockIdExt => "ton.blockIdExt",
    DetectAddressBase64Variant => "ext.utils.detectedAddressVariant",
    ExtraCurrencyBalance => "extraCurrency",
    InternalTransactionId => "internal.transactionId",
    AccountStateRaw => "raw.accountState",
    AccountStateWalletV3 => "wallet.v3.accountState",
    AccountStateWalletV4 => "wallet.v4.accountState",
    AccountStateWalletHighloadV1 => "wallet.highload.v1.accountState",
    AccountStateWalletHighloadV2 => "wallet.highload.v2.accountState",
    AccountStateDns => "dns.accountState",
    RWalletLimit => "rwallet.limit",
    RWalletConfig => "rwallet.config",
    AccountStateRWallet => "rwallet.accountState",
    PChanConfig => "pchan.config",
    PChanStateInit => "pchan.stateInit",
    PChanStateClose => "pchan.stateClose",
    PChanStatePayout => "pchan.statePayout",
    AccountStatePChan => "pchan.accountState",
    AccountStateUninited => "uninited.accountState",
    DnsRecordStorageAddress => "dns_storage_address",
    DnsRecordAdnlAddress => "dns_adnl_address",
    SmcAddr => "addr_std",
    DnsRecordSmcAddress => "dns_smc_address",
    DnsRecordNextResolver => "dns_next_resolver",
    JettonMasterData => "ext.tokens.jettonMasterData",
    JettonWalletData => "ext.tokens.jettonWalletData",
    NftCollectionData => "ext.tokens.nftCollectionData",
    NftItemData => "ext.tokens.nftItemData",
    DnsEntryDataUnknown => "dns.entryDataUnknown",
    DnsEntryDataText => "dns.entryDataText",
    DnsEntryDataNextResolver => "dns.entryDataNextResolver",
    DnsEntryDataSmcAddress => "dns.entryDataSmcAddress",
    DnsEntryDataAdnlAddress => "dns.entryDataAdnlAddress",
    DnsEntryDataStorageAddress => "dns.entryDataStorageAddress",
    DnsEntry => "dns.entry",
    DnsResolved => "dns.resolved",
    BlockSignature => "blocks.signature",
    ShardBlockLink => "blocks.shardBlockLink",
    BlockLinkBack => "blocks.blockLinkBack",
    OutMsgQueueSize => "blocks.outMsgQueueSize",
    LibraryEntry => "smc.libraryEntry",
    ShortTxId => "blocks.shortTxId",
    MsgDataRaw => "msg.dataRaw",
    MsgDataText => "msg.dataText",
    MsgDataDecryptedText => "msg.dataDecryptedText",
    MsgDataEncryptedText => "msg.dataEncryptedText",
    MessageStd => "raw.message",
    TransactionStd => "raw.transaction",
    TransactionExt => "raw.transactionExt",
    Message => "ext.message",
    Transaction => "ext.transaction",
    DetectAddress => "ext.utils.detectedAddress",
    DetectHash => "ext.utils.detectedHash",
    AddressInformation => "raw.fullAccountState",
    ExtendedAddressInformation => "fullAccountState",
    WalletInformation => "ext.accounts.walletInformation",
    MasterchainInfo => "blocks.masterchainInfo",
    BlockSignatures => "blocks.blockSignatures",
    BlockSignaturesSimplex => "blocks.blockSignatures.simplex",
    ShardBlockProof => "blocks.shardBlockProof",
    ConsensusBlock => "ext.blocks.consensusBlock",
    Shards => "blocks.shards",
    BlockData => "blocks.blockData",
    BlockHeader => "blocks.header",
    OutMsgQueueSizes => "blocks.outMsgQueueSizes",
    ConfigInfo => "configInfo",
    LibraryResult => "smc.libraryResult",
    BlockTransactions => "blocks.transactions",
    BlockTransactionsExt => "blocks.transactionsExt",
    TransactionsStd => "raw.transactions",
    ExtMessageInfo => "raw.extMessageInfo",
    ResultOk => "ok",
    Fees => "fees",
    QueryFees => "query.fees",
    RunGetMethodStdResult => "smc.runResult",
    RunGetMethodResult => "smc.runResult",
    TvmStackEntrySlice => "tvm.stackEntrySlice",
    TvmStackEntryCell => "tvm.stackEntryCell",
    TvmStackEntryNumber => "tvm.stackEntryNumber",
    TvmStackEntryTuple => "tvm.stackEntryTuple",
    TvmStackEntryList => "tvm.stackEntryList",
    TvmStackEntryUnsupported => "tvm.stackEntryUnsupported",
    TvmSlice => "tvm.slice",
    TvmCell => "tvm.cell",
    TvmNumberDecimal => "tvm.numberDecimal",
    TvmTuple => "tvm.tuple",
    TvmList => "tvm.list",
}
