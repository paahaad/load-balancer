# Load Balancer

A high-performance HTTP load balancer implemented in Rust with multiple load balancing algorithms, health checking, and real-time monitoring.

## 🎯 **Features Implemented**

### **Core Functionality**
- **HTTP Request Forwarding**: Full HTTP request/response proxying
- **Multiple Load Balancing Algorithms**:
  - Round Robin (default)
  - Least Connections
  - Random
  - Weighted Round Robin
- **Health Checking**: Automatic backend monitoring with configurable intervals
- **Connection Tracking**: Real-time active connection counting

### **Advanced Features**
- **Monitoring Endpoints**:
  - `/lb-health` - Load balancer health status
  - `/lb-stats` - Detailed statistics and metrics
- **Custom Headers**: Adds `X-Forwarded-By` and `X-Backend-Server` headers
- **Command-Line Interface**: Full CLI with help and configuration options
- **Structured Logging**: Comprehensive logging with tracing
- **Error Handling**: Graceful handling of backend failures

### **Performance & Reliability**
- **Async/Await**: High-performance concurrent request handling
- **Thread-Safe**: Safe concurrent access to shared state
- **Automatic Failover**: Removes unhealthy backends from rotation
- **Auto-Recovery**: Re-enables backends when they become healthy

## Quick Start

### 1. Build the project

```bash
cargo build --release
```

### 2. Start test backend servers

```bash
# Terminal 1 - Backend server on port 3001
PORT=3001 cargo run --bin test_server

# Terminal 2 - Backend server on port 3002  
PORT=3002 cargo run --bin test_server

# Terminal 3 - Backend server on port 3003
PORT=3003 cargo run --bin test_server
```

### 3. Start the load balancer

```bash
# Terminal 4 - Load balancer on port 8080
cargo run --bin lb
```

### 4. Test the load balancer

```bash
# Send requests to see load balancing in action
curl http://localhost:8080/

# Check load balancer health
curl http://localhost:8080/lb-health

# View statistics
curl http://localhost:8080/lb-stats
```

## Configuration

### Command Line Options

```bash
cargo run --bin lb -- --help
```

```
Options:
  -p, --port <PORT>                        Port to listen on [default: 8080]
  -b, --backends <BACKENDS>                Backend server addresses (comma-separated) [default: 127.0.0.1:3001,127.0.0.1:3002,127.0.0.1:3003]
  -a, --algorithm <ALGORITHM>              Load balancing algorithm [default: round_robin]
      --health-check-interval <HEALTH_CHECK_INTERVAL>  Health check interval in seconds [default: 30]
  -h, --help                               Print help
  -V, --version                            Print version
```

### Examples

```bash
# Use least connections algorithm
cargo run --bin lb -- --algorithm least_connections

# Custom backends and port
cargo run --bin lb -- --port 9000 --backends "192.168.1.10:8001,192.168.1.11:8001,192.168.1.12:8001"

# Random algorithm with frequent health checks
cargo run --bin lb -- --algorithm random --health-check-interval 10
```

## Load Balancing Algorithms

### 1. Round Robin (round_robin)
Distributes requests evenly across all healthy backends in sequential order.

### 2. Least Connections (least_connections)
Routes requests to the backend server with the fewest active connections.

### 3. Random (random)
Randomly selects a healthy backend server for each request.

### 4. Weighted Round Robin (weighted_round_robin)
Similar to round robin but considers server weights (currently all servers have equal weight).

## Monitoring Endpoints

### Health Check: `/lb-health`
Returns the health status of the load balancer and all backend servers.

```json
{
  "status": "ok",
  "healthy_backends": 3,
  "total_backends": 3,
  "backends": [
    {
      "address": "127.0.0.1:3001",
      "healthy": true,
      "connections": 0,
      "response_time_ms": 1
    }
  ]
}
```

### Statistics: `/lb-stats`
Provides detailed statistics about the load balancer and backend servers.

```json
{
  "algorithm": "RoundRobin",
  "backends": [
    {
      "address": "127.0.0.1:3001",
      "healthy": true,
      "connections": 2,
      "weight": 1,
      "response_time_ms": 5,
      "last_health_check": 15
    }
  ]
}
```

## Request Headers

The load balancer adds the following headers to forwarded requests:

- `X-Forwarded-By: rust-load-balancer` - Identifies the load balancer
- `X-Backend-Server: <backend_address>` - Shows which backend handled the request

## Health Checking

- Health checks are performed on the `/health` endpoint of each backend server
- Configurable interval (default: 30 seconds)
- Automatic failover when backends become unhealthy
- Backends are automatically re-enabled when they recover

## Backend Server Requirements

Backend servers should:

1. **Health Endpoint:** Respond to `GET /health` with HTTP 200 for healthy status
2. **HTTP Protocol:** Support standard HTTP requests and responses
3. **Port Binding:** Listen on the configured IP address and port

## 🧪 **Test Infrastructure**
- **Test Server**: Simple backend server for testing (`test_server.rs`)
- **Multiple Binaries**: Separate binaries for load balancer and test server
- **JSON Responses**: Structured responses for easy testing

## Development

### Testing

```bash
# Run tests
cargo test

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

### Architecture

The load balancer is built using:

- **Hyper:** High-performance HTTP server and client
- **Tokio:** Async runtime for concurrent request handling
- **Tracing:** Structured logging and diagnostics
- **Clap:** Command-line argument parsing
- **Serde:** JSON serialization for monitoring endpoints

## Performance

The load balancer is designed for high performance:

- Async/await for concurrent request handling
- Minimal memory allocation during request forwarding
- Lock-free operation for request routing (minimal mutex usage)
- Efficient health checking without blocking request processing

## 🚀 **Production Ready**

This load balancer is production-ready with enterprise-level features:

- **High Performance**: Built with Rust's zero-cost abstractions
- **Memory Safety**: Rust's ownership system prevents common bugs
- **Concurrent**: Async/await for handling thousands of concurrent requests
- **Observable**: Comprehensive logging and monitoring endpoints
- **Configurable**: CLI-based configuration for deployment flexibility
- **Resilient**: Automatic failover and recovery mechanisms