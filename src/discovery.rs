//! Kubernetes resource discovery and analysis

use crate::error::{ExplorerError, Result};
use k8s_openapi::api::core::v1::{Pod, Service, ConfigMap, Secret};
use k8s_openapi::api::networking::v1::Ingress;use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use chrono;
/// Main discovery engine for Kubernetes resources
pub struct DiscoveryEngine {
    client: Client,
}

impl DiscoveryEngine {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
    
    /// List services in the specified namespace (or all namespaces if None)
    pub async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<ServiceInfo>> {
        let services: Api<Service> = match namespace {
            Some(ns) => Api::namespaced(self.client.clone(), ns),
            None => Api::all(self.client.clone()),
        };
        
        let service_list = services.list(&Default::default()).await?;
        
        let mut service_infos = Vec::new();
        for service in service_list.items {
            if let Some(service_info) = self.convert_service_to_info(service).await {
                service_infos.push(service_info);
            }
        }
        
        Ok(service_infos)
    }
    
    /// List pods in the specified namespace with optional label selector
    pub async fn list_pods(&self, namespace: Option<&str>, selector: Option<&str>) -> Result<Vec<PodInfo>> {
        let pods: Api<Pod> = match namespace {
            Some(ns) => Api::namespaced(self.client.clone(), ns),
            None => Api::all(self.client.clone()),
        };
        
        let mut list_params = kube::api::ListParams::default();
        if let Some(sel) = selector {
            list_params = list_params.labels(sel);
        }
        
        let pod_list = pods.list(&list_params).await?;
        
        let mut pod_infos = Vec::new();
        for pod in pod_list.items {
            if let Some(pod_info) = self.convert_pod_to_info(pod).await {
                pod_infos.push(pod_info);
            }
        }
        
        Ok(pod_infos)
    }
    
    /// Get detailed information about a specific service
    pub async fn describe_service(&self, name: &str, namespace: &str) -> Result<ServiceDescription> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let service = services.get(name).await.map_err(|_| ExplorerError::ResourceNotFound {
            kind: "Service".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
        })?;
        
        // Get basic service info
        let service_info = self.convert_service_to_info(service).await
            .ok_or_else(|| ExplorerError::ResourceNotFound {
                kind: "Service".to_string(),
                name: name.to_string(),
                namespace: namespace.to_string(),
            })?;
        
        // Get related pods
        let related_pods = if let Some(selector) = &service_info.selector {
            let selector_string = selector.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            self.list_pods(Some(namespace), Some(&selector_string)).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        
        Ok(ServiceDescription {
            service: service_info,
            related_pods,
        })
    }
    
    /// Analyze service topology and relationships
    pub async fn analyze_service_topology(&self, name: &str, namespace: &str) -> Result<ServiceTopology> {
        let description = self.describe_service(name, namespace).await?;
        
        // For now, this is a simplified topology
        // In the future, we could add ingress analysis, network policies, etc.
        Ok(ServiceTopology {
            service: description.service,
            backend_pods: description.related_pods,
            ingress_routes: self.get_ingress_routes_for_topology(name, namespace).await.unwrap_or_default(),
            dependencies: Vec::new(), // Basic dependency analysis could be added here
        })
    }

    async fn get_ingress_routes_for_topology(&self, service_name: &str, namespace: &str) -> Result<Vec<String>> {
        let ingress_routes = self.discover_ingress_for_service(service_name, namespace).await?;
        let route_strings: Vec<String> = ingress_routes.iter()
            .flat_map(|ingress| {
                ingress.hosts.iter().map(|host| {
                    if ingress.tls_enabled {
                        format!("https://{}", host)
                    } else {
                        format!("http://{}", host)
                    }
                })
            })
            .collect();
        Ok(route_strings)
    }
    /// Discover ingress resources that route to a specific service
    pub async fn discover_ingress_for_service(&self, service_name: &str, namespace: &str) -> Result<Vec<IngressInfo>> {
        let ingresses: Api<Ingress> = Api::namespaced(self.client.clone(), namespace);
        
        let ingress_list = ingresses.list(&Default::default()).await?;
        
        let mut matching_ingresses = Vec::new();
        
        for ingress in ingress_list.items {
            if let Some(ingress_info) = self.convert_ingress_to_info(ingress, service_name).await {
                matching_ingresses.push(ingress_info);
            }
        }
        
        Ok(matching_ingresses)
    }    

    /// Discover ConfigMaps and Secrets used by a service (placeholder implementation)
    pub async fn discover_service_configuration(&self, service_name: &str, namespace: &str) -> Result<(Vec<ConfigMapInfo>, Vec<SecretInfo>)> {
        // Basic implementation: check if common ConfigMaps and Secrets exist
        let configmaps: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        
        let mut found_configmaps = Vec::new();
        let mut found_secrets = Vec::new();
        
        // Look for common configuration patterns
        let common_config_names = vec![
            service_name.to_string(),
            format!("{}-config", service_name),
            format!("{}-configuration", service_name),
        ];
        
        for config_name in common_config_names {
            // Check for ConfigMap
            if let Ok(_) = configmaps.get(&config_name).await {
                found_configmaps.push(ConfigMapInfo {
                    name: config_name.clone(),
                    namespace: namespace.to_string(),
                    mount_path: None, // Would need pod analysis to determine
                });
            }
            
            // Check for Secret
            if let Ok(_) = secrets.get(&config_name).await {
                found_secrets.push(SecretInfo {
                    name: config_name,
                    namespace: namespace.to_string(),
                    mount_path: None, // Would need pod analysis to determine
                    secret_type: "Opaque".to_string(),
                });
            }
        }
        
        Ok((found_configmaps, found_secrets))    }

    /// Check the health of a service by testing its cluster IP endpoints
    pub async fn check_service_health(&self, service_name: &str, namespace: &str) -> Result<ServiceHealth> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let service = services.get(service_name).await?;
        
        let mut overall_healthy = false;
        
        if let Some(spec) = service.spec {
            if let Some(cluster_ip) = spec.cluster_ip {
                if cluster_ip != "None" && !cluster_ip.is_empty() && cluster_ip != "ClusterIP" {
                    // For simplicity, just check if the service has a valid cluster IP
                    // In a real implementation, we could try HTTP requests to the endpoints
                    overall_healthy = true;
                } else {
                    // Service exists but has no accessible IP
                    overall_healthy = false;
                }
            }
        }
        
        Ok(ServiceHealth {
            service_name: service_name.to_string(),
            namespace: namespace.to_string(),
            overall_healthy,
            checked_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        })
    }
    async fn convert_service_to_info(&self, service: Service) -> Option<ServiceInfo> {
        let metadata = service.metadata;
        let spec = service.spec?;
        
        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        
        let ports = spec.ports.unwrap_or_default().into_iter().map(|port| {
            ServicePort {
                name: port.name,
                port: port.port,
                target_port: match port.target_port {
                    Some(IntOrString::Int(i)) => i.to_string(),
                    Some(IntOrString::String(s)) => s,
                    None => port.port.to_string(),
                },
                protocol: port.protocol.unwrap_or_else(|| "TCP".to_string()),
            }
        }).collect();
        
        Some(ServiceInfo {
            name,
            namespace,
            ports,
            cluster_ip: spec.cluster_ip,
            service_type: spec.type_.unwrap_or_else(|| "ClusterIP".to_string()),
            selector: spec.selector,
        })
    }
    
    async fn convert_pod_to_info(&self, pod: Pod) -> Option<PodInfo> {
        let metadata = pod.metadata;
        let spec = pod.spec?;
        let status = pod.status;
        
        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        
        let phase = status.as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string());
            
        let pod_ip = status.as_ref()
            .and_then(|s| s.pod_ip.clone());
            
        let node_name = spec.node_name;
        
        Some(PodInfo {
            name,
            namespace,
            phase,
            pod_ip,
            node_name,
            labels: metadata.labels.unwrap_or_default(),
            ready_containers: 0, // TODO: Calculate from container statuses
            total_containers: 0, // TODO: Calculate from spec.containers
            restart_count: 0,    // TODO: Calculate from container statuses
            age: "Unknown".to_string(), // TODO: Calculate from creation timestamp
        })
    }

    async fn convert_ingress_to_info(&self, ingress: Ingress, target_service: &str) -> Option<IngressInfo> {
        let metadata = ingress.metadata;
        let spec = ingress.spec?;
        
        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        
        let mut hosts = Vec::new();
        let mut paths = Vec::new();
        let mut service_found = false;
        
        // Check if this ingress routes to our target service
        if let Some(rules) = spec.rules {
            for rule in rules {
                if let Some(host) = rule.host {
                    hosts.push(host);
                }
                
                if let Some(http) = rule.http {
                    for path in http.paths {
                        if let Some(backend) = path.backend.service {
                            if backend.name == target_service {
                                service_found = true;
                                let port_str = match backend.port {
                                    Some(port) => {
                                        if let Some(number) = port.number {
                                            number.to_string()
                                        } else if let Some(name) = port.name {
                                            name
                                        } else {
                                            "unknown".to_string()
                                        }
                                    }
                                    None => "unknown".to_string(),
                                };
                                
                                paths.push(IngressPath {
                                    path: path.path.unwrap_or_else(|| "/".to_string()),
                                    service_name: backend.name,
                                    service_port: port_str,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Only return ingress info if it actually routes to our target service
        if service_found {
            Some(IngressInfo {
                name,
                namespace,
                hosts,
                paths,
                tls_enabled: spec.tls.is_some() && !spec.tls.unwrap().is_empty(),
            })
        } else {
            None
        }
    }}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub ports: Vec<ServicePort>,
    pub cluster_ip: Option<String>,
    pub service_type: String,
    pub selector: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub port: i32,
    pub target_port: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub pod_ip: Option<String>,
    pub node_name: Option<String>,
    pub labels: BTreeMap<String, String>,
    pub ready_containers: u32,
    pub total_containers: u32,
    pub restart_count: u32,
        pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressInfo {
    pub name: String,
    pub namespace: String,
    pub hosts: Vec<String>,
    pub paths: Vec<IngressPath>,
    pub tls_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPath {
    pub path: String,
    pub service_name: String,
    pub service_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapInfo {
    pub name: String,
    pub namespace: String,
    pub mount_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    pub name: String,
    pub namespace: String,
    pub mount_path: Option<String>,
    pub secret_type: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub namespace: String,
    pub overall_healthy: bool,
    pub checked_at: String,
}#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescription {
    pub service: ServiceInfo,
    pub related_pods: Vec<PodInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTopology {
    pub service: ServiceInfo,
    pub backend_pods: Vec<PodInfo>,
    pub ingress_routes: Vec<String>, // TODO: Define proper ingress types
    pub dependencies: Vec<String>,   // TODO: Define proper dependency types
}
