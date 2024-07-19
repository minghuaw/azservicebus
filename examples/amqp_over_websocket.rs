use azservicebus::{
    Client, ClientOptions, SenderOptions, TransportType,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // The connection string should look like:
    // "Endpoint=sb://<your-namespace>.servicebus.windows.net/;SharedAccessKeyName=<your-policy>;SharedAccessKey=<your-key>"
    let connection_string = std::env::var("SERVICE_BUS_CONNECTION_STRING")?;
    let queue_name = std::env::var("SERVICE_BUS_QUEUE")?;

    let options = ClientOptions {
        transport_type: TransportType::AmqpWebSocket,
        ..Default::default()
    };
    let mut client =
        Client::new_from_connection_string(connection_string, options).await?;

    // Create a sender for auth only
    let sender = client
        .create_sender(queue_name, SenderOptions::default())
        .await?;

    sender.dispose().await?;
    client.dispose().await?;
    Ok(())
}
