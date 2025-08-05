//! K8s Explorer - Kubernetes Cluster Discovery Tool
//!
//! A command-line tool for exploring and discovering resources in Kubernetes clusters.
//! Provides easy-to-use commands for listing services, pods, and understanding
//! cluster topology and relationships.

mod cache;
mod cli;
mod discovery;
mod error;
mod filtering;
mod graph;
mod output;
mod progress;

use clap::Parser;
use cli::{Cli, Commands};
use discovery::ServiceHealth;
use filtering::{FilterCriteria, GroupBy, ResourceFilter, ResourceGrouper};
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
            selector,
            group_by,
        } => {
            let mut services = if all_namespaces {
                // Use concurrent discovery for all namespaces
                let progress = if cli.show_progress {
                    Some(crate::progress::ProgressTracker::new(true, None))
                } else {
                    None
                };

                let namespaces = discovery.get_all_namespaces().await?;
                let result = discovery.list_services_concurrent(
                    namespaces,
                    selector.as_deref(),
                    cli.limit,
                    cli.page_size,
                    true, // Use cache
                    cli.concurrency,
                    progress.as_ref(),
                ).await?;

                if let Some(progress) = progress {
                    progress.finish_and_clear();
                }

                result
            } else {
                // Single namespace discovery
                let ns = namespace.as_deref().or(cli.namespace.as_deref());

                let progress = if cli.show_progress {
                    Some(crate::progress::ProgressTracker::new_spinner(true, "Discovering services..."))
                } else {
                    None
                };

                let result = discovery.list_services_with_options(
                    ns,
                    selector.as_deref(),
                    cli.limit,
                    cli.page_size,
                    true, // Use cache
                ).await?;

                if let Some(progress) = progress {
                    progress.finish_and_clear();
                }

                result
            };

            // Apply filtering
            let criteria = FilterCriteria {
                label_selector: selector,
                ..Default::default()
            };
            services = ResourceFilter::filter_services(services, &criteria);

            // Apply grouping if specified
            if let Some(group_by_str) = group_by {
                let group_by = parse_group_by(&group_by_str);
                let grouped = ResourceGrouper::group_resources(
                    services,
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    &group_by,
                );
                output::print_grouped_resources(&grouped, &cli.output)?;
            } else {
                output::print_services(&services, &cli.output)?;
            }
        }
        Commands::Pods {
            namespace,
            selector,
            all_namespaces,
            status,
            group_by,
        } => {
            let mut pods = if all_namespaces {
                // Use concurrent discovery for all namespaces
                let progress = if cli.show_progress {
                    Some(crate::progress::ProgressTracker::new(true, None))
                } else {
                    None
                };

                let namespaces = discovery.get_all_namespaces().await?;
                let result = discovery.list_pods_concurrent(
                    namespaces,
                    selector.as_deref(),
                    cli.limit,
                    cli.page_size,
                    true, // Use cache
                    cli.concurrency,
                    progress.as_ref(),
                ).await?;

                if let Some(progress) = progress {
                    progress.finish_and_clear();
                }

                result
            } else {
                // Single namespace discovery
                let ns = namespace.as_deref().or(cli.namespace.as_deref());

                let progress = if cli.show_progress {
                    Some(crate::progress::ProgressTracker::new_spinner(true, "Discovering pods..."))
                } else {
                    None
                };

                let result = discovery.list_pods_with_options(
                    ns,
                    selector.as_deref(),
                    cli.limit,
                    cli.page_size,
                    true, // Use cache
                ).await?;

                if let Some(progress) = progress {
                    progress.finish_and_clear();
                }

                result
            };

            // Apply additional filtering
            let criteria = FilterCriteria {
                label_selector: selector,
                status_filter: status,
                ..Default::default()
            };
            pods = ResourceFilter::filter_pods(pods, &criteria);

            // Apply grouping if specified
            if let Some(group_by_str) = group_by {
                let group_by = parse_group_by(&group_by_str);
                let grouped = ResourceGrouper::group_resources(
                    vec![],
                    pods,
                    vec![],
                    vec![],
                    vec![],
                    &group_by,
                );
                output::print_grouped_resources(&grouped, &cli.output)?;
            } else {
                output::print_pods(&pods, &cli.output)?;
            }
        }
        Commands::Deployments {
            namespace,
            all_namespaces,
            selector,
            status,
            group_by,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let progress = if cli.show_progress {
                Some(crate::progress::ProgressTracker::new_spinner(true, "Discovering deployments..."))
            } else {
                None
            };

            let mut deployments = discovery.list_deployments_with_options(
                ns,
                cli.limit,
                cli.page_size,
                true, // Use cache
            ).await?;

            if let Some(progress) = progress {
                progress.finish_and_clear();
            }

            // Apply filtering
            let criteria = FilterCriteria {
                label_selector: selector,
                status_filter: status,
                ..Default::default()
            };
            deployments = ResourceFilter::filter_deployments(deployments, &criteria);

            // Apply grouping if specified
            if let Some(group_by_str) = group_by {
                let group_by = parse_group_by(&group_by_str);
                let grouped = ResourceGrouper::group_resources(
                    vec![],
                    vec![],
                    deployments,
                    vec![],
                    vec![],
                    &group_by,
                );
                output::print_grouped_resources(&grouped, &cli.output)?;
            } else {
                output::print_deployments(&deployments, &cli.output)?;
            }
        }
        Commands::Statefulsets {
            namespace,
            all_namespaces,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let statefulsets = discovery.list_statefulsets(ns).await?;
            output::print_statefulsets(&statefulsets, &cli.output)?;
        }
        Commands::Daemonsets {
            namespace,
            all_namespaces,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let daemonsets = discovery.list_daemonsets(ns).await?;
            output::print_daemonsets(&daemonsets, &cli.output)?;
        }
        Commands::Configmaps {
            namespace,
            all_namespaces,
            selector,
            group_by,
            unused,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let progress = if cli.show_progress {
                Some(crate::progress::ProgressTracker::new_spinner(true, "Discovering configmaps..."))
            } else {
                None
            };

            let mut configmaps = discovery.list_configmaps_with_options(
                ns,
                cli.limit,
                cli.page_size,
                true, // Use cache
            ).await?;

            if let Some(progress) = progress {
                progress.finish_and_clear();
            }

            // Apply filtering
            let criteria = FilterCriteria {
                label_selector: selector,
                ..Default::default()
            };
            configmaps = ResourceFilter::filter_configmaps(configmaps, &criteria);

            // Filter for unused if requested
            if unused {
                configmaps.retain(|cm| cm.used_by.is_empty());
            }

            // Apply grouping if specified
            if let Some(group_by_str) = group_by {
                let group_by = parse_group_by(&group_by_str);
                let grouped = ResourceGrouper::group_configmaps(configmaps, &group_by);
                output::print_grouped_configmaps(&grouped, &cli.output)?;
            } else {
                output::print_configmaps(&configmaps, &cli.output)?;
            }
        }
        Commands::Secrets {
            namespace,
            all_namespaces,
            selector,
            group_by,
            unused,
            secret_type,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let mut secrets = discovery.list_secrets(ns).await?;

            // Apply filtering
            let criteria = FilterCriteria {
                label_selector: selector,
                ..Default::default()
            };
            secrets = ResourceFilter::filter_secrets(secrets, &criteria);

            // Filter by secret type if specified
            if let Some(stype) = secret_type {
                secrets.retain(|s| s.secret_type == stype);
            }

            // Filter for unused if requested
            if unused {
                secrets.retain(|s| s.used_by.is_empty());
            }

            // Apply grouping if specified
            if let Some(group_by_str) = group_by {
                let group_by = parse_group_by(&group_by_str);
                let grouped = ResourceGrouper::group_secrets(secrets, &group_by);
                output::print_grouped_secrets(&grouped, &cli.output)?;
            } else {
                output::print_secrets(&secrets, &cli.output)?;
            }
        }
        Commands::Crds {
            selector,
            group_by,
            with_instances,
            show_versions,
        } => {
            let mut crds = discovery.list_crds().await?;

            // Apply filtering
            let criteria = FilterCriteria {
                label_selector: selector,
                ..Default::default()
            };
            crds = ResourceFilter::filter_crds(crds, &criteria);

            // Filter for CRDs with instances if requested
            if with_instances {
                crds.retain(|crd| crd.instance_count > 0);
            }

            // Apply grouping if specified
            if let Some(group_by_str) = group_by {
                let group_by = parse_group_by(&group_by_str);
                let grouped = ResourceGrouper::group_crds(crds, &group_by);
                output::print_grouped_crds(&grouped, &cli.output, show_versions)?;
            } else {
                output::print_crds(&crds, &cli.output, show_versions)?;
            }
        }
        Commands::CustomResources {
            crd_name,
            namespace,
            all_namespaces,
            selector,
            group_by,
        } => {
            let ns = if all_namespaces {
                None
            } else {
                namespace.as_deref().or(cli.namespace.as_deref())
            };

            let mut custom_resources = discovery.list_custom_resources(&crd_name, ns).await?;

            // Apply filtering
            let criteria = FilterCriteria {
                label_selector: selector,
                ..Default::default()
            };
            custom_resources = ResourceFilter::filter_custom_resources(custom_resources, &criteria);

            // Apply grouping if specified
            if let Some(group_by_str) = group_by {
                let group_by = parse_group_by(&group_by_str);
                let grouped = ResourceGrouper::group_custom_resources(custom_resources, &group_by);
                output::print_grouped_custom_resources(&grouped, &cli.output)?;
            } else {
                output::print_custom_resources(&custom_resources, &cli.output)?;
            }
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

        Commands::Cache { action } => {
            use cli::CacheAction;

            match action {
                CacheAction::Stats => {
                    let stats = discovery.cache_stats();
                    println!("Cache Statistics:");
                    println!("  Services entries: {}", stats.services_entries);
                    println!("  Pods entries: {}", stats.pods_entries);
                    println!("  Deployments entries: {}", stats.deployments_entries);
                    println!("  StatefulSets entries: {}", stats.statefulsets_entries);
                    println!("  DaemonSets entries: {}", stats.daemonsets_entries);
                    println!("  ConfigMaps entries: {}", stats.configmaps_entries);
                    println!("  Secrets entries: {}", stats.secrets_entries);
                    println!("  CRDs entries: {}", stats.crds_entries);
                    println!("  Custom Resources entries: {}", stats.custom_resources_entries);
                    println!("  Total entries: {}", stats.total_entries());
                    println!("  Default TTL: {:?}", stats.default_ttl);
                }

                CacheAction::Clear => {
                    discovery.clear_cache();
                    println!("Cache cleared successfully");
                }

                CacheAction::Warm { namespaces, resources } => {
                    let progress = if cli.show_progress {
                        Some(crate::progress::ProgressTracker::new_spinner(true, "Warming cache..."))
                    } else {
                        None
                    };

                    let target_namespaces = if namespaces.is_empty() {
                        discovery.get_all_namespaces().await?
                    } else {
                        namespaces.clone()
                    };

                    let target_resources = if resources.is_empty() {
                        vec!["services".to_string(), "pods".to_string(), "deployments".to_string(), "configmaps".to_string()]
                    } else {
                        resources.clone()
                    };

                    let mut warmed_count = 0;

                    for resource_type in &target_resources {
                        match resource_type.as_str() {
                            "services" => {
                                for namespace in &target_namespaces {
                                    let _ = discovery.list_services_with_options(Some(namespace), None, None, 100, true).await;
                                    warmed_count += 1;
                                }
                            }
                            "pods" => {
                                for namespace in &target_namespaces {
                                    let _ = discovery.list_pods_with_options(Some(namespace), None, None, 100, true).await;
                                    warmed_count += 1;
                                }
                            }
                            "deployments" => {
                                for namespace in &target_namespaces {
                                    let _ = discovery.list_deployments_with_options(Some(namespace), None, 100, true).await;
                                    warmed_count += 1;
                                }
                            }
                            "configmaps" => {
                                for namespace in &target_namespaces {
                                    let _ = discovery.list_configmaps_with_options(Some(namespace), None, 100, true).await;
                                    warmed_count += 1;
                                }
                            }
                            _ => {
                                eprintln!("Warning: Unknown resource type '{}'", resource_type);
                            }
                        }
                    }

                    if let Some(progress) = progress {
                        progress.finish_and_clear();
                    }

                    println!("Cache warmed successfully: {} namespace/resource combinations loaded", warmed_count);
                }
            }
        }
    }

    Ok(())
}

/// Parse group-by string into GroupBy enum
fn parse_group_by(group_by_str: &str) -> GroupBy {
    match group_by_str.to_lowercase().as_str() {
        "app" => GroupBy::App,
        "tier" => GroupBy::Tier,
        "helm-release" | "helm" => GroupBy::HelmRelease,
        "namespace" | "ns" => GroupBy::Namespace,
        "none" => GroupBy::None,
        custom => GroupBy::CustomLabel(custom.to_string()),
    }
}
