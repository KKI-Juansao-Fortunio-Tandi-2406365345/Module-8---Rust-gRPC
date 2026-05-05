use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub mod services {
    tonic::include_proto!("services");
}

use services::{
    payment_service_server::{PaymentService, PaymentServiceServer},
    transaction_service_server::{TransactionService, TransactionServiceServer},
    chat_service_server::{ChatService, ChatServiceServer},
    PaymentRequest, PaymentResponse,
    TransactionRequest, TransactionResponse,
    ChatMessage,
};

// ─── Payment Service (Unary) ───────────────────────────────────────────────

#[derive(Default)]
pub struct MyPaymentService {}

#[tonic::async_trait]
impl PaymentService for MyPaymentService {
    async fn process_payment(
        &self,
        request: Request<PaymentRequest>,
    ) -> Result<Response<PaymentResponse>, Status> {
        println!("Got payment request: {:?}", request);

        let req = request.into_inner();
        let response = PaymentResponse {
            payment_id: format!("PAY-{}", get_millis()),
            status: "SUCCESS".to_string(),
            message: format!(
                "Payment of {} processed for user {} using {}",
                req.amount, req.user_id, req.payment_method
            ),
        };

        Ok(Response::new(response))
    }
}

// ─── Transaction Service (Server Streaming) ────────────────────────────────

#[derive(Default)]
pub struct MyTransactionService {}

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    type GetTransactionHistoryStream = ReceiverStream<Result<TransactionResponse, Status>>;

    async fn get_transaction_history(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<Self::GetTransactionHistoryStream>, Status> {
        println!("Got transaction history request: {:?}", request);

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for i in 1..=30 {
                let transaction = TransactionResponse {
                    transaction_id: format!("TXN-{:05}", i),
                    user_id: request.get_ref().user_id.clone(),
                    amount: (i as f64) * 10.0,
                    description: format!("Transaction #{}", i),
                };
                tx.send(Ok(transaction)).await.unwrap();

                if i % 10 == 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

// ─── Chat Service (Bi-directional Streaming) ───────────────────────────────

#[derive(Default)]
pub struct MyChatService {}

#[tonic::async_trait]
impl ChatService for MyChatService {
    type ChatStream = ReceiverStream<Result<ChatMessage, Status>>;

    async fn chat(
        &self,
        request: Request<tonic::Streaming<ChatMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        println!("Chat session started");

        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(message) = stream.message().await.unwrap_or_else(|_| None) {
                println!("Received from {}: {}", message.user_id, message.message);

                let reply = ChatMessage {
                    user_id: "server".to_string(),
                    message: format!("Server received: {}", message.message),
                };

                tx.send(Ok(reply)).await.unwrap_or_else(|_| {});
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn get_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

// ─── Main ──────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    println!("gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(PaymentServiceServer::new(MyPaymentService::default()))
        .add_service(TransactionServiceServer::new(MyTransactionService::default()))
        .add_service(ChatServiceServer::new(MyChatService::default()))
        .serve(addr)
        .await?;

    Ok(())
}
