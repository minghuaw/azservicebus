use azservicebus::{Client, ClientOptions};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = dotenv::from_filename(".env");

    // The connection string should look like:
    // "Endpoint=sb://<your-namespace>.servicebus.windows.net/;SharedAccessKeyName=<your-policy>;SharedAccessKey=<your-key>"
    let connection_string = std::env::var("SERVICE_BUS_CONNECTION_STRING")?;
    let queue_name = std::env::var("SERVICE_BUS_QUEUE")?;

    let mut client =
        Client::new_from_connection_string(connection_string, ClientOptions::default()).await?;

    // Create a sender and send a message
    let mut sender = client
        .create_sender(&queue_name, Default::default())
        .await?;
    sender.send_message("Hello World").await?;
    sender.dispose().await?;

    // Create a receiver and receive a message
    let mut receiver = client
        .create_receiver_for_queue(queue_name, Default::default())
        .await?;
    let message = receiver.receive_message().await?;

    println!("Received message: {:?}", message);

    // Complete the message
    receiver.complete_message(&message).await?;

    receiver.dispose().await?;
    client.dispose().await?;
    Ok(())
}
