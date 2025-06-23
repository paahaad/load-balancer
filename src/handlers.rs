use std::{convert::Infallible, sync::Arc};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming}, Request, Response, StatusCode
};

use crate::load_balancer::LoadBalancer;

pub async fn handle_request(
    req: Request<Incoming>,
    load_balancer: Arc<LoadBalancer>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if req.uri().path() == "lb-health" {
        return Ok(handle_health_check(&load_balancer));
    }
    if req.uri().path() == "lb-stats" {
        return Ok(handle_stats(&load_balancer));
    }

    match load_balancer.forward_request(req).await {
        Ok(response) => Ok(response),
        Err(_) => {
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from("Internal Server Error")))
                .unwrap();
            Ok(response)
        }
    }
}

fn handle_health_check(loadbalancer: &LoadBalancer) -> Response<Full<Bytes>> {

    let backend = loadbalancer.backends.lock().unwrap();
    let healty_count = backend.iter().filter(|b|b.healthy).count();
    let total_count = backend.len();
    
    let status = if healty_count > 0 {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let body = serde_json::json!({
        "status": "ok",
        "healthy_backends": healty_count,
        "total_backend": total_count,
        "backends": backend.iter().map(|b| serde_json::json!({
            "address" : b.address,
            "healthy" : b.healthy,
            "connections": b.connections,
            "response_time_ms": b.response_time.as_millis()
        })).collect::<Vec<_>>()
    });

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap()
}

fn handle_stats(load_balancer: &LoadBalancer) -> Response<Full<Bytes>> {
    let backends = load_balancer.backends.lock().unwrap();
    let stats = serde_json::json!({
        "algorithm": format!("{:?}", load_balancer.algorithm),
        "backend": backends.iter().map(|b| serde_json::json!({
            "address" : b.address,
            "healthy": b.healthy,
            "connections": b.connections,
            "weight": b.weight,
            "response_time_ms": b.response_time.as_millis(),
            "last_health_check": b.last_health_check.map(|t| t.elapsed().as_secs())
        })).collect::<Vec<_>>()
    });
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(stats.to_string())))
        .unwrap()
}