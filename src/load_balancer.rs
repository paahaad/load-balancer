use crate::algorithms::{select_backend, LoadBalancerAlgorithm};
use crate::backend::BackendServer;
use crate::health::perform_health_check;
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use std::sync::{Arc, Mutex};
use tracing::error;

#[derive(Debug)]
pub struct LoadBalancer {
    pub backends: Arc<Mutex<Vec<BackendServer>>>,
    pub algorithm: 
    LoadBalancerAlgorithm,
    pub current_index: Arc<Mutex<usize>>,
    pub client: Client<hyper_util::client::legacy::connect::HttpConnector, Incoming>,
    pub health_client: Client<hyper_util::client::legacy::connect::HttpConnector, Empty<Bytes>>,
}

impl LoadBalancer {
    pub fn new(backend_addresses: Vec<String>, algorithm: 
    LoadBalancerAlgorithm) -> Self {
        let backends = backend_addresses
            .into_iter()
            .map(|addr| BackendServer::new(addr, 1))
            .collect();

        Self {
            backends: Arc::new(Mutex::new(backends)),
            algorithm,
            current_index: Arc::new(Mutex::new(0)),
            client: Client::builder(TokioExecutor::new()).build_http(),
            health_client: Client::builder(TokioExecutor::new()).build_http(),
        }
    }

    pub fn select_backend(&self) -> Option<BackendServer> {
        select_backend(&self.backends, &self.algorithm, &self.current_index)
    }

    pub fn increment_connections(&self, address: &str) {
        let mut backends = self.backends.lock().unwrap();
        if let Some(backend) = backends.iter_mut().find(|b| b.address == address) {
            backend.connections += 1;
        }
    }

    pub fn decrement_connections(&self, address: &str) {
        let mut backends = self.backends.lock().unwrap();
        if let Some(backend) = backends.iter_mut().find(|b| b.address == address) {
            if backend.connections > 0 {
                backend.connections -= 1;
            }
        }
    }

    pub async fn health_check(&self) {
        perform_health_check(&self.backends, &self.health_client).await;
    }

    pub async fn forward_request(&self, mut req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let backend = match self.select_backend() {
            Some(backend) => backend,
            None => {
                let response = Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .body(Full::new(Bytes::from("No healthy backends available")))
                    .unwrap();
                return Ok(response);
            }
        };

        // Increment connection count
        self.increment_connections(&backend.address);

        // Modify the request URI to point to the selected backend
        let backend_uri = format!("http://{}{}", backend.address, req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("/"));
        *req.uri_mut() = backend_uri.parse().unwrap();

        // Add load balancer headers
        req.headers_mut().insert("X-Forwarded-By", "rust-load-balancer".parse().unwrap());
        req.headers_mut().insert("X-Backend-Server", backend.address.parse().unwrap());

        let result = self.client.request(req).await;

        // Decrement connection count
        self.decrement_connections(&backend.address);

        match result {
            Ok(response) => {
                // Convert the response body to our expected type
                let (parts, body) = response.into_parts();
                let body_bytes = http_body_util::BodyExt::collect(body).await?.to_bytes();
                let new_response = Response::from_parts(parts, Full::new(body_bytes));
                Ok(new_response)
            },
            Err(e) => {
                error!("Failed to forward request to backend {}: {}", backend.address, e);
                let response = Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(Full::new(Bytes::from("Backend server error")))
                    .unwrap();
                Ok(response)
            }
        }
    }
} 