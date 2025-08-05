//! Caching system for Kubernetes resource discovery

use crate::discovery::*;
use dashmap::DashMap;
use std::time::{Duration, Instant};

/// Cache entry with TTL support
#[derive(Clone)]
pub struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

/// Resource cache for improving performance
pub struct ResourceCache {
    services: DashMap<String, CacheEntry<Vec<ServiceInfo>>>,
    pods: DashMap<String, CacheEntry<Vec<PodInfo>>>,
    deployments: DashMap<String, CacheEntry<Vec<DeploymentInfo>>>,
    statefulsets: DashMap<String, CacheEntry<Vec<StatefulSetInfo>>>,
    daemonsets: DashMap<String, CacheEntry<Vec<DaemonSetInfo>>>,
    configmaps: DashMap<String, CacheEntry<Vec<ConfigMapInfo>>>,
    secrets: DashMap<String, CacheEntry<Vec<SecretInfo>>>,
    crds: DashMap<String, CacheEntry<Vec<CRDInfo>>>,
    custom_resources: DashMap<String, CacheEntry<Vec<CustomResourceInfo>>>,
    default_ttl: Duration,
}

impl ResourceCache {
    /// Create a new resource cache with default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            services: DashMap::new(),
            pods: DashMap::new(),
            deployments: DashMap::new(),
            statefulsets: DashMap::new(),
            daemonsets: DashMap::new(),
            configmaps: DashMap::new(),
            secrets: DashMap::new(),
            crds: DashMap::new(),
            custom_resources: DashMap::new(),
            default_ttl,
        }
    }

    /// Generate cache key for namespace-scoped resources
    fn namespace_key(namespace: Option<&str>, selector: Option<&str>) -> String {
        match (namespace, selector) {
            (Some(ns), Some(sel)) => format!("{}:{}", ns, sel),
            (Some(ns), None) => ns.to_string(),
            (None, Some(sel)) => format!("all:{}", sel),
            (None, None) => "all".to_string(),
        }
    }

    /// Get services from cache
    pub fn get_services(&self, namespace: Option<&str>, selector: Option<&str>) -> Option<Vec<ServiceInfo>> {
        let key = Self::namespace_key(namespace, selector);
        if let Some(entry) = self.services.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                // Remove expired entry
                self.services.remove(&key);
            }
        }
        None
    }

    /// Set services in cache
    pub fn set_services(&self, namespace: Option<&str>, selector: Option<&str>, data: Vec<ServiceInfo>) {
        let key = Self::namespace_key(namespace, selector);
        let entry = CacheEntry::new(data, self.default_ttl);
        self.services.insert(key, entry);
    }

    /// Get pods from cache
    pub fn get_pods(&self, namespace: Option<&str>, selector: Option<&str>) -> Option<Vec<PodInfo>> {
        let key = Self::namespace_key(namespace, selector);
        if let Some(entry) = self.pods.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.pods.remove(&key);
            }
        }
        None
    }

    /// Set pods in cache
    pub fn set_pods(&self, namespace: Option<&str>, selector: Option<&str>, data: Vec<PodInfo>) {
        let key = Self::namespace_key(namespace, selector);
        let entry = CacheEntry::new(data, self.default_ttl);
        self.pods.insert(key, entry);
    }

    /// Get deployments from cache
    pub fn get_deployments(&self, namespace: Option<&str>) -> Option<Vec<DeploymentInfo>> {
        let key = Self::namespace_key(namespace, None);
        if let Some(entry) = self.deployments.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.deployments.remove(&key);
            }
        }
        None
    }

    /// Set deployments in cache
    pub fn set_deployments(&self, namespace: Option<&str>, data: Vec<DeploymentInfo>) {
        let key = Self::namespace_key(namespace, None);
        let entry = CacheEntry::new(data, self.default_ttl);
        self.deployments.insert(key, entry);
    }

    /// Get statefulsets from cache
    pub fn get_statefulsets(&self, namespace: Option<&str>) -> Option<Vec<StatefulSetInfo>> {
        let key = Self::namespace_key(namespace, None);
        if let Some(entry) = self.statefulsets.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.statefulsets.remove(&key);
            }
        }
        None
    }

    /// Set statefulsets in cache
    pub fn set_statefulsets(&self, namespace: Option<&str>, data: Vec<StatefulSetInfo>) {
        let key = Self::namespace_key(namespace, None);
        let entry = CacheEntry::new(data, self.default_ttl);
        self.statefulsets.insert(key, entry);
    }

    /// Get daemonsets from cache
    pub fn get_daemonsets(&self, namespace: Option<&str>) -> Option<Vec<DaemonSetInfo>> {
        let key = Self::namespace_key(namespace, None);
        if let Some(entry) = self.daemonsets.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.daemonsets.remove(&key);
            }
        }
        None
    }

    /// Set daemonsets in cache
    pub fn set_daemonsets(&self, namespace: Option<&str>, data: Vec<DaemonSetInfo>) {
        let key = Self::namespace_key(namespace, None);
        let entry = CacheEntry::new(data, self.default_ttl);
        self.daemonsets.insert(key, entry);
    }

    /// Get configmaps from cache
    pub fn get_configmaps(&self, namespace: Option<&str>) -> Option<Vec<ConfigMapInfo>> {
        let key = Self::namespace_key(namespace, None);
        if let Some(entry) = self.configmaps.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.configmaps.remove(&key);
            }
        }
        None
    }

    /// Set configmaps in cache
    pub fn set_configmaps(&self, namespace: Option<&str>, data: Vec<ConfigMapInfo>) {
        let key = Self::namespace_key(namespace, None);
        let entry = CacheEntry::new(data, self.default_ttl);
        self.configmaps.insert(key, entry);
    }

    /// Get secrets from cache
    pub fn get_secrets(&self, namespace: Option<&str>) -> Option<Vec<SecretInfo>> {
        let key = Self::namespace_key(namespace, None);
        if let Some(entry) = self.secrets.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.secrets.remove(&key);
            }
        }
        None
    }

    /// Set secrets in cache
    pub fn set_secrets(&self, namespace: Option<&str>, data: Vec<SecretInfo>) {
        let key = Self::namespace_key(namespace, None);
        let entry = CacheEntry::new(data, self.default_ttl);
        self.secrets.insert(key, entry);
    }

    /// Get custom resources from cache
    pub fn get_custom_resources(&self, crd_name: &str, namespace: Option<&str>) -> Option<Vec<CustomResourceInfo>> {
        let key = format!("{}:{}", crd_name, Self::namespace_key(namespace, None));
        if let Some(entry) = self.custom_resources.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.custom_resources.remove(&key);
            }
        }
        None
    }

    /// Set custom resources in cache
    pub fn set_custom_resources(&self, crd_name: &str, namespace: Option<&str>, data: Vec<CustomResourceInfo>) {
        let key = format!("{}:{}", crd_name, Self::namespace_key(namespace, None));
        let entry = CacheEntry::new(data, self.default_ttl);
        self.custom_resources.insert(key, entry);
    }

    /// Get CRDs from cache
    pub fn get_crds(&self) -> Option<Vec<CRDInfo>> {
        let key = "all".to_string();
        if let Some(entry) = self.crds.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data().clone());
            } else {
                self.crds.remove(&key);
            }
        }
        None
    }

    /// Set CRDs in cache
    pub fn set_crds(&self, data: Vec<CRDInfo>) {
        let key = "all".to_string();
        let entry = CacheEntry::new(data, self.default_ttl);
        self.crds.insert(key, entry);
    }

    /// Clear all cached data
    pub fn clear(&self) {
        self.services.clear();
        self.pods.clear();
        self.deployments.clear();
        self.statefulsets.clear();
        self.daemonsets.clear();
        self.configmaps.clear();
        self.secrets.clear();
        self.crds.clear();
        self.custom_resources.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            services_entries: self.services.len(),
            pods_entries: self.pods.len(),
            deployments_entries: self.deployments.len(),
            statefulsets_entries: self.statefulsets.len(),
            daemonsets_entries: self.daemonsets.len(),
            configmaps_entries: self.configmaps.len(),
            secrets_entries: self.secrets.len(),
            crds_entries: self.crds.len(),
            custom_resources_entries: self.custom_resources.len(),
            default_ttl: self.default_ttl,
        }
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        // Clean services
        self.services.retain(|_, entry| !entry.is_expired());
        self.pods.retain(|_, entry| !entry.is_expired());
        self.deployments.retain(|_, entry| !entry.is_expired());
        self.statefulsets.retain(|_, entry| !entry.is_expired());
        self.daemonsets.retain(|_, entry| !entry.is_expired());
        self.configmaps.retain(|_, entry| !entry.is_expired());
        self.secrets.retain(|_, entry| !entry.is_expired());
        self.crds.retain(|_, entry| !entry.is_expired());
        self.custom_resources.retain(|_, entry| !entry.is_expired());
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub services_entries: usize,
    pub pods_entries: usize,
    pub deployments_entries: usize,
    pub statefulsets_entries: usize,
    pub daemonsets_entries: usize,
    pub configmaps_entries: usize,
    pub secrets_entries: usize,
    pub crds_entries: usize,
    pub custom_resources_entries: usize,
    pub default_ttl: Duration,
}

impl CacheStats {
    pub fn total_entries(&self) -> usize {
        self.services_entries
            + self.pods_entries
            + self.deployments_entries
            + self.statefulsets_entries
            + self.daemonsets_entries
            + self.configmaps_entries
            + self.secrets_entries
            + self.crds_entries
            + self.custom_resources_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_test_service() -> ServiceInfo {
        ServiceInfo {
            name: "test-service".to_string(),
            namespace: "default".to_string(),
            service_type: "ClusterIP".to_string(),
            cluster_ip: Some("10.0.0.1".to_string()),
            ports: vec![],
            selector: Some(BTreeMap::new()),
        }
    }

    #[test]
    fn test_cache_entry_expiration() {
        let data = vec![create_test_service()];
        let entry = CacheEntry::new(data, Duration::from_millis(1));
        
        assert!(!entry.is_expired());
        std::thread::sleep(Duration::from_millis(2));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_operations() {
        let cache = ResourceCache::new(Duration::from_secs(300));
        let services = vec![create_test_service()];

        // Test cache miss
        assert!(cache.get_services(Some("default"), None).is_none());

        // Test cache set and hit
        cache.set_services(Some("default"), None, services.clone());
        let cached = cache.get_services(Some("default"), None);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);

        // Test cache stats
        let stats = cache.stats();
        assert_eq!(stats.services_entries, 1);
        assert_eq!(stats.total_entries(), 1);
    }

    #[test]
    fn test_namespace_key_generation() {
        assert_eq!(ResourceCache::namespace_key(Some("default"), None), "default");
        assert_eq!(ResourceCache::namespace_key(None, None), "all");
        assert_eq!(ResourceCache::namespace_key(Some("kube-system"), Some("app=nginx")), "kube-system:app=nginx");
        assert_eq!(ResourceCache::namespace_key(None, Some("app=web")), "all:app=web");
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = ResourceCache::new(Duration::from_millis(1));
        let services = vec![create_test_service()];

        cache.set_services(Some("default"), None, services);
        assert_eq!(cache.stats().services_entries, 1);

        std::thread::sleep(Duration::from_millis(2));
        cache.cleanup_expired();
        assert_eq!(cache.stats().services_entries, 0);
    }
}
