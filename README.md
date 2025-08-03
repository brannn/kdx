# kdx - K8s Discovery Experience

A command-line tool for exploring and discovering resources in Kubernetes clusters. Provides commands for listing services, pods, and understanding cluster topology and relationships.

## Features

- Service Discovery: List and explore services across namespaces
- Pod Exploration: Find pods with flexible filtering options
- Service Descriptions: Get detailed information about services and their relationships
- Topology Analysis: Understand service topology and backend connections
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
```

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

## Requirements

- Kubernetes cluster access
- Valid kubeconfig file
- kubectl installed and configured

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
