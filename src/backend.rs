use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BackendServer {
    pub address: String,
    pub healthy: bool,
    pub connections: u32,
    pub weight: u32,
    pub last_health_check: Option<Instant>,
    pub response_time: Duration,
}

impl BackendServer {
    pub fn new(address: String, weight: u32) -> Self {
        Self { 
            address, 
            healthy: true, 
            connections: 0, 
            weight, 
            last_health_check: None, 
            response_time: Duration::from_millis(0) 
        }
    }
}