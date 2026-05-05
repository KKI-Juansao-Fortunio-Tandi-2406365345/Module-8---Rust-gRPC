use tokio::io::{self, AsyncBufReadExt};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;

pub mod services {
    tonic::include_proto!("services");
}

use services::{
    payment_service_client::PaymentServiceClient,
    transaction_service_client::TransactionServiceClient,
    chat_service_client::ChatServiceClient,
    PaymentRequest, TransactionRequest, ChatMessage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ─── Payment Service (Unary) ──────────────────────────────────────────
    println!("=== Payment Service (Unary) ===");

    let mut payment_client =
        PaymentServiceClient::connect("http://[::1]:50051").await?;

    let payment_request = tonic::Request::new(PaymentRequest {
        user_id: "user_123".to_string(),
        amount: 250.00,
        payment_method: "credit_card".to_string(),
    });

    let payment_response = payment_client
        .process_payment(payment_request)
        .await?;

    println!("Payment Response: {:?}", payment_response.into_inner());

    // ─── Transaction Service (Server Streaming) ───────────────────────────
    println!("\n=== Transaction Service (Server Streaming) ===");

    let mut transaction_client =
        TransactionServiceClient::connect("http://[::1]:50051").await?;

    let transaction_request = tonic::Request::new(TransactionRequest {
        user_id: "user_123".to_string(),
    });

    let mut stream = transaction_client
        .get_transaction_history(transaction_request)
        .await?
        .into_inner();

    while let Some(transaction) = stream.message().await? {
        println!("Transaction: {:?}", transaction);
    }

    // ─── Chat Service (Bi-directional Streaming) ──────────────────────────
    println!("\n=== Chat Service (Bi-directional Streaming) ===");
    println!("Type messages below (Ctrl+C to quit):");

    let channel = Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;

    let mut client = ChatServiceClient::new(channel);

    let (tx, rx) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            let message = ChatMessage {
                user_id: "user_123".to_string(),
                message: line,
            };
            if tx.send(message).await.is_err() {
                eprintln!("Error sending message to server");
                break;
            }
        }
    });

    let request = tonic::Request::new(ReceiverStream::new(rx));
    let mut response_stream = client.chat(request).await?.into_inner();

    while let Some(response) = response_stream.message().await? {
        println!("Server: {:?}", response);
    }

    Ok(())
}
