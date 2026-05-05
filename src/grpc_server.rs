use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub mod services {
    tonic::include_proto!("services");
}

use services::{
    payment_service_server::{PaymentService, PaymentServiceServer},
    transaction_service_server::{TransactionService, TransactionServiceServer},
    PaymentRequest, PaymentResponse,
    TransactionRequest, TransactionResponse,
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

                // Every 10th record simulate processing delay
                if i % 10 == 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
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
        .serve(addr)
        .await?;

    Ok(())
}
