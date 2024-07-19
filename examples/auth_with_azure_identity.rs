use azservicebus::{Client, ClientOptions, SenderOptions};
use azure_identity::{DefaultAzureCredential, TokenCredentialOptions};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = dotenv::from_filename(".env");

    // The namespace should look like: "<your-namespace>.servicebus.windows.net"
    let namespace = std::env::var("SERVICE_BUS_NAMESPACE")?;
    let queue_name = std::env::var("SERVICE_BUS_QUEUE")?;

    let credential = DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap();
    let mut client = Client::new_from_credential(
        namespace,
        credential,
        ClientOptions::default(),
    )
    .await?;

    // Create a sender for auth purpose only
    let sender = client
        .create_sender(queue_name, SenderOptions::default())
        .await?;

    sender.dispose().await?;
    client.dispose().await?;
    Ok(())
}
