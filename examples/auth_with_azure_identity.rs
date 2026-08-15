use std::sync::Arc;

use azservicebus::{ServiceBusClient, ServiceBusClientOptions, ServiceBusSenderOptions};
use azure_core::credentials::TokenCredential;
use azure_identity::{DeveloperToolsCredential, ManagedIdentityCredential};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = dotenv::from_filename(".env");

    // The namespace should look like: "<your-namespace>.servicebus.windows.net"
    let namespace = std::env::var("SERVICE_BUS_NAMESPACE")?;
    let queue_name = std::env::var("SERVICE_BUS_QUEUE")?;

    // `azure_identity` 1.0 removed `DefaultAzureCredential`. Pick the credential that matches
    // the environment instead: a managed identity when deployed to Azure, developer tools
    // (Azure CLI / Azure Developer CLI) when running locally.
    let credential: Arc<dyn TokenCredential> = if std::env::var("AZURE_CLIENT_ID").is_ok() {
        ManagedIdentityCredential::new(None)?
    } else {
        DeveloperToolsCredential::new(None)?
    };

    let mut client = ServiceBusClient::new_from_token_credential(
        namespace,
        credential,
        ServiceBusClientOptions::default(),
    )
    .await?;

    // Create a sender for auth purpose only
    let sender = client
        .create_sender(queue_name, ServiceBusSenderOptions::default())
        .await?;

    sender.dispose().await?;
    client.dispose().await?;
    Ok(())
}
