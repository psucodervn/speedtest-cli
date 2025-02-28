use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = Client::new();
    
    println!("Starting speed test...");
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    // Test download speed using a large file
    pb.set_message("Testing download speed...");
    let download_speed = test_download(&client, &pb, cli.verbose).await;
    
    // Test upload speed
    pb.set_message("Testing upload speed...");
    let upload_speed = test_upload(&client, &pb, cli.verbose).await;
    
    // Test latency
    pb.set_message("Testing latency...");
    let ping = test_latency(&client, cli.verbose).await;

    pb.finish_and_clear();

    println!("\nResults:");
    println!("Download: {:.2} Mbps", download_speed);
    println!("Upload: {:.2} Mbps", upload_speed);
    println!("Ping: {:.0}ms", ping);
}

async fn test_download(client: &Client, pb: &ProgressBar, verbose: bool) -> f64 {
    // Using Cloudflare's speed test file (100MB)
    let url = "https://speed.cloudflare.com/__down?bytes=100000000";
    let start = Instant::now();
    
    match client.get(url).send().await {
        Ok(response) => {
            match response.bytes().await {
                Ok(bytes) => {
                    let duration = start.elapsed().as_secs_f64();
                    let bits = bytes.len() as f64 * 8.0;
                    bits / duration / 1_000_000.0 // Convert to Mbps
                }
                Err(e) => {
                    if verbose {
                        eprintln!("Error reading download response: {}", e);
                    }
                    0.0
                }
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("Error during download test: {}", e);
            }
            0.0
        }
    }
}

async fn test_upload(client: &Client, pb: &ProgressBar, verbose: bool) -> f64 {
    // Generate 10MB of random data
    let data = vec![0u8; 10_000_000];
    let start = Instant::now();
    
    match client.post("https://speed.cloudflare.com/__up")
        .body(data)
        .send()
        .await {
            Ok(_) => {
                let duration = start.elapsed().as_secs_f64();
                80.0 / duration // Convert to Mbps (10MB * 8 = 80Mb)
            }
            Err(e) => {
                if verbose {
                    eprintln!("Error during upload test: {}", e);
                }
                0.0
            }
        }
}

async fn test_latency(client: &Client, verbose: bool) -> f64 {
    let mut times = Vec::new();
    let test_url = "https://www.cloudflare.com";
    
    for i in 0..3 {
        let start = Instant::now();
        match client.get(test_url).send().await {
            Ok(_) => {
                times.push(start.elapsed().as_millis() as f64);
            }
            Err(e) => {
                if verbose {
                    eprintln!("Error during ping test #{}: {}", i + 1, e);
                }
            }
        }
    }
    
    if times.is_empty() {
        if verbose {
            eprintln!("All ping tests failed");
        }
        return 0.0;
    }
    
    times.iter().sum::<f64>() / times.len() as f64
}
