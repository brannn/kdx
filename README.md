# kdx - K8s Discovery Experience


[![Build Status](https://github.com/brannn/kdx/actions/workflows/ci.yml/badge.svg)](https://github.com/brannn/kdx/actions)
[![Tests](https://img.shields.io/badge/tests-23%20passing-brightgreen.svg)](https://github.com/brannn/kdx/actions)
[![Release](https://img.shields.io/github/v/release/brannn/kdx?label=release)](https://github.com/brannn/kdx/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Homebrew](https://img.shields.io/badge/homebrew-available-brightgreen.svg)](https://github.com/brannn/homebrew-kdx)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/brannn/kdx)

A command-line tool for exploring and discovering resources in Kubernetes clusters. Provides commands for listing services, pods, and understanding cluster topology and relationships.

## Quick Demo

### Comprehensive Resource Discovery with Advanced Filtering

```console
$ kdx deployments --selector 'app=web,tier!=cache' --group-by helm-release

=== Deployment Group: web-frontend (helm-release) ===
NAME           NAMESPACE    READY   UP-TO-DATE   AVAILABLE   AGE
web-frontend   production   3/3     3            3           5d
web-api        production   2/2     2            2           5d

Total deployments in group: 2
```

### Configuration Management and Security Analysis

```console
$ kdx configmaps --unused --all-namespaces

NAME                NAMESPACE     DATA   AGE    USED BY
old-config          staging       3      30d    None
deprecated-settings production    1      45d    None

$ kdx secrets --secret-type kubernetes.io/tls --group-by namespace

=== Secret Group: production (namespace) ===
NAME              TYPE                AGE    USED BY
api-tls-cert      kubernetes.io/tls   90d    ingress-nginx
auth-tls-cert     kubernetes.io/tls   45d    auth-service
```

### Custom Resource Discovery and Analysis

```console
$ kdx crds --with-instances --show-versions

NAME                              GROUP                 VERSION   INSTANCES   AGE
prometheuses.monitoring.coreos.com monitoring.coreos.com v1        3          90d
certificates.cert-manager.io      cert-manager.io       v1        12         120d

$ kdx custom-resources prometheuses.monitoring.coreos.com

NAME                 NAMESPACE    VERSION   REPLICAS   AGE
prometheus-main      monitoring   v2.45.0   2          90d
prometheus-federate  monitoring   v2.45.0   1          60d
```

### Service Topology and Dependency Analysis

```console
$ kdx topology --namespace monitoring grafana

Service Topology: grafana
├── Namespace: monitoring
├── Type: LoadBalancer
├── Cluster IP: 10.43.132.227
└── Backend Pods:
    └── grafana-5df9c4787-fxl27 (Running)

$ kdx graph --namespace production --output dot > services.dot
# Generate visual dependency graphs for architecture documentation
```

**📖 For comprehensive documentation and examples of all commands, see the [User Guide](USER_GUIDE.md)**

```console
$ kdx services --namespace production --selector 'service.kubernetes.io/load-balancer-cleanup'

NAME           TYPE           CLUSTER-IP     EXTERNAL-IP      PORT(S)        AGE
api-gateway    LoadBalancer   10.43.132.227  203.0.113.10     80:30080/TCP   5d
auth-service   LoadBalancer   10.43.132.228  203.0.113.11     80:30081/TCP   5d
```

```console
$ kdx services --namespace production --output json

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
$ kdx services --namespace production --output yaml

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

### Resource Discovery
- **Workload Resources**: Deployments, StatefulSets, DaemonSets with replica status and metadata
- **Core Resources**: Services, Pods with comprehensive filtering and status information
- **Configuration Resources**: ConfigMaps and Secrets with usage tracking and association mapping
- **Custom Resources**: CRD discovery with version analysis and instance counting
- **Cross-namespace Discovery**: Query resources across all namespaces or specific namespaces

### Advanced Filtering
- **Label Selectors**: Complex expressions with equals, not-equals, in, not-in, exists, not-exists operators
- **Status Filtering**: Filter by resource status (Running, Pending, Failed, Ready, NotReady)
- **Usage Filtering**: Find unused ConfigMaps and Secrets for cleanup identification
- **Type Filtering**: Filter secrets by type (Opaque, TLS, Docker registry)
- **Instance Filtering**: Show only CRDs that have active instances

### Resource Grouping
- **Application Grouping**: Group by app label for application-centric views
- **Helm Release Grouping**: Group by Helm release for deployment management
- **Namespace Grouping**: Organize resources by namespace boundaries
- **Tier Grouping**: Group by tier labels (frontend, backend, database)
- **Custom Label Grouping**: Group by any custom label key

### Analysis and Visualization
- **Topology Analysis**: Service dependency mapping and relationship discovery
- **Graph Visualization**: Generate service dependency graphs in DOT and SVG formats
- **Configuration Analysis**: Map ConfigMaps and Secrets to consuming resources
- **Version Analysis**: CRD version tracking with served/storage status
- **Security Analysis**: Secret usage patterns without exposing sensitive data
- **Multiple Output Formats**: Table, JSON, and YAML output for all resource types

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
### Core Resource Commands

```bash
# Services
kdx services                                    # List services in current namespace
kdx services --all-namespaces                  # List services across all namespaces
kdx services --selector app=web                # Filter services by labels

# Pods
kdx pods                                        # List pods in current namespace
kdx pods --selector app=web,tier!=cache        # Complex label filtering
kdx pods --status Running                      # Filter by pod status
kdx pods --group-by app                        # Group pods by application

# Workload Resources
kdx deployments                                 # List deployments
kdx deployments --status Ready                 # Filter by deployment status
kdx statefulsets --group-by helm-release       # Group StatefulSets by Helm release
kdx daemonsets --all-namespaces                # List DaemonSets across all namespaces
```

### Configuration and Security

```bash
# ConfigMaps
kdx configmaps                                  # List ConfigMaps with usage info
kdx configmaps --unused                        # Find unused ConfigMaps
kdx configmaps --selector app=web              # Filter by labels
kdx configmaps --group-by namespace            # Group by namespace

# Secrets
kdx secrets                                     # List secrets (data keys only, no values)
kdx secrets --secret-type kubernetes.io/tls    # Filter by secret type
kdx secrets --unused --all-namespaces          # Find unused secrets cluster-wide
```

### Custom Resources

```bash
# Custom Resource Definitions
kdx crds                                        # List all CRDs
kdx crds --with-instances                       # Show only CRDs with active instances
kdx crds --show-versions                        # Display version information
kdx crds --group-by scope                       # Group by Cluster vs Namespaced

# Custom Resource Instances
kdx custom-resources prometheuses.monitoring.coreos.com    # List instances of specific CRD
kdx custom-resources certificates.cert-manager.io -n prod  # List in specific namespace
```

### Topology and Analysis

```bash
# Service Topology
kdx describe grafana -n monitoring             # Get detailed service information
kdx topology grafana -n monitoring             # Show service topology and relationships

# Graph Visualization
kdx graph -n monitoring                         # Generate service dependency graph
kdx graph --output dot                          # Generate DOT format graph
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

### Application Analysis

```bash
# Analyze a complete application stack
kdx deployments --selector app=ecommerce --group-by tier
kdx services --selector app=ecommerce --group-by tier
kdx configmaps --selector app=ecommerce

# Check application health across environments
kdx pods --selector 'app=api,env in (prod,staging)' --status Running
kdx deployments --selector 'app=api,env in (prod,staging)' --status Ready
```

### Security and Configuration Audit

```bash
# Find unused configurations for cleanup
kdx configmaps --unused --all-namespaces
kdx secrets --unused --all-namespaces

# Audit TLS certificates
kdx secrets --secret-type kubernetes.io/tls --all-namespaces

# Review configuration usage patterns
kdx configmaps --group-by app --output yaml
```

### Helm Release Management

```bash
# Analyze Helm releases
kdx deployments --group-by helm-release
kdx services --group-by helm-release
kdx configmaps --group-by helm-release

# Check specific Helm release
kdx pods --selector 'app.kubernetes.io/instance=prometheus' --group-by app
```

### Custom Resource Analysis

```bash
# Survey custom resources in cluster
kdx crds --with-instances --show-versions

# Analyze monitoring stack
kdx custom-resources prometheuses.monitoring.coreos.com
kdx custom-resources servicemonitors.monitoring.coreos.com

# Check cert-manager certificates
kdx custom-resources certificates.cert-manager.io --all-namespaces
```

### Service Discovery and Analysis

```bash
# List services with selector filtering
kdx services --selector app=nginx

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

# Generate DOT format for visualization tools
kdx graph --output dot

# Generate JSON format for programmatic use
kdx graph --output json

# Save graph to file
kdx graph -n monitoring > services.dot
```

## Advanced Filtering and Grouping

### Label Selector Syntax

```bash
# Basic equality
kdx pods --selector app=web

# Inequality
kdx pods --selector tier!=database

# Set-based selection
kdx pods --selector 'env in (prod,staging)'
kdx pods --selector 'tier notin (cache,temp)'

# Existence checks
kdx pods --selector app                    # Has 'app' label
kdx pods --selector '!temp'                # Does not have 'temp' label

# Complex combinations
kdx deployments --selector 'app=web,tier=frontend,env in (prod,staging)'
```

### Grouping Options

```bash
# Group by application
kdx services --group-by app

# Group by tier (frontend, backend, database)
kdx deployments --group-by tier

# Group by Helm release
kdx pods --group-by helm-release

# Group by namespace
kdx configmaps --group-by namespace

# Group by custom label
kdx secrets --group-by environment
```

### Status and Type Filtering

```bash
# Pod status filtering
kdx pods --status Running
kdx pods --status Pending
kdx pods --status Failed

# Deployment status filtering
kdx deployments --status Ready
kdx deployments --status NotReady

# Secret type filtering
kdx secrets --secret-type Opaque
kdx secrets --secret-type kubernetes.io/tls
kdx secrets --secret-type kubernetes.io/dockerconfigjson

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
