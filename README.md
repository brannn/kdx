# kdx - K8s Discovery Experience


[![Build Status](https://github.com/brannn/kdx/workflows/CI/badge.svg)](https://github.com/brannn/kdx/actions)
[![Release](https://github.com/brannn/kdx/workflows/Release/badge.svg)](https://github.com/brannn/kdx/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/kdx.svg)](https://crates.io/crates/kdx)
[![Downloads](https://img.shields.io/crates/d/kdx.svg)](https://crates.io/crates/kdx)
[![GitHub release](https://img.shields.io/github/release/brannn/kdx.svg)](https://github.com/brannn/kdx/releases)
[![GitHub issues](https://img.shields.io/github/issues/brannn/kdx.svg)](https://github.com/brannn/kdx/issues)
[![Homebrew](https://img.shields.io/badge/homebrew-available-brightgreen.svg)](https://github.com/brannn/homebrew-kdx)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/brannn/kdx)
[![Tests](https://img.shields.io/badge/tests-23%20passing-brightgreen.svg)](https://github.com/brannn/kdx/actions)
[![Code Coverage](https://img.shields.io/badge/coverage-core%20modules-green.svg)](https://github.com/brannn/kdx)
[![GitHub stars](https://img.shields.io/github/stars/brannn/kdx.svg)](https://github.com/brannn/kdx/stargazers)

A command-line tool for exploring and discovering resources in Kubernetes clusters. Provides commands for listing services, pods, and understanding cluster topology and relationships.

## Features

- Service Discovery: List and explore services across namespaces
- Pod Exploration: Find pods with flexible filtering options
- Service Descriptions: Get detailed information about services and their relationships
- Topology Analysis: Understand service topology and backend connections
- Graph Visualization: Generate service dependency graphs in DOT and SVG formats
- Multiple Output Formats: Table, JSON, and YAML output support
- Namespace Filtering: Work with specific namespaces or across all namespaces
- Ingress Discovery: Show which services are exposed via ingress
- Configuration Analysis: Discover ConfigMaps and Secrets used by services
- Health Checking: Test service accessibility

## Installation

### Homebrew (Recommended)

```bash
brew install brannn/kdx/kdx
```

### Download Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/brannn/kdx/releases).

### From Source

```bash
git clone https://github.com/brannn/kdx
cd kdx
cargo build --release
```

The binary will be available at `target/release/kdx`.
## Usage


## Quick Start

```bash
# List all services in current namespace
kdx services

# Explore a specific service
kdx describe my-service

# Visualize service dependencies
kdx graph

# Get help
kdx --help
```
### Basic Commands

List all services in the default namespace:
```bash
kdx services
```

List all pods in a specific namespace:
```bash
kdx pods -n monitoring
```

Get detailed information about a service:
```bash
kdx describe grafana -n monitoring
```

Show service topology:
```bash
kdx topology grafana -n monitoring

Generate a service dependency graph:
```bash
kdx graph -n monitoring
``````

### Output Formats

kdx supports multiple output formats:

```bash
# Human-readable table (default)
kdx services

# JSON output
kdx services --output json

# YAML output
kdx services --output yaml
```

### Namespace Options

```bash
# Specific namespace
kdx services -n kube-system

# All namespaces
kdx services --all-namespaces

# Use context default namespace
kdx services
```

## Examples

### Service Discovery

```bash
# List services with selector filtering
kdx services --selector app=nginx

# List services in JSON format
kdx services --output json -n production
```

### Pod Exploration

```bash
# List all pods
kdx pods

# Filter pods by selector
kdx pods --selector app=web

# Show pods across all namespaces
kdx pods --all-namespaces
```

### Service Analysis

```bash
# Get comprehensive service information
kdx describe api-service -n production

# Show service topology and relationships
kdx topology frontend -n web
```

### Graph Visualization

```bash
# Generate basic service dependency graph
kdx graph

# Graph for specific namespace
kdx graph -n production

# Include pod relationships
kdx graph --include-pods

# Highlight specific service
kdx graph --highlight=api-service

# Generate SVG format
kdx graph --format=svg

# Save graph to file
kdx graph -n monitoring > services.dot

# Convert to PNG (requires Graphviz)
kdx graph -n monitoring | dot -Tpng -o services.png

# Convert to SVG (requires Graphviz)
kdx graph -n monitoring | dot -Tsvg -o services.svg
```

## Example Output

### Service Listing

```bash
$ kdx services -n production
```

```
+-------------+-----------+-------------+------------+-------+-----+
| NAME        | NAMESPACE | TYPE        | CLUSTER-IP | PORTS | AGE |
+-------------+-----------+-------------+------------+-------+-----+
| api-gateway | production| LoadBalancer| 10.0.1.100 | 80    | 5d  |
| auth-service| production| ClusterIP   | 10.0.1.101 | 8080  | 5d  |
| database    | production| ClusterIP   | 10.0.1.102 | 5432  | 10d |
| redis-cache | production| ClusterIP   | 10.0.1.103 | 6379  | 8d  |
+-------------+-----------+-------------+------------+-------+-----+
```

### Service Description

```bash
$ kdx describe api-gateway -n production
```

```
Service: api-gateway
Namespace: production
Type: LoadBalancer
Cluster IP: 10.0.1.100
External IP: 203.0.113.10

Ports:
  http 80:TCP -> 8080 (TCP)
  https 443:TCP -> 8443 (TCP)

Selector:
  app = api-gateway
  version = v1.2.3

Related Pods:
+---------------------------+-----------+---------+-------+----------+-----+-------------+----------+
| NAME                      | NAMESPACE | STATUS  | READY | RESTARTS | AGE | IP          | NODE     |
+---------------------------+-----------+---------+-------+----------+-----+-------------+----------+
| api-gateway-7d4b8c9f-abc12| production| Running | 2/2   | 0        | 2d  | 10.244.1.15 | worker-1 |
| api-gateway-7d4b8c9f-def34| production| Running | 2/2   | 0        | 2d  | 10.244.2.20 | worker-2 |
+---------------------------+-----------+---------+-------+----------+-----+-------------+----------+

Ingress Routes:
  https://api.example.com -> api-gateway:80
  https://api.example.com/v2 -> api-gateway:80
```

### Graph Visualization

```bash
$ kdx graph -n production --highlight=api-gateway
```

```
digraph services {
  rankdir=LR;
  
  "api-gateway" [shape=box, style=filled, color=red, label="api-gateway\nproduction"];
  "auth-service" [shape=box, style=filled, color=lightblue, label="auth-service\nproduction"];
  "database" [shape=box, style=filled, color=lightblue, label="database\nproduction"];
  
  "ingress-nginx" [shape=diamond, style=filled, color=orange, label="ingress-nginx\ningress-nginx"];
  
  "ingress-nginx" -> "api-gateway" [style=bold, label="exposes"];
  "api-gateway" -> "auth-service" [style=dashed, label="depends on"];
  "auth-service" -> "database" [style=dashed, label="depends on"];
}
```

### JSON Output

```bash
$ kdx services -n production --output=json
```

```json
[
  {
    "name": "api-gateway",
    "namespace": "production",
    "type": "LoadBalancer",
    "cluster_ip": "10.0.1.100",
    "external_ips": ["203.0.113.10"],
    "ports": [
      {"name": "http", "port": 80, "target_port": "8080", "protocol": "TCP"},
      {"name": "https", "port": 443, "target_port": "8443", "protocol": "TCP"}
    ],
    "selector": {"app": "api-gateway", "version": "v1.2.3"},
    "age": "5d"
  }
]
```
## Requirements

- Kubernetes cluster access
- Valid kubeconfig file
- kubectl installed and configured
- Graphviz (optional, for converting DOT graphs to PNG/SVG)
## Configuration

kdx uses your existing kubectl configuration. Ensure you have:

1. A valid kubeconfig file (usually at `~/.kube/config`)
2. Appropriate cluster access permissions
3. kubectl configured for your target cluster

You can verify your configuration with:
```bash
kubectl cluster-info
```

## License

MIT License
