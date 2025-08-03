//! K8s Explorer - Kubernetes Cluster Discovery Tool
//!
//! A command-line tool for exploring and discovering resources in Kubernetes clusters.
//! Provides easy-to-use commands for listing services, pods, and understanding
//! cluster topology and relationships.

mod cli;
mod discovery;
mod output;
mod error;

use cli::{Cli, Commands};
use clap::Parser;
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
        }).await?
    } else {
        kube::Config::infer().await?
    };
    
    // Create Kubernetes client
    let client = kube::Client::try_from(config)?;
    
    // Create discovery engine
    let discovery = discovery::DiscoveryEngine::new(client);
    
    // Execute command
    match cli.command {
        Commands::Services { namespace, all_namespaces } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };
            
            let services = discovery.list_services(ns).await?;
            output::print_services(&services, &cli.output)?;
        }
        Commands::Pods { namespace, selector, all_namespaces } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };
            
            let pods = discovery.list_pods(ns, selector.as_deref()).await?;
            output::print_pods(&pods, &cli.output)?;
        }
        Commands::Describe { service, namespace } => {
            let ns = namespace.as_deref().or(cli.namespace.as_deref()).unwrap_or("default");
            let service_info = discovery.describe_service(&service, ns).await?;
            output::print_service_description(&service_info, &cli.output)?;
        }
        Commands::Topology { service, namespace } => {
            let ns = namespace.as_deref().or(cli.namespace.as_deref()).unwrap_or("default");
            let topology = discovery.analyze_service_topology(&service, ns).await?;
            output::print_service_topology(&topology, &cli.output)?;
        }
    }
    
    Ok(())
}
