# Chain Ping

A command-line tool for benchmarking the latency of Ethereum RPC endpoints.

## Installation

Install directly from crates.io using Cargo:

```bash
cargo install chain-ping
```

## Usage

### Basic Check

Ping a single endpoint to check its latency:

```bash
chain-ping https://eth.llamarpc.com
```

### Comparison Benchmark

Compare multiple providers to find the fastest one. By default, it pings each endpoint 4 times.

```bash
chain-ping https://eth.llamarpc.com https://rpc.ankr.com/eth https://1rpc.io/eth
```

### Customizing the Test

Ping each endpoint 10 times with a strict 2-second timeout

```bash
chain-ping --pings 10 --timeout 2 https://eth.llamarpc.com https://rpc.ankr.com/eth 
```

### Scripting & Automation

Output the results as JSON for use in scripts:

```bash
chain-ping --format json https://eth.llamarpc.com > results.json
```

## Output Examples

### Table Output (Default)

```
+-------------------------+-----------+-------------+-------------+-------------+---------+--------------+------------+
| Endpoint                | Status    | Avg Latency | Min         | Max         | Success | Block Number | Last Error |
+=====================================================================================================================+
| https://eth.llamarpc... | SUCCESS | 145ms       | 140ms       | 152ms       | 4/4     | "0x16bb624   | -          |
| https://rpc.ankr.com... | FAILURE | -           | -           | -           | 0/4     | -            | JSON-RPC...|
+-------------------------+-----------+-------------+-------------+-------------+---------+--------------+------------+
```

### JSON Output

```json
[
  {
    "endpoint": "https://eth.llamarpc.com",
    "avg_latency_ms": 145,
    "min_latency_ms": 140,
    "max_latency_ms": 152,
    "status": "Success",
    "block_number": "0x16bb624",
    "success_count": 4,
    "ping_count": 4,
    "error_message": null
  }
]
```

## License

This project is licensed under the MIT License.