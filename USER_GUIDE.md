# kdx User Guide

Complete documentation for all kdx commands and features.

## Table of Contents

- [Core Resource Commands](#core-resource-commands)
- [Configuration Management](#configuration-management)
- [Custom Resources](#custom-resources)
- [Service Analysis](#service-analysis)
- [Performance and Scale](#performance-and-scale)
- [Cache Management](#cache-management)
- [Advanced Filtering](#advanced-filtering)
- [Resource Grouping](#resource-grouping)
- [Output Formats](#output-formats)
- [Global Options](#global-options)

## Core Resource Commands

### Services

List and analyze Kubernetes services across your cluster.

```bash
# List services in current namespace
kdx services

# List services across all namespaces
kdx services --all-namespaces

# Filter services by labels
kdx services --selector app=web

# Group services by application
kdx services --group-by app

# Output in JSON format
kdx services --output json

# High-performance discovery with progress tracking
kdx services --all-namespaces --show-progress

# Stream large datasets efficiently
kdx services --all-namespaces --stream --output json --limit 1000
```

### Pods

Discover and filter pods with advanced criteria.

```bash
# List pods in current namespace
kdx pods

# Complex label filtering
kdx pods --selector 'app=web,tier!=cache'

# Filter by pod status
kdx pods --status Running
kdx pods --status Pending
kdx pods --status Failed

# Group pods by application
kdx pods --group-by app

# Check pods across environments
kdx pods --selector 'env in (prod,staging)' --all-namespaces

# Discovery with progress tracking
kdx pods --all-namespaces --show-progress

# Memory-efficient processing of large pod lists
kdx pods --all-namespaces --limit 500 --page-size 50
```

### Deployments

Analyze deployment status and organization.

```bash
# List deployments
kdx deployments

# Filter by deployment status
kdx deployments --status Ready
kdx deployments --status NotReady

# Group by Helm release
kdx deployments --group-by helm-release

# Complex filtering and grouping
kdx deployments --selector 'app=web,tier=frontend' --group-by tier
```

### StatefulSets

Manage and analyze StatefulSet resources.

```bash
# List StatefulSets
kdx statefulsets

# Group by Helm release for deployment management
kdx statefulsets --group-by helm-release

# Filter by labels
kdx statefulsets --selector app=database
```

### DaemonSets

Discover DaemonSets across your cluster.

```bash
# List DaemonSets
kdx daemonsets

# List across all namespaces
kdx daemonsets --all-namespaces

# Filter by labels
kdx daemonsets --selector app=monitoring
```

## Configuration Management

### ConfigMaps

Analyze ConfigMap usage and identify cleanup opportunities.

```bash
# List ConfigMaps with usage information
kdx configmaps

# Find unused ConfigMaps for cleanup
kdx configmaps --unused

# Find unused ConfigMaps across all namespaces
kdx configmaps --unused --all-namespaces

# Filter by labels
kdx configmaps --selector app=web

# Group by namespace
kdx configmaps --group-by namespace

# Group by application
kdx configmaps --group-by app
```

### Secrets

Manage secrets with security-conscious analysis.

```bash
# List secrets (shows data keys only, never values)
kdx secrets

# Filter by secret type
kdx secrets --secret-type Opaque
kdx secrets --secret-type kubernetes.io/tls
kdx secrets --secret-type kubernetes.io/dockerconfigjson

# Find unused secrets
kdx secrets --unused

# Find unused secrets cluster-wide
kdx secrets --unused --all-namespaces

# Group by namespace for organization
kdx secrets --group-by namespace
```

## Custom Resources

### Custom Resource Definitions (CRDs)

Discover and analyze CRDs in your cluster.

```bash
# List all CRDs
kdx crds

# Show only CRDs that have active instances
kdx crds --with-instances

# Display version information
kdx crds --show-versions

# Group by scope (Cluster vs Namespaced)
kdx crds --group-by scope

# Combine options for comprehensive analysis
kdx crds --with-instances --show-versions --group-by scope
```

### Custom Resource Instances

Explore instances of specific Custom Resources.

```bash
# List instances of a specific CRD
kdx custom-resources prometheuses.monitoring.coreos.com

# List in specific namespace
kdx custom-resources certificates.cert-manager.io --namespace production

# List across all namespaces
kdx custom-resources servicemonitors.monitoring.coreos.com --all-namespaces

# Filter by labels
kdx custom-resources prometheuses.monitoring.coreos.com --selector app=monitoring
```

## Service Analysis

### Service Description

Get detailed information about services and their relationships.

```bash
# Describe a service
kdx describe api-gateway

# Describe service in specific namespace
kdx describe grafana --namespace monitoring

# Get service information with backend details
kdx describe frontend --namespace production
```

### Service Topology

Understand service topology and backend connections.

```bash
# Show service topology
kdx topology grafana --namespace monitoring

# Analyze service relationships
kdx topology api-gateway --namespace production

# Understand service backend connections
kdx topology coredns --namespace kube-system
```

### Dependency Graphs

Generate visual service dependency graphs.

```bash
# Generate dependency graph for namespace
kdx graph --namespace monitoring

# Generate in DOT format for visualization tools
kdx graph --namespace production --output dot

# Generate in JSON format for programmatic use
kdx graph --namespace production --output json

# Save graph to file for external processing
kdx graph --namespace monitoring > services.dot

# Convert to PNG using Graphviz
kdx graph --namespace monitoring | dot -Tpng -o services.png

# Convert to SVG using Graphviz
kdx graph --namespace monitoring | dot -Tsvg -o services.svg
```

## Performance and Scale

kdx is designed to handle large Kubernetes clusters efficiently through concurrent discovery, intelligent caching, and memory optimization features.

### Concurrent Discovery

kdx automatically processes multiple namespaces in parallel for faster resource discovery when using `--all-namespaces`.

```bash
# Automatic concurrent discovery across all namespaces
kdx services --all-namespaces

# Concurrent discovery with progress tracking
kdx pods --all-namespaces --show-progress

# All multi-namespace operations use optimal concurrency automatically
kdx deployments --all-namespaces
```

### Pagination and Limits

Handle large datasets efficiently with pagination and result limiting.

```bash
# Limit total results
kdx services --limit 100

# Use smaller page sizes for API calls
kdx services --page-size 25

# Combine with other options
kdx pods --all-namespaces --limit 500 --page-size 50 --show-progress
```

### Memory Optimization

Use streaming output and lazy conversion for large datasets.

```bash
# Stream JSON output for large datasets
kdx services --stream --output json --limit 1000

# Stream YAML output
kdx pods --stream --output yaml --all-namespaces

# Enable memory optimization features
kdx services --memory-optimized --all-namespaces
```

### Progress Tracking

Monitor long-running operations with real-time progress indicators.

```bash
# Show progress for service discovery
kdx services --all-namespaces --show-progress

# Progress tracking for multi-namespace operations
kdx pods --all-namespaces --show-progress

# Progress tracking for cache operations
kdx cache warm --show-progress
```

## Cache Management

kdx includes intelligent caching to improve performance for repeated operations.

### Cache Statistics

View current cache status and performance metrics.

```bash
# Show detailed cache statistics
kdx cache stats

# Example output:
# Cache Statistics:
#   Services entries: 19
#   Pods entries: 19
#   Deployments entries: 12
#   Total entries: 50
#   Default TTL: 5m0s
```

### Cache Operations

Manage cache lifecycle for optimal performance.

```bash
# Clear all cached data
kdx cache clear

# Warm cache for better performance
kdx cache warm

# Warm specific namespaces
kdx cache warm --namespaces production staging

# Warm specific resource types
kdx cache warm --resources services pods
```

### Performance Testing

Benchmark kdx performance with your cluster configuration.

```bash
# Basic performance benchmark
kdx benchmark

# Test with more iterations
kdx benchmark --iterations 10

# Test concurrent discovery performance
kdx benchmark --test-concurrent

# Test memory optimization features
kdx benchmark --test-memory

# Benchmark specific resource types
kdx benchmark --resources services pods deployments
```

## Advanced Filtering

### Label Selector Syntax

kdx supports complex label selector expressions for precise resource filtering.

#### Basic Operations

```bash
# Equality
kdx pods --selector app=web

# Inequality
kdx pods --selector tier!=database

# Existence check
kdx pods --selector app                    # Has 'app' label
kdx pods --selector '!temp'                # Does not have 'temp' label
```

#### Set-based Operations

```bash
# In operator
kdx pods --selector 'env in (prod,staging)'

# Not in operator
kdx pods --selector 'tier notin (cache,temp)'
```

#### Complex Combinations

```bash
# Multiple conditions with AND
kdx deployments --selector 'app=web,tier=frontend,env in (prod,staging)'

# Complex application analysis
kdx pods --selector 'app=api,version!=deprecated,env in (prod,staging)'
```

### Status Filtering

Filter resources by their operational status.

```bash
# Pod status filtering
kdx pods --status Running
kdx pods --status Pending
kdx pods --status Failed

# Deployment status filtering
kdx deployments --status Ready
kdx deployments --status NotReady
```

## Resource Grouping

Organize resources by various criteria for better analysis.

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

### Practical Grouping Examples

```bash
# Analyze Helm releases
kdx deployments --group-by helm-release
kdx services --group-by helm-release
kdx configmaps --group-by helm-release

# Application-centric analysis
kdx pods --selector app=ecommerce --group-by tier
kdx services --selector app=ecommerce --group-by tier

# Environment analysis
kdx deployments --group-by env
kdx secrets --group-by env
```

## Output Formats

kdx supports multiple output formats for different use cases.

### Available Formats

```bash
# Human-readable table (default)
kdx services

# JSON output for programmatic use
kdx services --output json

# YAML output for configuration management
kdx services --output yaml
```

### Practical Examples

```bash
# Export service configuration
kdx services --namespace production --output yaml > services.yaml

# Generate JSON for automation scripts
kdx deployments --selector app=web --output json | jq '.[] | .name'

# Create reports
kdx configmaps --unused --all-namespaces --output json > unused-configs.json
```

## Namespace Options

Control which namespaces to query.

```bash
# Specific namespace
kdx services --namespace kube-system

# All namespaces
kdx services --all-namespaces

# Use context default namespace
kdx services
```

## Real-world Examples

### Application Health Check

```bash
# Check application health across environments
kdx pods --selector 'app=api,env in (prod,staging)' --status Running
kdx deployments --selector 'app=api,env in (prod,staging)' --status Ready
```

### Security Audit

```bash
# Audit TLS certificates
kdx secrets --secret-type kubernetes.io/tls --all-namespaces

# Find unused configurations for cleanup
kdx configmaps --unused --all-namespaces
kdx secrets --unused --all-namespaces
```

### Infrastructure Analysis

```bash
# Survey custom resources in cluster
kdx crds --with-instances --show-versions

# Analyze monitoring stack
kdx custom-resources prometheuses.monitoring.coreos.com
kdx custom-resources servicemonitors.monitoring.coreos.com

# Check cert-manager certificates
kdx custom-resources certificates.cert-manager.io --all-namespaces
```

### Helm Release Management

```bash
# Analyze specific Helm release
kdx pods --selector 'app.kubernetes.io/instance=prometheus' --group-by app
kdx services --selector 'app.kubernetes.io/instance=prometheus'
kdx configmaps --selector 'app.kubernetes.io/instance=prometheus'
```

## Global Options

These options are available for all kdx commands and can be combined for optimal performance and usability.

### Performance Options

```bash
# Pagination and result limiting
--limit <number>              # Limit total results returned
--page-size <number>          # Set API request page size (default: 100)

# Progress tracking
--show-progress              # Display progress indicators for long operations

# Memory optimization
--stream                     # Use streaming output for large datasets (JSON/YAML only)
--memory-optimized          # Enable memory optimization features
```

### Standard Options

```bash
# Namespace targeting
--namespace <name>           # Target specific namespace
--all-namespaces            # Query across all namespaces

# Output control
--output <format>           # Output format: table (default), json, yaml
--verbose                   # Enable verbose output

# Kubernetes context
--context <name>            # Use specific kubeconfig context
```

### Usage Examples

```bash
# High-performance discovery across large cluster
kdx services --all-namespaces --limit 1000 --show-progress

# Memory-efficient export of large dataset
kdx pods --all-namespaces --stream --output json --limit 5000 > pods.json

# Detailed analysis with progress tracking
kdx deployments --all-namespaces --verbose --show-progress --page-size 50

# Quick targeted discovery
kdx services --namespace production --limit 20 --output yaml
```

### Performance Recommendations

For optimal performance with large clusters:

1. **Automatic concurrent discovery**: kdx automatically uses optimal concurrency for multi-namespace operations
2. **Enable progress tracking**: Use `--show-progress` for long-running operations
3. **Limit results**: Use `--limit` to avoid processing unnecessary data
4. **Stream large datasets**: Use `--stream` with JSON/YAML output for datasets over 1000 items
5. **Warm the cache**: Run `kdx cache warm` before batch operations
6. **Monitor cache performance**: Use `kdx cache stats` to verify cache effectiveness
