#[tokio::main]
async fn main() -> anyhow::Result<()> {
    faucet::run().await
}
