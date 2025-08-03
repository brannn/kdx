//! Kubernetes resource discovery and analysis

use crate::error::{ExplorerError, Result};
use k8s_openapi::api::core::v1::{Pod, Service};
use k8s_openapi::api::networking::v1::Ingress;use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
            ingress_routes: Vec::new(), // TODO: Implement ingress discovery
            dependencies: Vec::new(),   // TODO: Implement dependency analysis
        })
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
