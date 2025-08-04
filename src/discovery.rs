//! Kubernetes resource discovery and analysis

use crate::error::{ExplorerError, Result};
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, StatefulSet};
use k8s_openapi::api::core::v1::{ConfigMap, Pod, Secret, Service};
use k8s_openapi::api::networking::v1::Ingress;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
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
    pub async fn list_pods(
        &self,
        namespace: Option<&str>,
        selector: Option<&str>,
    ) -> Result<Vec<PodInfo>> {
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
    pub async fn describe_service(
        &self,
        name: &str,
        namespace: &str,
    ) -> Result<ServiceDescription> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let service = services
            .get(name)
            .await
            .map_err(|_| ExplorerError::ResourceNotFound {
                kind: "Service".to_string(),
                name: name.to_string(),
                namespace: namespace.to_string(),
            })?;

        // Get basic service info
        let service_info = self.convert_service_to_info(service).await.ok_or_else(|| {
            ExplorerError::ResourceNotFound {
                kind: "Service".to_string(),
                name: name.to_string(),
                namespace: namespace.to_string(),
            }
        })?;

        // Get related pods
        let related_pods = if let Some(selector) = &service_info.selector {
            let selector_string = selector
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            self.list_pods(Some(namespace), Some(&selector_string))
                .await
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(ServiceDescription {
            service: service_info,
            related_pods,
        })
    }

    /// Analyze service topology and relationships
    pub async fn analyze_service_topology(
        &self,
        name: &str,
        namespace: &str,
    ) -> Result<ServiceTopology> {
        let description = self.describe_service(name, namespace).await?;

        // For now, this is a simplified topology
        // In the future, we could add ingress analysis, network policies, etc.
        Ok(ServiceTopology {
            service: description.service,
            backend_pods: description.related_pods,
            ingress_routes: self
                .get_ingress_routes_for_topology(name, namespace)
                .await
                .unwrap_or_default(),
            dependencies: Vec::new(), // Basic dependency analysis could be added here
        })
    }

    async fn get_ingress_routes_for_topology(
        &self,
        service_name: &str,
        namespace: &str,
    ) -> Result<Vec<String>> {
        let ingress_routes = self
            .discover_ingress_for_service(service_name, namespace)
            .await?;
        let route_strings: Vec<String> = ingress_routes
            .iter()
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
    pub async fn discover_ingress_for_service(
        &self,
        service_name: &str,
        namespace: &str,
    ) -> Result<Vec<IngressInfo>> {
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
    pub async fn discover_service_configuration(
        &self,
        service_name: &str,
        namespace: &str,
    ) -> Result<(Vec<ConfigMapInfo>, Vec<SecretInfo>)> {
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
            if let Ok(configmap) = configmaps.get(&config_name).await {
                if let Some(configmap_info) = self.convert_configmap_to_info(configmap).await {
                    found_configmaps.push(configmap_info);
                }
            }

            // Check for Secret
            if let Ok(secret) = secrets.get(&config_name).await {
                if let Some(secret_info) = self.convert_secret_to_info(secret).await {
                    found_secrets.push(secret_info);
                }
            }
        }

        Ok((found_configmaps, found_secrets))
    }

    /// List deployments in the specified namespace (or all namespaces if None)
    pub async fn list_deployments(&self, namespace: Option<&str>) -> Result<Vec<DeploymentInfo>> {
        let deployments: Api<Deployment> = match namespace {
            Some(ns) => Api::namespaced(self.client.clone(), ns),
            None => Api::all(self.client.clone()),
        };

        let deployment_list = deployments.list(&Default::default()).await?;

        let mut deployment_infos = Vec::new();
        for deployment in deployment_list.items {
            if let Some(deployment_info) = self.convert_deployment_to_info(deployment).await {
                deployment_infos.push(deployment_info);
            }
        }

        Ok(deployment_infos)
    }

    /// List statefulsets in the specified namespace (or all namespaces if None)
    pub async fn list_statefulsets(&self, namespace: Option<&str>) -> Result<Vec<StatefulSetInfo>> {
        let statefulsets: Api<StatefulSet> = match namespace {
            Some(ns) => Api::namespaced(self.client.clone(), ns),
            None => Api::all(self.client.clone()),
        };

        let statefulset_list = statefulsets.list(&Default::default()).await?;

        let mut statefulset_infos = Vec::new();
        for statefulset in statefulset_list.items {
            if let Some(statefulset_info) = self.convert_statefulset_to_info(statefulset).await {
                statefulset_infos.push(statefulset_info);
            }
        }

        Ok(statefulset_infos)
    }

    /// List daemonsets in the specified namespace (or all namespaces if None)
    pub async fn list_daemonsets(&self, namespace: Option<&str>) -> Result<Vec<DaemonSetInfo>> {
        let daemonsets: Api<DaemonSet> = match namespace {
            Some(ns) => Api::namespaced(self.client.clone(), ns),
            None => Api::all(self.client.clone()),
        };

        let daemonset_list = daemonsets.list(&Default::default()).await?;

        let mut daemonset_infos = Vec::new();
        for daemonset in daemonset_list.items {
            if let Some(daemonset_info) = self.convert_daemonset_to_info(daemonset).await {
                daemonset_infos.push(daemonset_info);
            }
        }

        Ok(daemonset_infos)
    }

    /// List configmaps in the specified namespace (or all namespaces if None)
    pub async fn list_configmaps(&self, namespace: Option<&str>) -> Result<Vec<ConfigMapInfo>> {
        let configmaps: Api<ConfigMap> = match namespace {
            Some(ns) => Api::namespaced(self.client.clone(), ns),
            None => Api::all(self.client.clone()),
        };

        let configmap_list = configmaps.list(&Default::default()).await?;

        let mut configmap_infos = Vec::new();
        for configmap in configmap_list.items {
            if let Some(configmap_info) = self.convert_configmap_to_info(configmap).await {
                configmap_infos.push(configmap_info);
            }
        }

        // Find associations with other resources
        self.find_configmap_associations(&mut configmap_infos).await?;

        Ok(configmap_infos)
    }

    /// List secrets in the specified namespace (or all namespaces if None)
    pub async fn list_secrets(&self, namespace: Option<&str>) -> Result<Vec<SecretInfo>> {
        let secrets: Api<Secret> = match namespace {
            Some(ns) => Api::namespaced(self.client.clone(), ns),
            None => Api::all(self.client.clone()),
        };

        let secret_list = secrets.list(&Default::default()).await?;

        let mut secret_infos = Vec::new();
        for secret in secret_list.items {
            if let Some(secret_info) = self.convert_secret_to_info(secret).await {
                secret_infos.push(secret_info);
            }
        }

        // Find associations with other resources
        self.find_secret_associations(&mut secret_infos).await?;

        Ok(secret_infos)
    }

    /// Check the health of a service by testing its cluster IP endpoints
    pub async fn check_service_health(
        &self,
        service_name: &str,
        namespace: &str,
    ) -> Result<ServiceHealth> {
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
            checked_at: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        })
    }
    async fn convert_service_to_info(&self, service: Service) -> Option<ServiceInfo> {
        let metadata = service.metadata;
        let spec = service.spec?;

        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());

        let ports = spec
            .ports
            .unwrap_or_default()
            .into_iter()
            .map(|port| ServicePort {
                name: port.name,
                port: port.port,
                target_port: match port.target_port {
                    Some(IntOrString::Int(i)) => i.to_string(),
                    Some(IntOrString::String(s)) => s,
                    None => port.port.to_string(),
                },
                protocol: port.protocol.unwrap_or_else(|| "TCP".to_string()),
            })
            .collect();

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

        let phase = status
            .as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let pod_ip = status.as_ref().and_then(|s| s.pod_ip.clone());

        let node_name = spec.node_name;

        Some(PodInfo {
            name,
            namespace,
            phase,
            pod_ip,
            node_name,
            labels: metadata.labels.unwrap_or_default(),
            ready_containers: 0,        // TODO: Calculate from container statuses
            total_containers: 0,        // TODO: Calculate from spec.containers
            restart_count: 0,           // TODO: Calculate from container statuses
            age: "Unknown".to_string(), // TODO: Calculate from creation timestamp
        })
    }

    async fn convert_ingress_to_info(
        &self,
        ingress: Ingress,
        target_service: &str,
    ) -> Option<IngressInfo> {
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
    }

    async fn convert_deployment_to_info(&self, deployment: Deployment) -> Option<DeploymentInfo> {
        let metadata = deployment.metadata;
        let spec = deployment.spec?;
        let status = deployment.status;

        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        let labels = metadata.labels.unwrap_or_default();

        let replicas = spec.replicas.unwrap_or(1);
        let ready_replicas = status.as_ref().and_then(|s| s.ready_replicas).unwrap_or(0);
        let available_replicas = status.as_ref().and_then(|s| s.available_replicas).unwrap_or(0);

        let strategy = spec.strategy
            .as_ref()
            .and_then(|s| s.type_.as_ref())
            .unwrap_or(&"RollingUpdate".to_string())
            .clone();

        let selector = spec.selector.match_labels.unwrap_or_default();

        Some(DeploymentInfo {
            name,
            namespace,
            replicas,
            ready_replicas,
            available_replicas,
            strategy,
            age: "Unknown".to_string(), // TODO: Calculate from creation timestamp
            labels,
            selector,
        })
    }

    async fn convert_statefulset_to_info(&self, statefulset: StatefulSet) -> Option<StatefulSetInfo> {
        let metadata = statefulset.metadata;
        let spec = statefulset.spec?;
        let status = statefulset.status;

        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        let labels = metadata.labels.unwrap_or_default();

        let replicas = spec.replicas.unwrap_or(1);
        let ready_replicas = status.as_ref().and_then(|s| s.ready_replicas).unwrap_or(0);
        let current_replicas = status.as_ref().and_then(|s| s.current_replicas).unwrap_or(0);

        let selector = spec.selector.match_labels.unwrap_or_default();

        Some(StatefulSetInfo {
            name,
            namespace,
            replicas,
            ready_replicas,
            current_replicas,
            age: "Unknown".to_string(), // TODO: Calculate from creation timestamp
            labels,
            selector,
        })
    }

    async fn convert_daemonset_to_info(&self, daemonset: DaemonSet) -> Option<DaemonSetInfo> {
        let metadata = daemonset.metadata;
        let spec = daemonset.spec?;
        let status = daemonset.status;

        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        let labels = metadata.labels.unwrap_or_default();

        let desired = status.as_ref().map(|s| s.desired_number_scheduled).unwrap_or(0);
        let current = status.as_ref().map(|s| s.current_number_scheduled).unwrap_or(0);
        let ready = status.as_ref().map(|s| s.number_ready).unwrap_or(0);
        let up_to_date = status.as_ref().and_then(|s| s.updated_number_scheduled).unwrap_or(0);

        let selector = spec.selector.match_labels.unwrap_or_default();

        Some(DaemonSetInfo {
            name,
            namespace,
            desired,
            current,
            ready,
            up_to_date,
            age: "Unknown".to_string(), // TODO: Calculate from creation timestamp
            labels,
            selector,
        })
    }

    async fn convert_configmap_to_info(&self, configmap: ConfigMap) -> Option<ConfigMapInfo> {
        let metadata = configmap.metadata;
        let data = configmap.data.unwrap_or_default();

        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        let labels = metadata.labels.unwrap_or_default();

        let data_keys: Vec<String> = data.keys().cloned().collect();

        Some(ConfigMapInfo {
            name,
            namespace,
            data_keys,
            age: "Unknown".to_string(), // TODO: Calculate from creation timestamp
            labels,
            used_by: Vec::new(), // Will be populated by association finding
            mount_paths: Vec::new(), // Will be populated by association finding
        })
    }

    async fn convert_secret_to_info(&self, secret: Secret) -> Option<SecretInfo> {
        let metadata = secret.metadata;
        let data = secret.data.unwrap_or_default();

        let name = metadata.name?;
        let namespace = metadata.namespace.unwrap_or_else(|| "default".to_string());
        let labels = metadata.labels.unwrap_or_default();
        let secret_type = secret.type_.unwrap_or_else(|| "Opaque".to_string());

        let data_keys: Vec<String> = data.keys().cloned().collect();

        Some(SecretInfo {
            name,
            namespace,
            secret_type,
            data_keys,
            age: "Unknown".to_string(), // TODO: Calculate from creation timestamp
            labels,
            used_by: Vec::new(), // Will be populated by association finding
            mount_paths: Vec::new(), // Will be populated by association finding
        })
    }

    async fn find_configmap_associations(&self, configmaps: &mut [ConfigMapInfo]) -> Result<()> {
        // Find all pods that reference these ConfigMaps
        let pods = self.list_pods(None, None).await?;

        for configmap in configmaps.iter_mut() {
            for pod in &pods {
                self.check_pod_configmap_references(pod, configmap);
            }
        }

        Ok(())
    }

    async fn find_secret_associations(&self, secrets: &mut [SecretInfo]) -> Result<()> {
        // Find all pods that reference these Secrets
        let pods = self.list_pods(None, None).await?;

        for secret in secrets.iter_mut() {
            for pod in &pods {
                self.check_pod_secret_references(pod, secret);
            }
        }

        Ok(())
    }

    fn check_pod_configmap_references(&self, pod: &PodInfo, configmap: &mut ConfigMapInfo) {
        // This is a simplified implementation
        // In a real implementation, we would need to access the Pod spec
        // to check for volume mounts and environment variable references

        // For now, we'll add a placeholder reference if the pod is in the same namespace
        if pod.namespace == configmap.namespace {
            let reference = ResourceReference {
                kind: "Pod".to_string(),
                name: pod.name.clone(),
                namespace: pod.namespace.clone(),
                reference_type: ReferenceType::VolumeMount, // Placeholder
            };

            // Only add if not already present
            if !configmap.used_by.iter().any(|r| r.name == reference.name && r.kind == reference.kind) {
                configmap.used_by.push(reference);
            }
        }
    }

    fn check_pod_secret_references(&self, pod: &PodInfo, secret: &mut SecretInfo) {
        // This is a simplified implementation
        // In a real implementation, we would need to access the Pod spec
        // to check for volume mounts, environment variables, and imagePullSecrets

        // For now, we'll add a placeholder reference if the pod is in the same namespace
        if pod.namespace == secret.namespace {
            let reference = ResourceReference {
                kind: "Pod".to_string(),
                name: pod.name.clone(),
                namespace: pod.namespace.clone(),
                reference_type: ReferenceType::VolumeMount, // Placeholder
            };

            // Only add if not already present
            if !secret.used_by.iter().any(|r| r.name == reference.name && r.kind == reference.kind) {
                secret.used_by.push(reference);
            }
        }
    }
}

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
    pub data_keys: Vec<String>,
    pub age: String,
    pub labels: BTreeMap<String, String>,
    pub used_by: Vec<ResourceReference>,
    pub mount_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    pub name: String,
    pub namespace: String,
    pub secret_type: String,
    pub data_keys: Vec<String>,
    pub age: String,
    pub labels: BTreeMap<String, String>,
    pub used_by: Vec<ResourceReference>,
    pub mount_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReference {
    pub kind: String,
    pub name: String,
    pub namespace: String,
    pub reference_type: ReferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    VolumeMount,
    Environment,
    EnvironmentFrom,
    ImagePullSecret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub strategy: String,
    pub age: String,
    pub labels: BTreeMap<String, String>,
    pub selector: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetInfo {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub current_replicas: i32,
    pub age: String,
    pub labels: BTreeMap<String, String>,
    pub selector: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetInfo {
    pub name: String,
    pub namespace: String,
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub up_to_date: i32,
    pub age: String,
    pub labels: BTreeMap<String, String>,
    pub selector: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub namespace: String,
    pub overall_healthy: bool,
    pub checked_at: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_creation() {
        let service = ServiceInfo {
            name: "test-service".to_string(),
            namespace: "default".to_string(),
            ports: vec![],
            cluster_ip: Some("10.0.0.1".to_string()),
            service_type: "ClusterIP".to_string(),
            selector: Some(std::collections::BTreeMap::new()),
        };

        assert_eq!(service.name, "test-service");
        assert_eq!(service.namespace, "default");
        assert_eq!(service.service_type, "ClusterIP");
        assert_eq!(service.cluster_ip, Some("10.0.0.1".to_string()));
    }

    #[test]
    fn test_pod_info_creation() {
        let pod = PodInfo {
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            phase: "Running".to_string(),
            pod_ip: Some("10.0.0.2".to_string()),
            node_name: Some("node1".to_string()),
            labels: std::collections::BTreeMap::new(),
            ready_containers: 2,
            total_containers: 2,
            restart_count: 0,
            age: "1d".to_string(),
        };

        assert_eq!(pod.name, "test-pod");
        assert_eq!(pod.phase, "Running");
        assert_eq!(pod.ready_containers, 2);
        assert_eq!(pod.total_containers, 2);
        assert_eq!(pod.restart_count, 0);
    }

    #[test]
    fn test_ingress_info_creation() {
        let ingress = IngressInfo {
            name: "test-ingress".to_string(),
            namespace: "default".to_string(),
            hosts: vec!["example.com".to_string()],
            paths: vec![],
            tls_enabled: false,
        };

        assert_eq!(ingress.name, "test-ingress");
        assert_eq!(ingress.namespace, "default");
        assert_eq!(ingress.hosts.len(), 1);
        assert!(!ingress.tls_enabled);
    }

    #[test]
    fn test_service_port_creation() {
        let port = ServicePort {
            name: Some("http".to_string()),
            port: 80,
            target_port: "8080".to_string(),
            protocol: "TCP".to_string(),
        };

        assert_eq!(port.name, Some("http".to_string()));
        assert_eq!(port.port, 80);
        assert_eq!(port.target_port, "8080");
        assert_eq!(port.protocol, "TCP");
    }

    #[test]
    fn test_deployment_info_creation() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "web".to_string());
        labels.insert("version".to_string(), "v1.0.0".to_string());

        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "web".to_string());

        let deployment = DeploymentInfo {
            name: "test-deployment".to_string(),
            namespace: "default".to_string(),
            replicas: 3,
            ready_replicas: 2,
            available_replicas: 2,
            strategy: "RollingUpdate".to_string(),
            age: "5d".to_string(),
            labels: labels.clone(),
            selector: selector.clone(),
        };

        assert_eq!(deployment.name, "test-deployment");
        assert_eq!(deployment.namespace, "default");
        assert_eq!(deployment.replicas, 3);
        assert_eq!(deployment.ready_replicas, 2);
        assert_eq!(deployment.available_replicas, 2);
        assert_eq!(deployment.strategy, "RollingUpdate");
        assert_eq!(deployment.age, "5d");
        assert_eq!(deployment.labels.get("app"), Some(&"web".to_string()));
        assert_eq!(deployment.selector.get("app"), Some(&"web".to_string()));
    }

    #[test]
    fn test_statefulset_info_creation() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "database".to_string());

        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "database".to_string());

        let statefulset = StatefulSetInfo {
            name: "test-statefulset".to_string(),
            namespace: "default".to_string(),
            replicas: 3,
            ready_replicas: 3,
            current_replicas: 3,
            age: "10d".to_string(),
            labels: labels.clone(),
            selector: selector.clone(),
        };

        assert_eq!(statefulset.name, "test-statefulset");
        assert_eq!(statefulset.namespace, "default");
        assert_eq!(statefulset.replicas, 3);
        assert_eq!(statefulset.ready_replicas, 3);
        assert_eq!(statefulset.current_replicas, 3);
        assert_eq!(statefulset.age, "10d");
        assert_eq!(statefulset.labels.get("app"), Some(&"database".to_string()));
    }

    #[test]
    fn test_daemonset_info_creation() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "monitoring".to_string());
        labels.insert("component".to_string(), "agent".to_string());

        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "monitoring".to_string());

        let daemonset = DaemonSetInfo {
            name: "test-daemonset".to_string(),
            namespace: "kube-system".to_string(),
            desired: 5,
            current: 5,
            ready: 4,
            up_to_date: 5,
            age: "30d".to_string(),
            labels: labels.clone(),
            selector: selector.clone(),
        };

        assert_eq!(daemonset.name, "test-daemonset");
        assert_eq!(daemonset.namespace, "kube-system");
        assert_eq!(daemonset.desired, 5);
        assert_eq!(daemonset.current, 5);
        assert_eq!(daemonset.ready, 4);
        assert_eq!(daemonset.up_to_date, 5);
        assert_eq!(daemonset.age, "30d");
        assert_eq!(daemonset.labels.get("component"), Some(&"agent".to_string()));
    }

    #[test]
    fn test_deployment_info_serialization() {
        let deployment = DeploymentInfo {
            name: "web-app".to_string(),
            namespace: "production".to_string(),
            replicas: 5,
            ready_replicas: 5,
            available_replicas: 5,
            strategy: "RollingUpdate".to_string(),
            age: "2d".to_string(),
            labels: BTreeMap::new(),
            selector: BTreeMap::new(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&deployment).expect("Failed to serialize to JSON");
        assert!(json.contains("web-app"));
        assert!(json.contains("production"));
        assert!(json.contains("RollingUpdate"));

        // Test deserialization
        let deserialized: DeploymentInfo = serde_json::from_str(&json).expect("Failed to deserialize from JSON");
        assert_eq!(deserialized.name, deployment.name);
        assert_eq!(deserialized.replicas, deployment.replicas);
    }

    #[test]
    fn test_resource_info_with_empty_labels() {
        let deployment = DeploymentInfo {
            name: "minimal-deployment".to_string(),
            namespace: "default".to_string(),
            replicas: 1,
            ready_replicas: 0,
            available_replicas: 0,
            strategy: "Recreate".to_string(),
            age: "1h".to_string(),
            labels: BTreeMap::new(),
            selector: BTreeMap::new(),
        };

        assert!(deployment.labels.is_empty());
        assert!(deployment.selector.is_empty());
        assert_eq!(deployment.ready_replicas, 0);
    }

    #[test]
    fn test_resource_info_with_multiple_labels() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "frontend".to_string());
        labels.insert("tier".to_string(), "web".to_string());
        labels.insert("environment".to_string(), "staging".to_string());
        labels.insert("version".to_string(), "v2.1.0".to_string());

        let statefulset = StatefulSetInfo {
            name: "frontend-statefulset".to_string(),
            namespace: "staging".to_string(),
            replicas: 2,
            ready_replicas: 2,
            current_replicas: 2,
            age: "7d".to_string(),
            labels: labels.clone(),
            selector: labels.clone(),
        };

        assert_eq!(statefulset.labels.len(), 4);
        assert_eq!(statefulset.labels.get("tier"), Some(&"web".to_string()));
        assert_eq!(statefulset.labels.get("environment"), Some(&"staging".to_string()));
        assert_eq!(statefulset.selector.len(), 4);
    }
}
