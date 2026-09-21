//! Fetch an account balance and state from TON Center mainnet.

use toncenter::v2::Response;
use toncenter::v2::requests::AddressInformationRequest;
use toncenter::v2::responses::AddressInformation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let request = AddressInformationRequest {
        address: "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c".to_owned(),
        seqno: None,
    };

    let response: Response<AddressInformation> = client
        .post("https://toncenter.com/api/v2/getAddressInformation")
        .json(&request)
        .send()?
        .json()?;
    let account = response.into_result()?;

    println!("Balance: {} nanograms", account.balance);
    println!("State: {:?}", account.state);
    Ok(())
}
