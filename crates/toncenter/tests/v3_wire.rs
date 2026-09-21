use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use toncenter::v3;

#[test]
fn untagged_values_select_variants_in_wire_order() {
    let integers: Vec<_> = [json!("001"), json!(-1), json!(0), json!(u64::MAX)]
        .into_iter()
        .map(|value| serde_json::from_value::<v3::StringOrNumber>(value).unwrap())
        .collect();
    let stacks: Vec<_> = [
        json!([]),
        json!([{ "type": "num", "value": "1" }]),
        json!([1]),
        json!(null),
    ]
    .into_iter()
    .map(|value| serde_json::from_value::<v3::StackValue>(value).unwrap())
    .collect();
    expect_test::expect![[r#"
        (
            [
                String(
                    "001",
                ),
                Number(
                    -1,
                ),
                Number(
                    0,
                ),
                Unsigned(
                    18446744073709551615,
                ),
            ],
            [
                Entries(
                    [],
                ),
                Entries(
                    [
                        StackEntity {
                            kind: "num",
                            value: Json(
                                String("1"),
                            ),
                        },
                    ],
                ),
                Json(
                    Array [
                        Number(1),
                    ],
                ),
                Json(
                    Null,
                ),
            ],
        )
    "#]]
    .assert_debug_eq(&(integers, stacks));
}

fn record<T: DeserializeOwned + Serialize>(output: &mut Value, fixtures: &Value, name: &str) {
    output[name] = fixtures[name]
        .as_array()
        .expect("fixture inputs are arrays")
        .iter()
        .map(|input| match serde_json::from_value::<T>(input.clone()) {
            Ok(value) => {
                json!({ "accepted": true, "output": serde_json::to_value(value).expect("serializable v3 model") })
            }
            Err(_) => json!({ "accepted": false }),
        })
        .collect();
}

#[test]
fn all_v3_models_preserve_the_wire_snapshot() {
    let fixtures: Value = serde_json::from_str(include_str!("fixtures/v3-input.json")).unwrap();
    let mut output = json!({});
    record::<v3::requests::MasterchainInfoQuery>(
        &mut output,
        &fixtures,
        "requests::MasterchainInfoQuery",
    );
    record::<v3::requests::MasterchainBlockShardStateQuery>(
        &mut output,
        &fixtures,
        "requests::MasterchainBlockShardStateQuery",
    );
    record::<v3::requests::MasterchainBlockShardsQuery>(
        &mut output,
        &fixtures,
        "requests::MasterchainBlockShardsQuery",
    );
    record::<v3::requests::AddressInformationQuery>(
        &mut output,
        &fixtures,
        "requests::AddressInformationQuery",
    );
    record::<v3::requests::AddressesQuery>(&mut output, &fixtures, "requests::AddressesQuery");
    record::<v3::requests::WalletInformationQuery>(
        &mut output,
        &fixtures,
        "requests::WalletInformationQuery",
    );
    record::<v3::requests::AccountStatesQuery>(
        &mut output,
        &fixtures,
        "requests::AccountStatesQuery",
    );
    record::<v3::requests::TracesQuery>(&mut output, &fixtures, "requests::TracesQuery");
    record::<v3::requests::PendingActionsQuery>(
        &mut output,
        &fixtures,
        "requests::PendingActionsQuery",
    );
    record::<v3::requests::PendingTracesQuery>(
        &mut output,
        &fixtures,
        "requests::PendingTracesQuery",
    );
    record::<v3::requests::DnsRecordsQuery>(&mut output, &fixtures, "requests::DnsRecordsQuery");
    record::<v3::requests::JettonBurnsQuery>(&mut output, &fixtures, "requests::JettonBurnsQuery");
    record::<v3::requests::JettonTransfersQuery>(
        &mut output,
        &fixtures,
        "requests::JettonTransfersQuery",
    );
    record::<v3::requests::NftCollectionsQuery>(
        &mut output,
        &fixtures,
        "requests::NftCollectionsQuery",
    );
    record::<v3::requests::NftSalesQuery>(&mut output, &fixtures, "requests::NftSalesQuery");
    record::<v3::requests::NftTransfersQuery>(
        &mut output,
        &fixtures,
        "requests::NftTransfersQuery",
    );
    record::<v3::requests::MultisigOrdersQuery>(
        &mut output,
        &fixtures,
        "requests::MultisigOrdersQuery",
    );
    record::<v3::requests::MultisigWalletsQuery>(
        &mut output,
        &fixtures,
        "requests::MultisigWalletsQuery",
    );
    record::<v3::requests::VestingQuery>(&mut output, &fixtures, "requests::VestingQuery");
    record::<v3::requests::TransactionsQuery>(
        &mut output,
        &fixtures,
        "requests::TransactionsQuery",
    );
    record::<v3::requests::BlocksQuery>(&mut output, &fixtures, "requests::BlocksQuery");
    record::<v3::requests::TransactionsByMessageQuery>(
        &mut output,
        &fixtures,
        "requests::TransactionsByMessageQuery",
    );
    record::<v3::requests::TransactionsByMasterchainBlockQuery>(
        &mut output,
        &fixtures,
        "requests::TransactionsByMasterchainBlockQuery",
    );
    record::<v3::requests::MessagesQuery>(&mut output, &fixtures, "requests::MessagesQuery");
    record::<v3::requests::AdjacentTransactionsQuery>(
        &mut output,
        &fixtures,
        "requests::AdjacentTransactionsQuery",
    );
    record::<v3::requests::WalletStatesQuery>(
        &mut output,
        &fixtures,
        "requests::WalletStatesQuery",
    );
    record::<v3::requests::TopAccountsByBalanceQuery>(
        &mut output,
        &fixtures,
        "requests::TopAccountsByBalanceQuery",
    );
    record::<v3::requests::EstimateFeeRequest>(
        &mut output,
        &fixtures,
        "requests::EstimateFeeRequest",
    );
    record::<v3::requests::PendingTransactionsQuery>(
        &mut output,
        &fixtures,
        "requests::PendingTransactionsQuery",
    );
    record::<v3::requests::JettonMastersQuery>(
        &mut output,
        &fixtures,
        "requests::JettonMastersQuery",
    );
    record::<v3::requests::JettonWalletsQuery>(
        &mut output,
        &fixtures,
        "requests::JettonWalletsQuery",
    );
    record::<v3::requests::NftItemsQuery>(&mut output, &fixtures, "requests::NftItemsQuery");
    record::<v3::requests::SendMessageRequest>(
        &mut output,
        &fixtures,
        "requests::SendMessageRequest",
    );
    record::<v3::requests::RunGetMethodRequest>(
        &mut output,
        &fixtures,
        "requests::RunGetMethodRequest",
    );
    record::<v3::requests::StackEntry>(&mut output, &fixtures, "requests::StackEntry");
    record::<v3::responses::AddressBookRow>(&mut output, &fixtures, "responses::AddressBookRow");
    record::<v3::responses::AddressMetadata>(&mut output, &fixtures, "responses::AddressMetadata");
    record::<v3::responses::TokenInfo>(&mut output, &fixtures, "responses::TokenInfo");
    record::<v3::responses::AccountStatesResponse>(
        &mut output,
        &fixtures,
        "responses::AccountStatesResponse",
    );
    record::<v3::responses::TracesResponse>(&mut output, &fixtures, "responses::TracesResponse");
    record::<v3::responses::ActionsResponse>(&mut output, &fixtures, "responses::ActionsResponse");
    record::<v3::responses::DnsRecord>(&mut output, &fixtures, "responses::DnsRecord");
    record::<v3::responses::DnsRecordsResponse>(
        &mut output,
        &fixtures,
        "responses::DnsRecordsResponse",
    );
    record::<v3::responses::JettonTransfer>(&mut output, &fixtures, "responses::JettonTransfer");
    record::<v3::responses::JettonTransfersResponse>(
        &mut output,
        &fixtures,
        "responses::JettonTransfersResponse",
    );
    record::<v3::responses::JettonBurn>(&mut output, &fixtures, "responses::JettonBurn");
    record::<v3::responses::JettonBurnsResponse>(
        &mut output,
        &fixtures,
        "responses::JettonBurnsResponse",
    );
    record::<v3::responses::NftCollection>(&mut output, &fixtures, "responses::NftCollection");
    record::<v3::responses::NftCollectionsResponse>(
        &mut output,
        &fixtures,
        "responses::NftCollectionsResponse",
    );
    record::<v3::responses::NftTransfer>(&mut output, &fixtures, "responses::NftTransfer");
    record::<v3::responses::NftTransfersResponse>(
        &mut output,
        &fixtures,
        "responses::NftTransfersResponse",
    );
    record::<v3::responses::NftSale>(&mut output, &fixtures, "responses::NftSale");
    record::<v3::responses::NftSalesResponse>(
        &mut output,
        &fixtures,
        "responses::NftSalesResponse",
    );
    record::<v3::responses::MultisigOrderAction>(
        &mut output,
        &fixtures,
        "responses::MultisigOrderAction",
    );
    record::<v3::responses::MultisigOrder>(&mut output, &fixtures, "responses::MultisigOrder");
    record::<v3::responses::Multisig>(&mut output, &fixtures, "responses::Multisig");
    record::<v3::responses::MultisigOrdersResponse>(
        &mut output,
        &fixtures,
        "responses::MultisigOrdersResponse",
    );
    record::<v3::responses::MultisigsResponse>(
        &mut output,
        &fixtures,
        "responses::MultisigsResponse",
    );
    record::<v3::responses::VestingInfo>(&mut output, &fixtures, "responses::VestingInfo");
    record::<v3::responses::VestingContractsResponse>(
        &mut output,
        &fixtures,
        "responses::VestingContractsResponse",
    );
    record::<v3::responses::TransactionsResponse>(
        &mut output,
        &fixtures,
        "responses::TransactionsResponse",
    );
    record::<v3::responses::MessagesResponse>(
        &mut output,
        &fixtures,
        "responses::MessagesResponse",
    );
    record::<v3::responses::WalletStatesResponse>(
        &mut output,
        &fixtures,
        "responses::WalletStatesResponse",
    );
    record::<v3::responses::WalletState>(&mut output, &fixtures, "responses::WalletState");
    record::<v3::responses::AccountBalance>(&mut output, &fixtures, "responses::AccountBalance");
    record::<v3::responses::EstimateFeeResult>(
        &mut output,
        &fixtures,
        "responses::EstimateFeeResult",
    );
    record::<v3::responses::EstimatedFee>(&mut output, &fixtures, "responses::EstimatedFee");
    record::<v3::responses::BlockId>(&mut output, &fixtures, "responses::BlockId");
    record::<v3::responses::Block>(&mut output, &fixtures, "responses::Block");
    record::<v3::responses::BlocksResponse>(&mut output, &fixtures, "responses::BlocksResponse");
    record::<v3::responses::MasterchainInfo>(&mut output, &fixtures, "responses::MasterchainInfo");
    record::<v3::responses::JettonMaster>(&mut output, &fixtures, "responses::JettonMaster");
    record::<v3::responses::JettonMastersResponse>(
        &mut output,
        &fixtures,
        "responses::JettonMastersResponse",
    );
    record::<v3::responses::JettonWalletsResponse>(
        &mut output,
        &fixtures,
        "responses::JettonWalletsResponse",
    );
    record::<v3::responses::NftItem>(&mut output, &fixtures, "responses::NftItem");
    record::<v3::responses::NftCollectionRef>(
        &mut output,
        &fixtures,
        "responses::NftCollectionRef",
    );
    record::<v3::responses::NftItemsResponse>(
        &mut output,
        &fixtures,
        "responses::NftItemsResponse",
    );
    record::<v3::responses::SendMessageResult>(
        &mut output,
        &fixtures,
        "responses::SendMessageResult",
    );
    record::<v3::responses::RunGetMethodResult>(
        &mut output,
        &fixtures,
        "responses::RunGetMethodResult",
    );
    record::<v3::responses::StackEntity>(&mut output, &fixtures, "responses::StackEntity");
    record::<v3::responses::V2AddressInformation>(
        &mut output,
        &fixtures,
        "responses::V2AddressInformation",
    );
    record::<v3::responses::V2WalletInformation>(
        &mut output,
        &fixtures,
        "responses::V2WalletInformation",
    );
    record::<v3::responses::RequestError>(&mut output, &fixtures, "responses::RequestError");
    record::<v3::responses::TraceNode>(&mut output, &fixtures, "responses::TraceNode");
    record::<v3::responses::Action>(&mut output, &fixtures, "responses::Action");
    record::<v3::responses::JettonWallet>(&mut output, &fixtures, "responses::JettonWallet");
    record::<v3::responses::AccountStateFull>(
        &mut output,
        &fixtures,
        "responses::AccountStateFull",
    );
    record::<v3::responses::Transaction>(&mut output, &fixtures, "responses::Transaction");
    record::<v3::responses::AccountState>(&mut output, &fixtures, "responses::AccountState");
    record::<v3::responses::TransactionDescr>(
        &mut output,
        &fixtures,
        "responses::TransactionDescr",
    );
    record::<v3::responses::CreditPhase>(&mut output, &fixtures, "responses::CreditPhase");
    record::<v3::responses::ComputePhase>(&mut output, &fixtures, "responses::ComputePhase");
    record::<v3::responses::ActionPhase>(&mut output, &fixtures, "responses::ActionPhase");
    record::<v3::responses::MsgSize>(&mut output, &fixtures, "responses::MsgSize");
    record::<v3::responses::StoragePhase>(&mut output, &fixtures, "responses::StoragePhase");
    record::<v3::responses::Message>(&mut output, &fixtures, "responses::Message");
    record::<v3::responses::MessageContent>(&mut output, &fixtures, "responses::MessageContent");
    record::<v3::responses::Trace>(&mut output, &fixtures, "responses::Trace");
    record::<v3::responses::TraceInfo>(&mut output, &fixtures, "responses::TraceInfo");
    record::<v3::responses::AddressBook>(&mut output, &fixtures, "responses::AddressBook");
    record::<v3::responses::Metadata>(&mut output, &fixtures, "responses::Metadata");
    record::<v3::responses::StackValue>(&mut output, &fixtures, "responses::StackValue");
    record::<v3::StringOrNumber>(&mut output, &fixtures, "StringOrNumber");
    expect_test::expect_file!["fixtures/v3-output.json"]
        .assert_eq(&(serde_json::to_string_pretty(&output).unwrap() + "\n"));
}
