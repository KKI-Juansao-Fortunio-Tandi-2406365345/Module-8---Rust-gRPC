use tonic::{transport::Server, Request, Response, Status};

pub mod services {
    tonic::include_proto!("services");
}

use services::{
    payment_service_server::{PaymentService, PaymentServiceServer},
    PaymentRequest, PaymentResponse,
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
            payment_id: format!("PAY-{}", chrono_millis()),
            status: "SUCCESS".to_string(),
            message: format!(
                "Payment of {} processed for user {} using {}",
                req.amount, req.user_id, req.payment_method
            ),
        };

        Ok(Response::new(response))
    }
}

fn chrono_millis() -> u128 {
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
        .serve(addr)
        .await?;

    Ok(())
}
