//! K8s Explorer - Kubernetes Cluster Discovery Tool
//!
//! A command-line tool for exploring and discovering resources in Kubernetes clusters.
//! Provides easy-to-use commands for listing services, pods, and understanding
//! cluster topology and relationships.

mod cli;
mod discovery;
mod error;
mod graph;
mod output;

use clap::Parser;
use cli::{Cli, Commands};
use discovery::ServiceHealth;
use std::process;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    // Load Kubernetes configuration
    let config = if let Some(context) = &cli.context {
        kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions {
            context: Some(context.clone()),
            cluster: None,
            user: None,
        })
        .await?
    } else {
        kube::Config::infer().await?
    };

    // Create Kubernetes client
    let client = kube::Client::try_from(config)?;

    // Create discovery engine
    let discovery = discovery::DiscoveryEngine::new(client);

    // Execute command
    match cli.command {
        Commands::Services {
            namespace,
            all_namespaces,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let services = discovery.list_services(ns).await?;
            output::print_services(&services, &cli.output)?;
        }
        Commands::Pods {
            namespace,
            selector,
            all_namespaces,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let pods = discovery.list_pods(ns, selector.as_deref()).await?;
            output::print_pods(&pods, &cli.output)?;
        }
        Commands::Describe { service, namespace } => {
            let ns = namespace
                .as_deref()
                .or(cli.namespace.as_deref())
                .unwrap_or("default");
            let service_info = discovery.describe_service(&service, ns).await?;
            output::print_service_description(&service_info, &cli.output)?;

            // Also show ingress information if available
            let ingress_routes = discovery
                .discover_ingress_for_service(&service, ns)
                .await
                .unwrap_or_default();
            if !ingress_routes.is_empty() {
                output::print_ingress_info(&ingress_routes, &cli.output)?;

                // Also show configuration information if available
                let (configmaps, secrets) = discovery
                    .discover_service_configuration(&service, ns)
                    .await
                    .unwrap_or_default();
                if !configmaps.is_empty() || !secrets.is_empty() {
                    output::print_configuration_info(&configmaps, &secrets, &cli.output)?;

                    // Also show health information
                    let health = discovery
                        .check_service_health(&service, ns)
                        .await
                        .unwrap_or_else(|_| ServiceHealth {
                            service_name: service.clone(),
                            namespace: ns.to_string(),
                            overall_healthy: false,
                            checked_at: "Error checking health".to_string(),
                        });
                    output::print_health_info(&health, &cli.output)?;
                }
            }
        }
        Commands::Topology { service, namespace } => {
            let ns = namespace
                .as_deref()
                .or(cli.namespace.as_deref())
                .unwrap_or("default");
            let topology = discovery.analyze_service_topology(&service, ns).await?;
            output::print_service_topology(&topology, &cli.output)?;
        }
        Commands::Graph {
            namespace,
            format,
            include_pods,
            highlight,
        } => {
            let ns = namespace.as_deref();
            let service_graph =
                graph::generate_service_graph(&discovery, ns, include_pods, highlight.as_deref())
                    .await?;

            match format {
                cli::GraphFormat::Dot => {
                    println!("{}", service_graph.to_dot());
                }
                cli::GraphFormat::Svg => {
                    println!("{}", service_graph.to_svg()?);
                }
            }
        }
    }

    Ok(())
}
