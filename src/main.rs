use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::Serialize;
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,

    /// Output format (text, json, yaml, csv)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Download file size in MB (default: 100)
    #[arg(long, default_value = "100")]
    download_size: u32,

    /// Upload file size in MB (default: 20)
    #[arg(long, default_value = "20")]
    upload_size: u32,

    /// Timeout in seconds (default: 30)
    #[arg(long, default_value = "30")]
    timeout: u64,
}

#[derive(Serialize)]
struct SpeedTestResult {
    download: f64,
    upload: f64,
    ping: f64,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = Client::builder()
        .timeout(Duration::from_secs(cli.timeout))
        .build()
        .unwrap();
    
    if cli.format == "text" && cli.output.is_none() {
        println!("Starting speed test...");
    }
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    pb.set_message("Testing download speed...");
    let download_speed = test_download(&client, &pb, cli.verbose, cli.download_size).await;
    
    pb.set_message("Testing upload speed...");
    let upload_speed = test_upload(&client, &pb, cli.verbose, cli.upload_size).await;
    
    pb.set_message("Testing latency...");
    let ping = test_latency(&client, cli.verbose).await;

    pb.finish_and_clear();

    let result = SpeedTestResult {
        download: download_speed,
        upload: upload_speed,
        ping,
    };

    let output = match cli.format.as_str() {
        "json" => serde_json::to_string_pretty(&result).unwrap(),
        "yaml" => serde_yaml::to_string(&result).unwrap(),
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            wtr.serialize(&result).unwrap();
            String::from_utf8(wtr.into_inner().unwrap()).unwrap()
        }
        _ => format!(
            "Results:\nDownload: {:.2} Mbps\nUpload: {:.2} Mbps\nPing: {:.0}ms",
            download_speed, upload_speed, ping
        ),
    };

    match cli.output {
        Some(path) => {
            let mut file = File::create(path).expect("Failed to create output file");
            file.write_all(output.as_bytes()).expect("Failed to write to file");
        }
        None => println!("{}", output),
    }
}

async fn test_download(client: &Client, _pb: &ProgressBar, verbose: bool, size: u32) -> f64 {
    let url = format!("https://speed.cloudflare.com/__down?bytes={}", size * 1_000_000);
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

async fn test_upload(client: &Client, _pb: &ProgressBar, verbose: bool, size: u32) -> f64 {
    let data = vec![0u8; (size * 1_000_000) as usize];
    let start = Instant::now();
    
    match client.post("https://speed.cloudflare.com/__up")
        .body(data)
        .send()
        .await {
            Ok(_) => {
                let duration = start.elapsed().as_secs_f64();
                (size as f64 * 8.0) / duration // Convert to Mbps
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
