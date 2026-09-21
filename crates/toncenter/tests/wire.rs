//! Independent wire examples derived from the pinned C++ schema and converters.

use serde_json::{Value, json};
use toncenter::v2::{TonlibErrorResponse, TonlibResponse, requests, responses, stack};

#[test]
fn extended_stacks_require_an_explicit_entry_type() {
    let request = json!({
        "method": "runGetMethod",
        "params": { "address": "account", "method": "value", "stack": [["null", null]] }
    });
    let fixtures: Value = serde_json::from_str(include_str!("fixtures/wire.json")).unwrap();
    let mut result = fixtures["RunGetMethodResult"][0].clone();
    result["stack"] = json!([["null", null]]);

    let extended_request: requests::JsonRpcRequest<Value> =
        serde_json::from_value(request.clone()).unwrap();
    let extended_result: responses::RunGetMethodResult<Value> =
        serde_json::from_value(result.clone()).unwrap();
    let summary = json!({
        "default_request_rejects_null": serde_json::from_value::<requests::JsonRpcRequest>(request.clone()).is_err(),
        "default_result_rejects_null": serde_json::from_value::<responses::RunGetMethodResult>(result.clone()).is_err(),
        "extended_request_preserves_null": serde_json::to_value(extended_request).unwrap() == request,
        "extended_result_preserves_null": serde_json::to_value(extended_result).unwrap() == result,
    });
    expect_test::expect![[r#"
        {
          "default_request_rejects_null": true,
          "default_result_rejects_null": true,
          "extended_request_preserves_null": true,
          "extended_result_preserves_null": true
        }"#]]
    .assert_eq(&serde_json::to_string_pretty(&summary).unwrap());
}

#[test]
fn every_request_and_response_round_trips_the_reference_snapshot() {
    let fixtures: Value =
        serde_json::from_str(include_str!("fixtures/wire.json")).expect("fixture JSON");
    let mut actual = serde_json::Map::new();

    macro_rules! check {
        ($name:literal, $ty:ty) => {{
            let mut values = Vec::new();
            for raw in fixtures[$name].as_array().expect($name) {
                let decoded: $ty = serde_json::from_value(raw.clone()).expect($name);
                values.push(serde_json::to_value(decoded).expect($name));
                #[cfg(feature = "openapi")]
                validate_schema::<$ty>(raw);
            }
            actual.insert($name.to_owned(), Value::Array(values));
        }};
    }

    check!("EmptyRequest", requests::EmptyRequest);
    check!("AddressRequest", requests::AddressRequest);
    check!("AddressWithSeqnoRequest", requests::AddressWithSeqnoRequest);
    check!("SeqnoRequest", requests::SeqnoRequest);
    check!("DetectAddressRequest", requests::DetectAddressRequest);
    check!("DetectHashRequest", requests::DetectHashRequest);
    check!("PackAddressRequest", requests::PackAddressRequest);
    check!("UnpackAddressRequest", requests::UnpackAddressRequest);
    check!(
        "AddressInformationRequest",
        requests::AddressInformationRequest
    );
    check!("ShardAccountCellRequest", requests::ShardAccountCellRequest);
    check!(
        "ExtendedAddressInformationRequest",
        requests::ExtendedAddressInformationRequest
    );
    check!(
        "WalletInformationRequest",
        requests::WalletInformationRequest
    );
    check!("AddressBalanceRequest", requests::AddressBalanceRequest);
    check!("AddressStateRequest", requests::AddressStateRequest);
    check!("TokenDataRequest", requests::TokenDataRequest);
    check!("DnsResolveRequest", requests::DnsResolveRequest);
    check!("MasterchainInfoRequest", requests::MasterchainInfoRequest);
    check!(
        "MasterchainBlockSignaturesRequest",
        requests::MasterchainBlockSignaturesRequest
    );
    check!("ShardBlockProofRequest", requests::ShardBlockProofRequest);
    check!("ConsensusBlockRequest", requests::ConsensusBlockRequest);
    check!("LookupBlockRequest", requests::LookupBlockRequest);
    check!("ShardsRequest", requests::ShardsRequest);
    check!("BlockDataRequest", requests::BlockDataRequest);
    check!("BlockHeaderRequest", requests::BlockHeaderRequest);
    check!("OutMsgQueueSizeRequest", requests::OutMsgQueueSizeRequest);
    check!(
        "BlockTransactionsRequest",
        requests::BlockTransactionsRequest
    );
    check!(
        "BlockTransactionsExtRequest",
        requests::BlockTransactionsExtRequest
    );
    check!("TransactionsRequest", requests::TransactionsRequest);
    check!("TryLocateTxRequest", requests::TryLocateTxRequest);
    check!(
        "TryLocateResultTxRequest",
        requests::TryLocateResultTxRequest
    );
    check!(
        "TryLocateSourceTxRequest",
        requests::TryLocateSourceTxRequest
    );
    check!("ConfigParamRequest", requests::ConfigParamRequest);
    check!("ConfigAllRequest", requests::ConfigAllRequest);
    check!("LibrariesRequest", requests::LibrariesRequest);
    check!("SendBocRequest", requests::SendBocRequest);
    check!("EstimateFeeRequest", requests::EstimateFeeRequest);
    check!("AccountAddress", responses::AccountAddress);
    check!("AdnlAddress", responses::AdnlAddress);
    check!("TonBlockIdExt", responses::TonBlockIdExt);
    check!(
        "DetectAddressBase64Variant",
        responses::DetectAddressBase64Variant
    );
    check!("ExtraCurrencyBalance", responses::ExtraCurrencyBalance);
    check!("InternalTransactionId", responses::InternalTransactionId);
    check!("AccountStateRaw", responses::AccountStateRaw);
    check!("AccountStateWalletV3", responses::AccountStateWalletV3);
    check!("AccountStateWalletV4", responses::AccountStateWalletV4);
    check!(
        "AccountStateWalletHighloadV1",
        responses::AccountStateWalletHighloadV1
    );
    check!(
        "AccountStateWalletHighloadV2",
        responses::AccountStateWalletHighloadV2
    );
    check!("AccountStateDns", responses::AccountStateDns);
    check!("RWalletLimit", responses::RWalletLimit);
    check!("RWalletConfig", responses::RWalletConfig);
    check!("AccountStateRWallet", responses::AccountStateRWallet);
    check!("PChanConfig", responses::PChanConfig);
    check!("PChanStateInit", responses::PChanStateInit);
    check!("PChanStateClose", responses::PChanStateClose);
    check!("PChanStatePayout", responses::PChanStatePayout);
    check!("PChanState", responses::PChanState);
    check!("AccountStatePChan", responses::AccountStatePChan);
    check!("AccountStateUninited", responses::AccountStateUninited);
    check!("AccountState", responses::AccountState);
    check!("TokenContentDict", responses::TokenContentDict);
    check!(
        "DnsRecordStorageAddress",
        responses::DnsRecordStorageAddress
    );
    check!("DnsRecordAdnlAddress", responses::DnsRecordAdnlAddress);
    check!("SmcAddr", responses::SmcAddr);
    check!("DnsRecordSmcAddress", responses::DnsRecordSmcAddress);
    check!("DnsRecordNextResolver", responses::DnsRecordNextResolver);
    check!("DnsRecord", responses::DnsRecord);
    check!("DnsRecordSet", responses::DnsRecordSet);
    check!("DnsContent", responses::DnsContent);
    check!("TokenContent", responses::TokenContent);
    check!("JettonMasterData", responses::JettonMasterData);
    check!("JettonWalletData", responses::JettonWalletData);
    check!("NftCollectionData", responses::NftCollectionData);
    check!("NftItemData", responses::NftItemData);
    check!("TokenData", responses::TokenData);
    check!("DnsEntryDataUnknown", responses::DnsEntryDataUnknown);
    check!("DnsEntryDataText", responses::DnsEntryDataText);
    check!(
        "DnsEntryDataNextResolver",
        responses::DnsEntryDataNextResolver
    );
    check!("DnsEntryDataSmcAddress", responses::DnsEntryDataSmcAddress);
    check!(
        "DnsEntryDataAdnlAddress",
        responses::DnsEntryDataAdnlAddress
    );
    check!(
        "DnsEntryDataStorageAddress",
        responses::DnsEntryDataStorageAddress
    );
    check!("DnsEntryData", responses::DnsEntryData);
    check!("DnsEntry", responses::DnsEntry);
    check!("DnsResolved", responses::DnsResolved);
    check!("AccountStateEnum", responses::AccountStateEnum);
    check!("BlockSignature", responses::BlockSignature);
    check!("ShardBlockLink", responses::ShardBlockLink);
    check!("BlockLinkBack", responses::BlockLinkBack);
    check!("OutMsgQueueSize", responses::OutMsgQueueSize);
    check!("LibraryEntry", responses::LibraryEntry);
    check!("ShortTxId", responses::ShortTxId);
    check!("MsgDataRaw", responses::MsgDataRaw);
    check!("MsgDataText", responses::MsgDataText);
    check!("MsgDataDecryptedText", responses::MsgDataDecryptedText);
    check!("MsgDataEncryptedText", responses::MsgDataEncryptedText);
    check!("MsgData", responses::MsgData);
    check!("MessageStd", responses::MessageStd);
    check!("TransactionStd", responses::TransactionStd);
    check!("TransactionExt", responses::TransactionExt);
    check!("Message", responses::Message);
    check!("Transaction", responses::Transaction);
    check!("DetectAddress", responses::DetectAddress);
    check!("DetectHash", responses::DetectHash);
    check!("PackAddress", responses::PackAddress);
    check!("UnpackAddress", responses::UnpackAddress);
    check!("AddressInformation", responses::AddressInformation);
    check!(
        "ExtendedAddressInformation",
        responses::ExtendedAddressInformation
    );
    check!("WalletInformation", responses::WalletInformation);
    check!("AddressBalance", responses::AddressBalance);
    check!("AddressState", responses::AddressState);
    check!("MasterchainInfo", responses::MasterchainInfo);
    check!("BlockSignatures", responses::BlockSignatures);
    check!("BlockSignaturesSimplex", responses::BlockSignaturesSimplex);
    check!(
        "MasterchainBlockSignatures",
        responses::MasterchainBlockSignatures
    );
    check!("ShardBlockProof", responses::ShardBlockProof);
    check!("ConsensusBlock", responses::ConsensusBlock);
    check!("LookupBlock", responses::LookupBlock);
    check!("Shards", responses::Shards);
    check!("BlockData", responses::BlockData);
    check!("BlockHeader", responses::BlockHeader);
    check!("OutMsgQueueSizes", responses::OutMsgQueueSizes);
    check!("ConfigInfo", responses::ConfigInfo);
    check!("LibraryResult", responses::LibraryResult);
    check!("BlockTransactions", responses::BlockTransactions);
    check!("BlockTransactionsExt", responses::BlockTransactionsExt);
    check!("Transactions", responses::Transactions);
    check!("TransactionsStd", responses::TransactionsStd);
    check!("ExtMessageInfo", responses::ExtMessageInfo);
    check!("ResultOk", responses::ResultOk);
    check!("Fees", responses::Fees);
    check!("QueryFees", responses::QueryFees);
    check!("TvmStackEntrySlice", stack::TvmStackEntrySlice);
    check!("TvmStackEntryCell", stack::TvmStackEntryCell);
    check!("TvmStackEntryNumber", stack::TvmStackEntryNumber);
    check!("TvmStackEntryTuple", stack::TvmStackEntryTuple);
    check!("TvmStackEntryList", stack::TvmStackEntryList);
    check!("TvmStackEntryUnsupported", stack::TvmStackEntryUnsupported);
    check!("TvmSlice", stack::TvmSlice);
    check!("TvmCell", stack::TvmCell);
    check!("TvmNumberDecimal", stack::TvmNumberDecimal);
    check!("TvmTuple", stack::TvmTuple);
    check!("TvmList", stack::TvmList);
    check!("RunGetMethodStdRequest", requests::RunGetMethodStdRequest);
    check!("RunGetMethodStdResult", responses::RunGetMethodStdResult);
    check!("LegacyTvmCell", stack::LegacyTvmCell);
    check!("LegacyStackEntryCell", stack::LegacyStackEntryCell);
    check!("RunGetMethodRequest", requests::RunGetMethodRequest);
    check!("RunGetMethodResult", responses::RunGetMethodResult);
    check!("LegacyTvmCellData", stack::LegacyTvmCellData);
    check!(
        "WalletInformationWalletType",
        responses::WalletInformationWalletType
    );
    check!("DetectAddressGivenType", responses::DetectAddressGivenType);
    check!("NftItemDataContent", responses::NftItemDataContent);
    check!(
        "NftItemDataContractType",
        responses::NftItemDataContractType
    );
    check!(
        "NftCollectionDataContractType",
        responses::NftCollectionDataContractType
    );
    check!(
        "JettonWalletDataContractType",
        responses::JettonWalletDataContractType
    );
    check!(
        "JettonMasterDataContractType",
        responses::JettonMasterDataContractType
    );
    check!("TokenContentData", responses::TokenContentData);
    check!("TokenContentType", responses::TokenContentType);
    check!("LegacyStackEntry", stack::LegacyStackEntry);
    check!("TvmStackEntry", stack::TvmStackEntry);
    check!("JsonRpcRequest", requests::JsonRpcRequest);
    check!(
        "SuccessEnvelope",
        TonlibResponse<responses::MasterchainInfo>
    );
    check!("ErrorEnvelope", TonlibErrorResponse);

    expect_test::expect_file!["fixtures/wire.json"]
        .assert_eq(&(serde_json::to_string_pretty(&actual).expect("snapshot JSON") + "\n"));
}

#[cfg(feature = "openapi")]
fn validate_schema<T: utoipa::ToSchema>(value: &Value) {
    let document = toncenter::v2::openapi::document();
    let mut schema = serde_json::to_value(T::schema()).expect("schema JSON");
    schema["components"] = serde_json::to_value(document.components).expect("components JSON");
    let validator = jsonschema::validator_for(&schema).expect("valid JSON schema");
    let errors = validator
        .iter_errors(value)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("{}: {errors:#?}", std::any::type_name::<T>());
    }
}

#[test]
fn malformed_wire_values_are_rejected() {
    let cases = json!({
        "wrong_object_tag": serde_json::from_value::<responses::AccountAddress>(json!({"@type":"wrong", "account_address":"0:00"})).is_err(),
        "missing_object_tag": serde_json::from_value::<responses::AccountAddress>(json!({"account_address":"0:00"})).is_err(),
        "unknown_stack_tag": serde_json::from_value::<stack::TvmStackEntry>(json!({"@type":"unknown"})).is_err(),
        "wrong_nested_stack_tag": serde_json::from_value::<stack::TvmStackEntry>(json!({"@type":"tvm.stackEntryNumber", "number":{"@type":"tvm.cell", "number":"123"}})).is_err(),
        "short_stack_pair": serde_json::from_value::<stack::LegacyStackEntry>(json!(["num"])).is_err(),
        "long_stack_pair": serde_json::from_value::<stack::LegacyStackEntry>(json!(["num","0x1",false])).is_err(),
        "wrong_stack_payload": serde_json::from_value::<stack::LegacyStackEntry>(json!(["cell",123])).is_err(),
        "wrong_success_flag": serde_json::from_value::<TonlibResponse<String>>(json!({"ok":false,"result":"1","@extra":""})).is_err(),
        "wrong_error_flag": serde_json::from_value::<TonlibErrorResponse>(json!({"ok":true,"error":"bad","code":400})).is_err(),
        "unknown_request_field": serde_json::from_value::<requests::AddressRequest>(json!({"address":"0:00","typo":1})).is_err(),
        "missing_required_request_field": serde_json::from_value::<requests::AddressRequest>(json!({})).is_err(),
        "floating_point_integer": serde_json::from_value::<toncenter::v2::Int64Input>(json!(1.5)).is_err(),
        "overflow_i32": serde_json::from_value::<toncenter::v2::Int32Input>(json!(2147483648_i64)).is_err(),
    });
    expect_test::expect_file!["fixtures/rejections.json"]
        .assert_eq(&(serde_json::to_string_pretty(&cases).expect("snapshot") + "\n"));
}
