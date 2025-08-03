# kdx - K8s Discovery Experience


[![Build Status](https://github.com/brannn/kdx/workflows/CI/badge.svg)](https://github.com/brannn/kdx/actions)
[![Release](https://github.com/brannn/kdx/workflows/Release/badge.svg)](https://github.com/brannn/kdx/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/kdx.svg)](https://crates.io/crates/kdx)
[![Downloads](https://img.shields.io/crates/d/kdx.svg)](https://crates.io/crates/kdx)
[![GitHub release](https://img.shields.io/github/release/brannn/kdx.svg)](https://github.com/brannn/kdx/releases)
[![GitHub stars](https://img.shields.io/github/stars/brannn/kdx.svg)](https://github.com/brannn/kdx/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/brannn/kdx.svg)](https://github.com/brannn/kdx/issues)
[![Homebrew](https://img.shields.io/badge/homebrew-available-brightgreen.svg)](https://github.com/brannn/homebrew-kdx)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/brannn/kdx)
[![Tests](https://img.shields.io/badge/tests-23%20passing-brightgreen.svg)](https://github.com/brannn/kdx/actions)
[![Code Coverage](https://img.shields.io/badge/coverage-core%20modules-green.svg)](https://github.com/brannn/kdx)
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

### From Source

```bash
git clone https://github.com/brannn/kdx
cd kdx
cargo build --release
```

The binary will be available at `target/release/kdx`.

## Usage

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
