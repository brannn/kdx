# kdx - K8s Discovery Experience


[![Build Status](https://github.com/brannn/kdx/actions/workflows/ci.yml/badge.svg)](https://github.com/brannn/kdx/actions)
[![Release](https://img.shields.io/github/v/release/brannn/kdx?label=release)](https://github.com/brannn/kdx/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Homebrew](https://img.shields.io/badge/homebrew-available-brightgreen.svg)](https://github.com/brannn/homebrew-kdx)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/brannn/kdx)

A command-line tool for exploring and discovering resources in Kubernetes clusters. Provides commands for listing services, pods, and understanding cluster topology and relationships.

## Quick Demo

```console
$ kdx topology --namespace monitoring grafana

Service Topology: grafana
├── Namespace: monitoring
├── Type: LoadBalancer
├── Cluster IP: 10.43.132.227
└── Backend Pods:
    └── grafana-5df9c4787-fxl27 (Running)
```

```console
$ kdx graph --namespace production --output dot > services.dot

$ dot -Tpng services.dot -o services.png
$ dot -Tsvg services.dot -o services.svg

Generated visual dependency graph:
- services.dot (DOT format)
- services.png (PNG image)
- services.svg (SVG vector graphic)
```

```console
$ kdx relationships --namespace production api-gateway

Service Relationships: api-gateway
├── Depends on:
│   ├── auth-service (ClusterIP)
│   └── user-service (ClusterIP)
├── Used by:
│   └── ingress-nginx (LoadBalancer)
└── Exposes:
    ├── Port 80 -> 8080
    └── Port 443 -> 8443
```

```console
$ kdx ingress --namespace production

Ingress Discovery:
├── main-ingress
│   ├── Host: api.company.com
│   ├── Path: /
│   └── Backend: api-gateway:80
├── auth-ingress
│   ├── Host: auth.company.com
│   ├── Path: /
│   └── Backend: auth-service:8080
└── LoadBalancer Services:
    └── api-gateway (External IP: 203.0.113.10)
```

```console
$ kdx services --namespace production --format json

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

```console
$ kdx services --namespace production --format yaml

- name: api-gateway
  namespace: production
  type: LoadBalancer
  cluster_ip: 10.0.1.100
  external_ips:
    - 203.0.113.10
  ports:
    - name: http
      port: 80
      target_port: "8080"
      protocol: TCP
    - name: https
      port: 443
      target_port: "8443"
      protocol: TCP
  selector:
    app: api-gateway
    version: v1.2.3
  age: 5d
```

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

### Service Topology Analysis (Most Unique Feature)

<details>
<summary><strong>Click to see how kdx maps your service dependencies</strong></summary>

```console
$ kdx topology --namespace production api-gateway

Service Topology: api-gateway
├── Namespace: production
├── Type: LoadBalancer
├── Cluster IP: 10.0.1.100
├── External IP: 203.0.113.10
└── Backend Pods:
    ├── api-gateway-7d4b8c9f-abc12 (Running)
    └── api-gateway-7d4b8c9f-def34 (Running)
```

```console
$ kdx topology --namespace kube-system coredns

Service Topology: coredns
├── Namespace: kube-system
├── Type: ClusterIP
├── Cluster IP: 10.43.0.10
└── Backend Pods:
    ├── coredns-6799fbcd5-m8r7x (Running)
    └── coredns-6799fbcd5-n9s4y (Running)
```

</details>

### Service Discovery & Health

<details>
<summary><strong>Click to see service listing with health status</strong></summary>

```console
$ kdx services -n production --health

Scanning services in production namespace...

┌─────────────┬─────────────┬────────────┬───────┬─────┬────────┬──────────┐
│ NAME        │ TYPE        │ CLUSTER-IP │ PORTS │ AGE │ HEALTH │ REPLICAS │
├─────────────┼─────────────┼────────────┼───────┼─────┼────────┼──────────┤
│ api-gateway │ LoadBalancer│ 10.0.1.100 │ 80    │ 5d  │ UP     │ 3/3      │
│ auth-service│ ClusterIP   │ 10.0.1.101 │ 8080  │ 5d  │ UP     │ 2/2      │
│ user-service│ ClusterIP   │ 10.0.1.102 │ 8080  │ 3d  │ UP     │ 2/2      │
│ database    │ ClusterIP   │ 10.0.1.103 │ 5432  │ 10d │ UP     │ 1/1      │
│ redis-cache │ ClusterIP   │ 10.0.1.104 │ 6379  │ 8d  │ WARN   │ 1/2      │
│ old-service │ ClusterIP   │ 10.0.1.105 │ 8080  │ 30d │ DOWN   │ 0/1      │
└─────────────┴─────────────┴────────────┴───────┴─────┴────────┴──────────┘

Health Summary: 4 healthy, 1 warning, 1 down
WARNING: redis-cache: 1 pod not ready
ERROR: old-service: No healthy pods found

Found 6 services in production namespace
```

</details>

### Ingress Discovery & External Access

<details>
<summary><strong>Click to see how services are exposed externally</strong></summary>

```console
$ kdx ingress -n production --show-backends

Discovering external access points...

┌─────────────────────────────────────────────────────────────────────────────┐
│                            External Access Map                              │
└─────────────────────────────────────────────────────────────────────────────┘

Ingress Routes:
┌─────────────────────────┬─────────────────┬─────────────────┬──────────────┐
│ EXTERNAL URL            │ INGRESS         │ SERVICE         │ BACKEND PODS │
├─────────────────────────┼─────────────────┼─────────────────┼──────────────┤
│ https://api.company.com │ main-ingress    │ api-gateway:80  │ 3 ready      │
│ https://app.company.com │ main-ingress    │ frontend:3000   │ 2 ready      │
│ https://auth.company.com│ auth-ingress    │ auth-service:80 │ 2 ready      │
│ https://admin.company.com│ admin-ingress  │ admin-panel:80  │ 1 ready      │
└─────────────────────────┴─────────────────┴─────────────────┴──────────────┘

LoadBalancer Services:
┌─────────────────┬─────────────────┬─────────────────┬──────────────────────┐
│ SERVICE         │ EXTERNAL IP     │ PORTS           │ STATUS               │
├─────────────────┼─────────────────┼─────────────────┼──────────────────────┤
│ api-gateway     │ 203.0.113.10    │ 80:30080/TCP    │ Ready                │
│ monitoring-lb   │ 203.0.113.11    │ 3000:30300/TCP  │ Ready                │
│ legacy-service  │ <pending>       │ 8080:31080/TCP  │ Provisioning         │
└─────────────────┴─────────────────┴─────────────────┴──────────────────────┘

Access Summary:
  - 4 ingress routes configured
  - 2 LoadBalancer services active
  - 1 service pending external IP
  - All backend pods healthy

Use 'kdx describe <service>' for detailed routing information
```

</details>

### Service Description

<details>
<summary><strong>Click to see detailed service information</strong></summary>

```console
$ kdx describe api-gateway -n production

Service Details
┌─────────────────────────────────────────────────────────────────┐
│ Service: api-gateway                                            │
│ Namespace: production                                           │
│ Type: LoadBalancer                                              │
│ Cluster IP: 10.0.1.100                                         │
│ External IP: 203.0.113.10                                      │
└─────────────────────────────────────────────────────────────────┘

Ports:
  - http  80:TCP -> 8080 (TCP)
  - https 443:TCP -> 8443 (TCP)

Selector:
  - app = api-gateway
  - version = v1.2.3

Related Pods:
┌───────────────────────────┬───────────┬─────────┬───────┬──────────┬─────┬─────────────┬──────────┐
│ NAME                      │ NAMESPACE │ STATUS  │ READY │ RESTARTS │ AGE │ IP          │ NODE     │
├───────────────────────────┼───────────┼─────────┼───────┼──────────┼─────┼─────────────┼──────────┤
│ api-gateway-7d4b8c9f-abc12│ production│ Running │ 2/2   │ 0        │ 2d  │ 10.244.1.15│ worker-1 │
│ api-gateway-7d4b8c9f-def34│ production│ Running │ 2/2   │ 0        │ 2d  │ 10.244.2.20│ worker-2 │
└───────────────────────────┴───────────┴─────────┴───────┴──────────┴─────┴─────────────┴──────────┘

Ingress Routes:
  - https://api.example.com -> api-gateway:80
  - https://api.example.com/v2 -> api-gateway:80

Service is healthy and accessible
```

</details>

### Advanced Graph Visualization

<details>
<summary><strong>Click to see exportable dependency graphs</strong></summary>

```console
$ kdx graph -n microservices --include-pods --format=dot

Generating comprehensive service dependency graph...
Including pod relationships and external connections...

digraph microservices {
  rankdir=TB;
  compound=true;

  // External layer
  subgraph cluster_external {
    label="External Access";
    style=filled;
    color=lightgrey;

    "internet" [shape=cloud, style=filled, color=lightblue, label="Internet\nTraffic"];
    "load_balancer" [shape=diamond, style=filled, color=orange, label="AWS ALB\n203.0.113.10"];
  }

  // Ingress layer
  subgraph cluster_ingress {
    label="Ingress Layer";
    style=filled;
    color=lightyellow;

    "nginx_ingress" [shape=box, style=filled, color=green, label="nginx-ingress\ningress-nginx\n2 pods"];
  }

  // Application layer
  subgraph cluster_app {
    label="Application Services";
    style=filled;
    color=lightcyan;

    "frontend" [shape=box, style=filled, color=lightblue, label="frontend\nmicroservices\n3 pods"];
    "api_gateway" [shape=box, style=filled, color=red, label="api-gateway\nmicroservices\n2 pods"];
    "auth_service" [shape=box, style=filled, color=lightgreen, label="auth-service\nmicroservices\n2 pods"];
    "user_service" [shape=box, style=filled, color=lightgreen, label="user-service\nmicroservices\n3 pods"];
  }

  // Data layer
  subgraph cluster_data {
    label="Data Layer";
    style=filled;
    color=mistyrose;

    "postgres" [shape=cylinder, style=filled, color=blue, label="postgresql\ndatabase\n1 pod"];
    "redis" [shape=cylinder, style=filled, color=red, label="redis\ndatabase\n1 pod"];
  }

  // Connections
  "internet" -> "load_balancer" [style=bold, color=blue, label="HTTPS"];
  "load_balancer" -> "nginx_ingress" [style=bold, color=blue, label="routes"];
  "nginx_ingress" -> "frontend" [style=bold, color=green, label="serves"];
  "frontend" -> "api_gateway" [style=dashed, color=purple, label="API calls"];
  "api_gateway" -> "auth_service" [style=dashed, color=purple, label="auth"];
  "api_gateway" -> "user_service" [style=dashed, color=purple, label="user data"];
  "auth_service" -> "postgres" [style=dotted, color=brown, label="stores"];
  "user_service" -> "postgres" [style=dotted, color=brown, label="queries"];
  "user_service" -> "redis" [style=dotted, color=brown, label="caches"];
}

Graph generated successfully!
Saved to: microservices-topology.dot
Convert to image: dot -Tpng microservices-topology.dot -o topology.png
Or SVG: dot -Tsvg microservices-topology.dot -o topology.svg
```

</details>

### JSON Output

<details>
<summary><strong>Click to see JSON format output</strong></summary>

```console
$ kdx services -n production --output=json

Exporting services in JSON format...
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

```console
Successfully exported 1 service
```

</details>
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
