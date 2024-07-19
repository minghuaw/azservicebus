use azservicebus::{Client, ClientOptions, Message};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // The connection string should look like:
    // "Endpoint=sb://<your-namespace>.servicebus.windows.net/;SharedAccessKeyName=<your-policy>;SharedAccessKey=<your-key>"
    let connection_string = std::env::var("SERVICE_BUS_CONNECTION_STRING")?;
    // The queue must have session enabled
    let queue_name = std::env::var("SERVICE_BUS_SESSION_QUEUE")?;
    let session_id = "session1";

    let mut client =
        Client::new_from_connection_string(connection_string, ClientOptions::default()).await?;

    // Create a sender and send a session message
    let mut sender = client
        .create_sender(&queue_name, Default::default())
        .await?;
    let mut message = Message::new("Hello World");
    message.set_session_id(String::from(session_id))?;
    sender.send_message(message).await?;
    sender.dispose().await?;

    // Create a receiver and receive the session message
    let mut receiver = client
        .accept_session_for_queue(queue_name, session_id, Default::default())
        .await?;
    let received = receiver.receive_message().await?;
    receiver.complete_message(&received).await?;
    receiver.dispose().await?;

    client.dispose().await?;
    Ok(())
}
