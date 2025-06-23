use crate::backend::BackendServer;
use hyper::Uri;
use hyper_util::client::legacy::Client;
use http_body_util::Empty;
use hyper::body::Bytes;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{error, info, warn};

pub async fn perform_health_check(
    backends: &Arc<Mutex<Vec<BackendServer>>>,
    client: &Client<hyper_util::client::legacy::connect::HttpConnector, Empty<Bytes>>,
) {
    let backend_addresses: Vec<String> = {
        let backends = backends.lock().unwrap();
        backends.iter().map(|b| b.address.clone()).collect()
    };
    
    let mut health_results = Vec::new();
    for address in &backend_addresses {
        let health_url = format!("http://{}/health", address);
        let start_time = Instant::now();
        
        let (healthy, response_time) = match client.get(health_url.parse::<Uri>().unwrap()).await {
            Ok(response) => {
                let response_time = start_time.elapsed();
                let healthy = response.status().is_success();
                
                if healthy {
                    info!("Backend {} is healthy (response time: {:?})", address, response_time);
                } else {
                    warn!("Backend {} returned unhealthy status: {}", address, response.status());
                }
                
                (healthy, response_time)
            }
            Err(e) => {
                error!("Health check failed for backend {}: {}", address, e);
                (false, start_time.elapsed())
            }
        };
        
        health_results.push((address.clone(), healthy, response_time));
    }
    
    let mut backends = backends.lock().unwrap();
    for (address, healthy, response_time) in health_results {
        if let Some(backend) = backends.iter_mut().find(|b| b.address == address) {
            backend.healthy = healthy;
            backend.response_time = response_time;
            backend.last_health_check = Some(Instant::now());
        }
    }
} 