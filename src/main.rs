use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time;
use tracing::info;
use clap::Parser;

use lb::algorithms::LoadBalancerAlgorithm;
use lb::config::Args;
use lb::handlers::handle_request;
use lb::load_balancer::LoadBalancer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    tracing_subscriber::fmt::init();

    let args = Args::parse();
    
    info!("Starting load balancer on port {}", args.port);
    info!("Load balancing algorithm: {}", args.algorithm);
    
    let backend_addresses: Vec<String> = args.backends
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    info!("Backend servers: {:?}", backend_addresses);
    
    let algorithm = LoadBalancerAlgorithm::from(args.algorithm.as_str());
    let lb = Arc::new(LoadBalancer::new(backend_addresses, algorithm));
    
    // Start health check task
    let health_check_lb = Arc::clone(&lb);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(args.health_check_interval));
        loop {
            interval.tick().await;
            health_check_lb.health_check().await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = TcpListener::bind(addr).await?;

    info!("Load balancer listening on http://{}", addr);
    info!("Health endpoint: http://{}/lb-health", addr);
    info!("Stats endpoint: http://{}/lb-stats", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let lb = Arc::clone(&lb);

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                handle_request(req, Arc::clone(&lb))
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                tracing::error!("Error serving connection: {:?}", err);
            }
        });
    }
}
