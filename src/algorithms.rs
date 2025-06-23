use crate::backend::BackendServer;

use std::sync::{Arc, Mutex};
use rand::Rng;

#[derive(Debug, Clone)]
pub enum LoadBalancerAlgorithm {
    RoundRobin,
    LeastConnections,
    Random,
    WeightedRoundRobin,
}

impl From<&str> for LoadBalancerAlgorithm {
    fn from(s: &str) -> Self {
        match s {
            "least_connections" => LoadBalancerAlgorithm::LeastConnections,
            "random" => LoadBalancerAlgorithm::Random,
            "weighted_round_robin" => LoadBalancerAlgorithm::WeightedRoundRobin,
            _ => LoadBalancerAlgorithm::RoundRobin
        }
    }
}


pub fn select_backend(
    backends: &Arc<Mutex<Vec<BackendServer>>>,
    algorithm: &LoadBalancerAlgorithm,
    current_index: &Arc<Mutex<usize>>,
) -> Option<BackendServer> {

    let backends = backends.lock().unwrap();
    let healthy_backends: Vec<_> = backends.iter().filter(|b| b.healthy).collect();

    if healthy_backends.is_empty() {
        return None;
    }
    let selected = match algorithm {
        LoadBalancerAlgorithm::RoundRobin => {
            let mut index = current_index.lock().unwrap();
            let backend =  healthy_backends[*index % healthy_backends.len()].clone();
            *index = (*index + 1) % healthy_backends.len();
            backend
        }
        LoadBalancerAlgorithm::LeastConnections => {
            (*healthy_backends.iter().min_by_key(|b|b.connections).unwrap()).clone()
        },
        LoadBalancerAlgorithm::Random => {
            let mut rng = rand::rng();
            let index = rng.random_range(0..healthy_backends.len());
            healthy_backends[index].clone()
        },
        // TODO: Implement production grade weightedRoundRobin
        LoadBalancerAlgorithm::WeightedRoundRobin => {
            let total_weight: u32 = healthy_backends.iter().map(|b| b.weight).sum();
            let mut rng = rand::rng();
            let mut random_weight = rng.random_range(0..total_weight);
            for backend in &healthy_backends {
                if random_weight < backend.weight {
                    return Some((*backend).clone());
                }
                random_weight -= backend.weight
            }
            healthy_backends[0].clone()
        },
    };
    
    Some(selected)
    
}