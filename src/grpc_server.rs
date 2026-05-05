use tonic::transport::Server;

pub mod services {
    tonic::include_proto!("services");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("gRPC Server skeleton - services will be added in next steps");
    let _server = Server::builder();
    Ok(())
}
