# speedtest-cli

A fast and lightweight command-line tool for testing internet connection speeds using Cloudflare's network.

## Features

- **Speed Testing**

  - Download speed measurement
  - Upload speed measurement
  - Latency (ping) testing
  - Network jitter measurement
  - Configurable file sizes for testing
  - Adjustable timeout settings
  - Multiple server testing
  - Network interface selection

- **Data Management**
  - Historical data tracking
  - Multiple output formats supported:
    - Human-readable text
    - JSON
    - YAML
    - CSV
  - Output to console or file
  - Export to Clickhouse for time-series analysis
  - Verbose mode for detailed logging

### Clickhouse schema

```sql
CREATE TABLE internet_speed
(
    id UUID DEFAULT generateUUIDv4(),
    timestamp DateTime DEFAULT now(),
    download_speed_mbps Float32,
    upload_speed_mbps Float32,
    ping_ms Float32,
    server_id String,
    jitter_ms Float32
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (timestamp, id)
SETTINGS index_granularity = 8192;
```

## Installation

### From releases

```sh
curl -sL https://raw.githubusercontent.com/psucodervn/speedtest-cli/master/scripts/install-latest.sh | sudo sh
```

### From Source

```bash
git clone https://github.com/psucodervn/speedtest-cli
cd speedtest-cli
cargo install --path .
```

## Usage

Basic usage:

```bash
speedtest-cli
```

### Options

```bash
Options:
  -v, --verbose                 Show detailed information
  -f, --format <FORMAT>         Output format (text, json, yaml, csv) [default: text]
  -o, --output <FILE>          Output file path
      --download-size <SIZE>    Download file size in MB [default: 100]
      --upload-size <SIZE>      Upload file size in MB [default: 20]
      --timeout <SECONDS>       Timeout in seconds [default: 30]
  -i, --interface <INTERFACE>   Network interface to use (e.g., eth0, wlan0)
  -n, --iterations <NUMBER>     Number of test iterations [default: 1]
      --history                 Enable historical data tracking
      --clickhouse-url <URL>    Clickhouse URL for result export
      --clickhouse-db <DB>      Clickhouse database name
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

1. Basic speed test with default settings:

```bash
speedtest-cli
```

2. Test specific network interface with multiple iterations:

```bash
speedtest-cli --interface eth0 --iterations 3
```

3. Enable historical tracking and export to Clickhouse:

```bash
speedtest-cli --history --clickhouse-url http://localhost:8123 --clickhouse-db speedtest
```

4. Custom test sizes with verbose output and jitter measurement:

```bash
speedtest-cli --verbose --download-size 200 --upload-size 50
```

5. Save results to JSON file with all metrics:

```bash
speedtest-cli --format json --output results.json
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
