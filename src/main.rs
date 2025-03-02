use clap::Parser;
use clickhouse::{Client, Row};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client as ReqwestClient;
use serde::Serialize;
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::{self};
use chrono::{DateTime, Utc};

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

    /// Network interface to use (e.g., eth0, wlan0)
    #[arg(short, long)]
    interface: Option<String>,

    /// Number of test iterations for multiple server testing
    #[arg(long, default_value = "1")]
    iterations: u32,

    /// Enable historical data tracking
    #[arg(long)]
    history: bool,

    /// Clickhouse URL for result export
    #[arg(long)]
    clickhouse_url: Option<String>,

    /// Clickhouse database name
    #[arg(long, default_value="default")]
    clickhouse_db: Option<String>,

    /// Clickhouse user
    #[arg(long)]
    clickhouse_user: Option<String>,

    /// Clickhouse password
    #[arg(long)]
    clickhouse_password: Option<String>,
}

#[derive(Serialize, Row)]
struct SpeedTestResult {
    timestamp: DateTime<Utc>,
    download_speed_mbps: f32,
    upload_speed_mbps: f32,
    ping_ms: f32,
    server_id: String,
    jitter_ms: f32,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = ReqwestClient::builder()
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

    pb.set_message("Testing jitter...");
    let jitter = test_jitter(&client, cli.verbose).await;

    pb.finish_and_clear();

    let result = SpeedTestResult {
        timestamp: Utc::now(),
        download_speed_mbps: download_speed as f32,
        upload_speed_mbps: upload_speed as f32,
        ping_ms: ping as f32,
        jitter_ms: jitter as f32,
        server_id: "cloudflare".to_string(),
    };

    // Export to Clickhouse if configured
    if let (Some(url), Some(db), Some(user), Some(password)) = (cli.clickhouse_url.as_ref(), cli.clickhouse_db.as_ref(), cli.clickhouse_user.as_ref(), cli.clickhouse_password.as_ref()) {
        if let Err(e) = export_to_clickhouse(&result, url, db, user, password).await {
            eprintln!("Failed to export to Clickhouse: {}", e);
        } else if cli.verbose {
            println!("Successfully exported results to Clickhouse");
        }
    }

    let output = match cli.format.as_str() {
        "json" => serde_json::to_string_pretty(&result).unwrap(),
        "yaml" => serde_yaml::to_string(&result).unwrap(),
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            wtr.serialize(&result).unwrap();
            String::from_utf8(wtr.into_inner().unwrap()).unwrap()
        }
        _ => format!(
            "Results:\nDownload: {:.2} Mbps\nUpload: {:.2} Mbps\nPing: {:.0}ms\nJitter: {:.2}ms",
            download_speed, upload_speed, ping, jitter
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

async fn export_to_clickhouse(
    result: &SpeedTestResult,
    url: &str,
    db: &str,
    user: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default()
        .with_url(url)
        .with_database(db)
        .with_user(user)
        .with_password(password);

    // Create table if it doesn't exist
    client
        .query(
            "CREATE TABLE IF NOT EXISTS internet_speed (
                id UUID DEFAULT generateUUIDv4(),
                timestamp DateTime DEFAULT now(),
                download_speed_mbps Float32,
                upload_speed_mbps Float32,
                ping_ms Float32,
                server_id String,
                jitter_ms Float32
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (timestamp, id)
            SETTINGS index_granularity = 8192"
        )
        .execute()
        .await?;

    // Insert the result
    let insert_query = format!(
        "INSERT INTO internet_speed (
            timestamp, download_speed_mbps, upload_speed_mbps, ping_ms, server_id, jitter_ms
        ) VALUES (
            '{}', {}, {}, {}, '{}', {}
        )",
        result.timestamp.format("%Y-%m-%d %H:%M:%S"),
        result.download_speed_mbps,
        result.upload_speed_mbps,
        result.ping_ms,
        result.server_id,
        result.jitter_ms
    );

    client.query(&insert_query).execute().await?;

    Ok(())
}

async fn test_download(client: &ReqwestClient, _pb: &ProgressBar, verbose: bool, size: u32) -> f64 {
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

async fn test_upload(client: &ReqwestClient, _pb: &ProgressBar, verbose: bool, size: u32) -> f64 {
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

async fn test_latency(client: &ReqwestClient, verbose: bool) -> f64 {
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

async fn test_jitter(client: &ReqwestClient, verbose: bool) -> f64 {
    let mut jitter_samples = Vec::new();
    let num_samples = 10;

    for _ in 0..num_samples {
        let start = Instant::now();
        let _ = client
            .get("https://1.1.1.1/cdn-cgi/trace")
            .send()
            .await
            .unwrap();
        let duration = start.elapsed().as_secs_f64() * 1000.0;
        jitter_samples.push(duration);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Calculate jitter as the average deviation between consecutive samples
    let mut total_jitter = 0.0;
    for i in 1..jitter_samples.len() {
        total_jitter += (jitter_samples[i] - jitter_samples[i - 1]).abs();
    }
    let avg_jitter = total_jitter / (jitter_samples.len() - 1) as f64;

    if verbose {
        println!("Jitter: {:.2} ms", avg_jitter);
    }

    avg_jitter
}
