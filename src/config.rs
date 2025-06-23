use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author = "Parvat Raj Singh <parvat.raj2@gmail.com>", 
    version = "0.0.1", 
    about = "CLI to start the Load balancer see more in github README.md", 
    long_about= None
)]
pub struct Args {
    /// Port to listen on
    #[arg(short, long, default_value="8080")]
    pub port: u16,

    /// BE server (comma-seoarated)
    #[arg(short, long, default_value="127.0.0.1:3001,127.0.0.1:3002,127.0.0.1:3003")]
    pub backends: String,

    /// Load balancer algo (round_robin, leas_connections, random, weighted_round_robin)
    #[arg(short, long, default_value="round_robin")]
    pub algorithm: String,

    /// Health check interval in seconds
    #[arg(long, default_value="30")]
    pub health_check_interval: u64,
}