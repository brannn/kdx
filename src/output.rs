//! Output formatting for different data types

use crate::cli::OutputFormat;
use crate::discovery::{ServiceInfo, PodInfo, ServiceDescription, ServiceTopology};
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

/// Print service description in the specified format
pub fn print_service_description(description: &ServiceDescription, format: &OutputFormat) -> Result<()> {
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
    
    let rows: Vec<ServiceRow> = services.iter().map(|service| {
        let ports = service.ports.iter()
            .map(|p| {
                let name = p.name.as_ref().map(|s| s.as_str()).unwrap_or("");
                format!("{}:{}/{}", name, p.port, p.protocol)
            })
            .collect::<Vec<_>>()
            .join(",");
            
        ServiceRow {
            name: service.name.clone(),
            namespace: service.namespace.clone(),
            service_type: service.service_type.clone(),
            cluster_ip: service.cluster_ip.clone().unwrap_or_else(|| "None".to_string()),
            ports,
        }
    }).collect();
    
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
        #[tabled(rename = "IP")]
        ip: String,
        #[tabled(rename = "NODE")]
        node: String,
    }
    
    let rows: Vec<PodRow> = pods.iter().map(|pod| {
        let status = match pod.phase.as_str() {
            "Running" => pod.phase.green().to_string(),
            "Pending" => pod.phase.yellow().to_string(),
            "Failed" => pod.phase.red().to_string(),
            "Succeeded" => pod.phase.blue().to_string(),
            _ => pod.phase.clone(),
        };
        
        PodRow {
            name: pod.name.clone(),
            namespace: pod.namespace.clone(),
            status,
            ip: pod.pod_ip.clone().unwrap_or_else(|| "None".to_string()),
            node: pod.node_name.clone().unwrap_or_else(|| "None".to_string()),
        }
    }).collect();
    
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
            let name = port.name.as_ref().map(|s| s.as_str()).unwrap_or("unnamed");
            println!("  {} {}:{} -> {} ({})", 
                name, 
                port.port, 
                port.protocol, 
                port.target_port,
                port.protocol
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
            let prefix = if i == topology.backend_pods.len() - 1 { "    └──" } else { "    ├──" };
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
