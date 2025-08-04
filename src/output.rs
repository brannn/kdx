//! Output formatting for different data types

use crate::cli::OutputFormat;
use crate::discovery::{
    ConfigMapInfo, DaemonSetInfo, DeploymentInfo, IngressInfo, PodInfo, SecretInfo,
    ServiceDescription, ServiceHealth, ServiceInfo, ServiceTopology, StatefulSetInfo,
};
use crate::error::{ExplorerError, Result};
use colored::*;
use tabled::{Table, Tabled};

/// Print services in the specified format
pub fn print_services(services: &[ServiceInfo], format: &OutputFormat) -> Result<()> {
    if services.is_empty() {
        println!("No services found");
        return Ok(());
    }

    match format {
        OutputFormat::Table => print_services_table(services),
        OutputFormat::Json => print_json(&services)?,
        OutputFormat::Yaml => print_yaml(&services)?,
    }

    Ok(())
}

/// Print pods in the specified format
pub fn print_pods(pods: &[PodInfo], format: &OutputFormat) -> Result<()> {
    if pods.is_empty() {
        println!("No pods found");
        return Ok(());
    }

    match format {
        OutputFormat::Table => print_pods_table(pods),
        OutputFormat::Json => print_json(&pods)?,
        OutputFormat::Yaml => print_yaml(&pods)?,
    }

    Ok(())
}

/// Print deployments in the specified format
pub fn print_deployments(deployments: &[DeploymentInfo], format: &OutputFormat) -> Result<()> {
    if deployments.is_empty() {
        println!("No deployments found");
        return Ok(());
    }

    match format {
        OutputFormat::Table => print_deployments_table(deployments),
        OutputFormat::Json => print_json(&deployments)?,
        OutputFormat::Yaml => print_yaml(&deployments)?,
    }

    Ok(())
}

/// Print statefulsets in the specified format
pub fn print_statefulsets(statefulsets: &[StatefulSetInfo], format: &OutputFormat) -> Result<()> {
    if statefulsets.is_empty() {
        println!("No statefulsets found");
        return Ok(());
    }

    match format {
        OutputFormat::Table => print_statefulsets_table(statefulsets),
        OutputFormat::Json => print_json(&statefulsets)?,
        OutputFormat::Yaml => print_yaml(&statefulsets)?,
    }

    Ok(())
}

/// Print daemonsets in the specified format
pub fn print_daemonsets(daemonsets: &[DaemonSetInfo], format: &OutputFormat) -> Result<()> {
    if daemonsets.is_empty() {
        println!("No daemonsets found");
        return Ok(());
    }

    match format {
        OutputFormat::Table => print_daemonsets_table(daemonsets),
        OutputFormat::Json => print_json(&daemonsets)?,
        OutputFormat::Yaml => print_yaml(&daemonsets)?,
    }

    Ok(())
}

/// Print service description in the specified format
pub fn print_service_description(
    description: &ServiceDescription,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Table => print_service_description_table(description),
        OutputFormat::Json => print_json(&description)?,
        OutputFormat::Yaml => print_yaml(&description)?,
    }

    Ok(())
}

/// Print service topology in the specified format
pub fn print_service_topology(topology: &ServiceTopology, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => print_service_topology_table(topology),
        OutputFormat::Json => print_json(&topology)?,
        OutputFormat::Yaml => print_yaml(&topology)?,
    }

    Ok(())
}

fn print_services_table(services: &[ServiceInfo]) {
    #[derive(Tabled)]
    struct ServiceRow {
        #[tabled(rename = "NAME")]
        name: String,
        #[tabled(rename = "NAMESPACE")]
        namespace: String,
        #[tabled(rename = "TYPE")]
        service_type: String,
        #[tabled(rename = "CLUSTER-IP")]
        cluster_ip: String,
        #[tabled(rename = "PORTS")]
        ports: String,
    }

    let rows: Vec<ServiceRow> = services
        .iter()
        .map(|service| {
            let ports = service
                .ports
                .iter()
                .map(|p| {
                    let name = p.name.as_deref().unwrap_or("");
                    format!("{}:{}/{}", name, p.port, p.protocol)
                })
                .collect::<Vec<_>>()
                .join(",");

            ServiceRow {
                name: service.name.clone(),
                namespace: service.namespace.clone(),
                service_type: service.service_type.clone(),
                cluster_ip: service
                    .cluster_ip
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
                ports,
            }
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
}

fn print_pods_table(pods: &[PodInfo]) {
    #[derive(Tabled)]
    struct PodRow {
        #[tabled(rename = "NAME")]
        name: String,
        #[tabled(rename = "NAMESPACE")]
        namespace: String,
        #[tabled(rename = "STATUS")]
        status: String,
        #[tabled(rename = "READY")]
        ready: String,
        #[tabled(rename = "RESTARTS")]
        restarts: u32,
        #[tabled(rename = "AGE")]
        age: String,
        #[tabled(rename = "IP")]
        ip: String,
        #[tabled(rename = "NODE")]
        node: String,
    }
    let rows: Vec<PodRow> = pods
        .iter()
        .map(|pod| {
            // Use plain text for table alignment - colors mess up column widths
            let status = pod.phase.clone();
            PodRow {
                name: pod.name.clone(),
                namespace: pod.namespace.clone(),
                status,
                ready: format!("{}/{}", pod.ready_containers, pod.total_containers),
                restarts: pod.restart_count,
                age: pod.age.clone(),
                ip: pod.pod_ip.clone().unwrap_or_else(|| "None".to_string()),
                node: pod.node_name.clone().unwrap_or_else(|| "None".to_string()),
            }
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
}

fn print_deployments_table(deployments: &[DeploymentInfo]) {
    #[derive(Tabled)]
    struct DeploymentRow {
        #[tabled(rename = "NAME")]
        name: String,
        #[tabled(rename = "NAMESPACE")]
        namespace: String,
        #[tabled(rename = "READY")]
        ready: String,
        #[tabled(rename = "UP-TO-DATE")]
        up_to_date: String,
        #[tabled(rename = "AVAILABLE")]
        available: String,
        #[tabled(rename = "STRATEGY")]
        strategy: String,
        #[tabled(rename = "AGE")]
        age: String,
    }

    let rows: Vec<DeploymentRow> = deployments
        .iter()
        .map(|d| DeploymentRow {
            name: d.name.clone(),
            namespace: d.namespace.clone(),
            ready: format!("{}/{}", d.ready_replicas, d.replicas),
            up_to_date: d.ready_replicas.to_string(),
            available: d.available_replicas.to_string(),
            strategy: d.strategy.clone(),
            age: d.age.clone(),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
}

fn print_statefulsets_table(statefulsets: &[StatefulSetInfo]) {
    #[derive(Tabled)]
    struct StatefulSetRow {
        #[tabled(rename = "NAME")]
        name: String,
        #[tabled(rename = "NAMESPACE")]
        namespace: String,
        #[tabled(rename = "READY")]
        ready: String,
        #[tabled(rename = "CURRENT")]
        current: String,
        #[tabled(rename = "AGE")]
        age: String,
    }

    let rows: Vec<StatefulSetRow> = statefulsets
        .iter()
        .map(|s| StatefulSetRow {
            name: s.name.clone(),
            namespace: s.namespace.clone(),
            ready: format!("{}/{}", s.ready_replicas, s.replicas),
            current: s.current_replicas.to_string(),
            age: s.age.clone(),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
}

fn print_daemonsets_table(daemonsets: &[DaemonSetInfo]) {
    #[derive(Tabled)]
    struct DaemonSetRow {
        #[tabled(rename = "NAME")]
        name: String,
        #[tabled(rename = "NAMESPACE")]
        namespace: String,
        #[tabled(rename = "DESIRED")]
        desired: String,
        #[tabled(rename = "CURRENT")]
        current: String,
        #[tabled(rename = "READY")]
        ready: String,
        #[tabled(rename = "UP-TO-DATE")]
        up_to_date: String,
        #[tabled(rename = "AGE")]
        age: String,
    }

    let rows: Vec<DaemonSetRow> = daemonsets
        .iter()
        .map(|d| DaemonSetRow {
            name: d.name.clone(),
            namespace: d.namespace.clone(),
            desired: d.desired.to_string(),
            current: d.current.to_string(),
            ready: d.ready.to_string(),
            up_to_date: d.up_to_date.to_string(),
            age: d.age.clone(),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
}

fn print_service_description_table(description: &ServiceDescription) {
    let service = &description.service;

    println!("{}", format!("Service: {}", service.name).bold());
    println!("Namespace: {}", service.namespace);
    println!("Type: {}", service.service_type);

    if let Some(cluster_ip) = &service.cluster_ip {
        println!("Cluster IP: {}", cluster_ip);
    }

    if !service.ports.is_empty() {
        println!("\nPorts:");
        for port in &service.ports {
            let name = port.name.as_deref().unwrap_or("unnamed");
            println!(
                "  {} {}:{} -> {} ({})",
                name, port.port, port.protocol, port.target_port, port.protocol
            );
        }
    }

    if let Some(selector) = &service.selector {
        println!("\nSelector:");
        for (key, value) in selector {
            println!("  {} = {}", key, value);
        }
    }

    if !description.related_pods.is_empty() {
        println!("\nRelated Pods:");
        print_pods_table(&description.related_pods);
    }
}

fn print_service_topology_table(topology: &ServiceTopology) {
    let service = &topology.service;

    println!("{}", format!("Service Topology: {}", service.name).bold());
    println!("├── Namespace: {}", service.namespace);
    println!("├── Type: {}", service.service_type);

    if let Some(cluster_ip) = &service.cluster_ip {
        println!("├── Cluster IP: {}", cluster_ip);
    }

    if !topology.backend_pods.is_empty() {
        println!("└── Backend Pods:");
        for (i, pod) in topology.backend_pods.iter().enumerate() {
            let prefix = if i == topology.backend_pods.len() - 1 {
                "    └──"
            } else {
                "    ├──"
            };
            let status_color = match pod.phase.as_str() {
                "Running" => pod.phase.green(),
                "Pending" => pod.phase.yellow(),
                "Failed" => pod.phase.red(),
                _ => pod.phase.normal(),
            };
            println!("{} {} ({})", prefix, pod.name, status_color);
        }
    }

    // TODO: Add ingress routes and dependencies when implemented
}

fn print_json<T: serde::Serialize + ?Sized>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| ExplorerError::OutputFormat(format!("JSON serialization failed: {}", e)))?;
    println!("{}", json);
    Ok(())
}

fn print_yaml<T: serde::Serialize + ?Sized>(data: &T) -> Result<()> {
    let yaml = serde_yaml::to_string(data)
        .map_err(|e| ExplorerError::OutputFormat(format!("YAML serialization failed: {}", e)))?;
    println!("{}", yaml);
    Ok(())
}

/// Print ingress information in the specified format
pub fn print_ingress_info(ingress_routes: &[IngressInfo], format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => print_ingress_table(ingress_routes),
        OutputFormat::Json => print_json(&ingress_routes)?,
        OutputFormat::Yaml => print_yaml(&ingress_routes)?,
    }

    Ok(())
}

fn print_ingress_table(ingress_routes: &[IngressInfo]) {
    if ingress_routes.is_empty() {
        return;
    }

    println!("\nIngress Routes:");
    for ingress in ingress_routes {
        println!(
            "  Ingress: {} (namespace: {})",
            ingress.name.cyan(),
            ingress.namespace
        );

        if !ingress.hosts.is_empty() {
            println!("    Hosts: {}", ingress.hosts.join(", "));
        }

        if ingress.tls_enabled {
            println!("    TLS: {}", "Enabled".green());
        }

        if !ingress.paths.is_empty() {
            println!("    Paths:");
            for path in &ingress.paths {
                println!(
                    "      {} -> {}:{}",
                    path.path.yellow(),
                    path.service_name,
                    path.service_port
                );
            }
        }
        println!();
    }
}

/// Print configuration information (ConfigMaps and Secrets) in the specified format
pub fn print_configuration_info(
    configmaps: &[ConfigMapInfo],
    secrets: &[SecretInfo],
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Table => print_configuration_table(configmaps, secrets),
        OutputFormat::Json => {
            let config = serde_json::json!({
                "configmaps": configmaps,
                "secrets": secrets
            });
            print_json(&config)?;
        }
        OutputFormat::Yaml => {
            let config = serde_json::json!({
                "configmaps": configmaps,
                "secrets": secrets
            });
            print_yaml(&config)?;
        }
    }

    Ok(())
}

fn print_configuration_table(configmaps: &[ConfigMapInfo], secrets: &[SecretInfo]) {
    if configmaps.is_empty() && secrets.is_empty() {
        return;
    }

    println!("\nConfiguration:");

    if !configmaps.is_empty() {
        println!("  ConfigMaps:");
        for cm in configmaps {
            let mount_info = cm
                .mount_path
                .as_ref()
                .map(|path| format!(" (mounted at {})", path))
                .unwrap_or_else(|| " (environment variable)".to_string());
            println!(
                "    {} (namespace: {}){}",
                cm.name.cyan(),
                cm.namespace,
                mount_info
            );
        }
    }

    if !secrets.is_empty() {
        println!("  Secrets:");
        for secret in secrets {
            let mount_info = secret
                .mount_path
                .as_ref()
                .map(|path| format!(" (mounted at {})", path))
                .unwrap_or_else(|| " (environment variable)".to_string());
            println!(
                "    {} (namespace: {}, type: {}){}",
                secret.name.yellow(),
                secret.namespace,
                secret.secret_type,
                mount_info
            );
        }
    }
}

/// Print health information in the specified format
pub fn print_health_info(health: &ServiceHealth, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => print_health_table(health),
        OutputFormat::Json => print_json(&health)?,
        OutputFormat::Yaml => print_yaml(&health)?,
    }

    Ok(())
}

fn print_health_table(health: &ServiceHealth) {
    println!("\nHealth Status:");

    let status_color = if health.overall_healthy {
        "Healthy".green()
    } else {
        "Unhealthy".red()
    };

    println!("  Status: {}", status_color);
    println!("  Checked at: {}", health.checked_at);

    if !health.overall_healthy {
        println!("  Note: Service may not be accessible or may not have a valid cluster IP");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_test_deployment() -> DeploymentInfo {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "test-app".to_string());

        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "test-app".to_string());

        DeploymentInfo {
            name: "test-deployment".to_string(),
            namespace: "default".to_string(),
            replicas: 3,
            ready_replicas: 2,
            available_replicas: 2,
            strategy: "RollingUpdate".to_string(),
            age: "5d".to_string(),
            labels,
            selector,
        }
    }

    fn create_test_statefulset() -> StatefulSetInfo {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "database".to_string());

        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "database".to_string());

        StatefulSetInfo {
            name: "test-statefulset".to_string(),
            namespace: "default".to_string(),
            replicas: 3,
            ready_replicas: 3,
            current_replicas: 3,
            age: "10d".to_string(),
            labels,
            selector,
        }
    }

    fn create_test_daemonset() -> DaemonSetInfo {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "monitoring".to_string());

        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "monitoring".to_string());

        DaemonSetInfo {
            name: "test-daemonset".to_string(),
            namespace: "kube-system".to_string(),
            desired: 5,
            current: 5,
            ready: 4,
            up_to_date: 5,
            age: "30d".to_string(),
            labels,
            selector,
        }
    }

    #[test]
    fn test_print_deployments_json() {
        let deployments = vec![create_test_deployment()];
        let result = print_deployments(&deployments, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_deployments_yaml() {
        let deployments = vec![create_test_deployment()];
        let result = print_deployments(&deployments, &OutputFormat::Yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_deployments_table() {
        let deployments = vec![create_test_deployment()];
        let result = print_deployments(&deployments, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_empty_deployments() {
        let deployments = vec![];
        let result = print_deployments(&deployments, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_statefulsets_json() {
        let statefulsets = vec![create_test_statefulset()];
        let result = print_statefulsets(&statefulsets, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_statefulsets_yaml() {
        let statefulsets = vec![create_test_statefulset()];
        let result = print_statefulsets(&statefulsets, &OutputFormat::Yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_statefulsets_table() {
        let statefulsets = vec![create_test_statefulset()];
        let result = print_statefulsets(&statefulsets, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_empty_statefulsets() {
        let statefulsets = vec![];
        let result = print_statefulsets(&statefulsets, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_daemonsets_json() {
        let daemonsets = vec![create_test_daemonset()];
        let result = print_daemonsets(&daemonsets, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_daemonsets_yaml() {
        let daemonsets = vec![create_test_daemonset()];
        let result = print_daemonsets(&daemonsets, &OutputFormat::Yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_daemonsets_table() {
        let daemonsets = vec![create_test_daemonset()];
        let result = print_daemonsets(&daemonsets, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_empty_daemonsets() {
        let daemonsets = vec![];
        let result = print_daemonsets(&daemonsets, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_deployments_output() {
        let deployments = vec![
            create_test_deployment(),
            {
                let mut deployment = create_test_deployment();
                deployment.name = "second-deployment".to_string();
                deployment.namespace = "production".to_string();
                deployment.replicas = 5;
                deployment
            }
        ];

        // Test all output formats with multiple deployments
        assert!(print_deployments(&deployments, &OutputFormat::Table).is_ok());
        assert!(print_deployments(&deployments, &OutputFormat::Json).is_ok());
        assert!(print_deployments(&deployments, &OutputFormat::Yaml).is_ok());
    }

    #[test]
    fn test_resource_with_zero_replicas() {
        let mut deployment = create_test_deployment();
        deployment.replicas = 0;
        deployment.ready_replicas = 0;
        deployment.available_replicas = 0;

        let deployments = vec![deployment];
        let result = print_deployments(&deployments, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resource_with_high_replica_count() {
        let mut statefulset = create_test_statefulset();
        statefulset.replicas = 100;
        statefulset.ready_replicas = 95;
        statefulset.current_replicas = 98;

        let statefulsets = vec![statefulset];
        let result = print_statefulsets(&statefulsets, &OutputFormat::Json);
        assert!(result.is_ok());
    }
}
