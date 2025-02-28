# speedtest-cli

A fast and lightweight command-line tool for testing internet connection speeds using Cloudflare's network.

## Features

- **Speed Testing**

  - Download speed measurement
  - Upload speed measurement
  - Latency (ping) testing
  - Configurable file sizes for testing
  - Adjustable timeout settings

- **Flexible Output**
  - Multiple output formats supported:
    - Human-readable text
    - JSON
    - YAML
    - CSV
  - Output to console or file
  - Verbose mode for detailed logging

## Installation

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
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

1. Basic speed test with default settings:

```bash
speedtest-cli
```

2. Output results in JSON format:

```bash
speedtest-cli --format json
```

3. Save results to a file:

```bash
speedtest-cli --output results.txt
```

4. Custom test sizes with verbose output:

```bash
speedtest-cli --verbose --download-size 200 --upload-size 50
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
