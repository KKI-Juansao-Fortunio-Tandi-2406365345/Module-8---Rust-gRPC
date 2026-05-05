pub mod services {
    tonic::include_proto!("services");
}

use services::{
    payment_service_client::PaymentServiceClient,
    transaction_service_client::TransactionServiceClient,
    PaymentRequest, TransactionRequest,
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

    Ok(())
}
