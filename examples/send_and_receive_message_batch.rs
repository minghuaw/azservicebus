use azservicebus::{Client, ClientOptions, CreateMessageBatchOptions, Message, Receiver, Sender};

async fn send_batch(mut sender: Sender) -> Result<(), anyhow::Error> {
    let mut batch = sender.create_message_batch(CreateMessageBatchOptions::default())?;

    // Add messages to the batch
    // The three lines below are all equivalent
    batch.try_add_message("Message 1")?;
    batch.try_add_message(Message::new("Message 2"))?;
    batch.try_add_message(Message::from("Message 3"))?;

    // Send the batch
    sender.send_message_batch(batch).await?;

    sender.dispose().await?;
    Ok(())
}

async fn receive_messages(mut receiver: Receiver) -> Result<(), anyhow::Error> {
    // This will wait indefinitely until at least one message is received
    let received = receiver.receive_messages(3).await?;
    for message in received {
        receiver.complete_message(&message).await?;
    }
    receiver.dispose().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    // The connection string should look like:
    // "Endpoint=sb://<your-namespace>.servicebus.windows.net/;SharedAccessKeyName=<your-policy>;SharedAccessKey=<your-key>"
    let connection_string = std::env::var("SERVICE_BUS_CONNECTION_STRING")?;
    let queue_name = std::env::var("SERVICE_BUS_QUEUE")?;

    let mut client =
        Client::new_from_connection_string(connection_string, ClientOptions::default()).await?;

    // Create a sender and then send a batch of messages
    let sender = client
        .create_sender(&queue_name, Default::default())
        .await?;
    let sender_handle = tokio::spawn(send_batch(sender));

    // Create a receiver and then receive the messages
    let receiver = client
        .create_receiver_for_queue(queue_name, Default::default())
        .await?;
    let receiver_handle = tokio::spawn(receive_messages(receiver));

    sender_handle.await??;
    receiver_handle.await??;
    client.dispose().await?;
    Ok(())
}
