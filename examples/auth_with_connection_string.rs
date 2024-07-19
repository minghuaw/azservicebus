use azservicebus::{Client, ClientOptions, SenderOptions};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // The connection string should look like:
    // "Endpoint=sb://<your-namespace>.servicebus.windows.net/;SharedAccessKeyName=<your-policy>;SharedAccessKey=<your-key>"
    let connection_string = std::env::var("SERVICE_BUS_CONNECTION_STRING")?;
    let queue_name = std::env::var("SERVICE_BUS_QUEUE")?;

    let mut client =
        Client::new_from_connection_string(connection_string, ClientOptions::default()).await?;

    // Create a sender for auth purpose only
    let sender = client
        .create_sender(queue_name, SenderOptions::default())
        .await?;

    sender.dispose().await?;
    client.dispose().await?;
    Ok(())
}
