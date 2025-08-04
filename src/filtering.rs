//! Advanced filtering and grouping capabilities for Kubernetes resources

use crate::discovery::{ConfigMapInfo, DaemonSetInfo, DeploymentInfo, PodInfo, SecretInfo, ServiceInfo, StatefulSetInfo};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Filter criteria for resources
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    /// Label selector expressions (e.g., "app=web,tier!=cache")
    pub label_selector: Option<String>,
    /// Status filter (Running, Pending, Failed, etc.)
    pub status_filter: Option<String>,
    /// Age filter - resources newer than this duration
    pub newer_than: Option<Duration>,
    /// Age filter - resources older than this duration
    pub older_than: Option<Duration>,
    /// Resource type inclusion filter
    pub include_types: Vec<String>,
    /// Resource type exclusion filter
    pub exclude_types: Vec<String>,
}

/// Grouping criteria for resources
#[derive(Debug, Clone)]
pub enum GroupBy {
    /// Group by application label
    App,
    /// Group by tier label
    Tier,
    /// Group by Helm release
    HelmRelease,
    /// Group by namespace
    Namespace,
    /// Group by custom label key
    CustomLabel(String),
    /// No grouping
    None,
}

/// Grouped resource collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedResources {
    pub groups: BTreeMap<String, ResourceGroup>,
}

/// A group of resources with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    pub name: String,
    pub group_type: String,
    pub services: Vec<ServiceInfo>,
    pub pods: Vec<PodInfo>,
    pub deployments: Vec<DeploymentInfo>,
    pub statefulsets: Vec<StatefulSetInfo>,
    pub daemonsets: Vec<DaemonSetInfo>,
    pub configmaps: Vec<ConfigMapInfo>,
    pub secrets: Vec<SecretInfo>,
    pub metadata: BTreeMap<String, String>,
}

impl ResourceGroup {
    pub fn new(name: String, group_type: String) -> Self {
        Self {
            name,
            group_type,
            services: Vec::new(),
            pods: Vec::new(),
            deployments: Vec::new(),
            statefulsets: Vec::new(),
            daemonsets: Vec::new(),
            configmaps: Vec::new(),
            secrets: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn total_resources(&self) -> usize {
        self.services.len()
            + self.pods.len()
            + self.deployments.len()
            + self.statefulsets.len()
            + self.daemonsets.len()
            + self.configmaps.len()
            + self.secrets.len()
    }
}

/// Label selector parser and evaluator
pub struct LabelSelector {
    expressions: Vec<LabelExpression>,
}

#[derive(Debug, Clone)]
enum LabelExpression {
    Equals(String, String),
    NotEquals(String, String),
    In(String, Vec<String>),
    NotIn(String, Vec<String>),
    Exists(String),
    NotExists(String),
}

impl LabelSelector {
    /// Parse a label selector string (e.g., "app=web,tier!=cache,env in (prod,staging)")
    pub fn parse(selector: &str) -> Result<Self, String> {
        let mut expressions = Vec::new();

        // Split by commas, but be careful about commas inside parentheses
        let expr_strings = Self::split_expressions(selector)?;

        for expr in expr_strings {
            let expr = expr.trim();
            if expr.is_empty() {
                continue;
            }

            let expression = if expr.contains(" in ") {
                Self::parse_in_expression(expr, false)?
            } else if expr.contains(" notin ") {
                Self::parse_in_expression(expr, true)?
            } else if expr.contains("!=") {
                let parts: Vec<&str> = expr.splitn(2, "!=").collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid expression: {}", expr));
                }
                let key = parts[0].trim();
                let value = parts[1].trim();
                if key.is_empty() || value.is_empty() {
                    return Err(format!("Invalid expression: {}", expr));
                }
                LabelExpression::NotEquals(key.to_string(), value.to_string())
            } else if expr.contains('=') {
                let parts: Vec<&str> = expr.splitn(2, '=').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid expression: {}", expr));
                }
                let key = parts[0].trim();
                let value = parts[1].trim();
                if key.is_empty() || value.is_empty() {
                    return Err(format!("Invalid expression: {}", expr));
                }
                LabelExpression::Equals(key.to_string(), value.to_string())
            } else {
                // Existence check
                if expr.starts_with('!') {
                    LabelExpression::NotExists(expr[1..].to_string())
                } else {
                    LabelExpression::Exists(expr.to_string())
                }
            };

            expressions.push(expression);
        }

        Ok(Self { expressions })
    }

    /// Split expressions by commas, but respect parentheses
    fn split_expressions(selector: &str) -> Result<Vec<String>, String> {
        let mut expressions = Vec::new();
        let mut current_expr = String::new();
        let mut paren_depth = 0;

        for ch in selector.chars() {
            match ch {
                '(' => {
                    paren_depth += 1;
                    current_expr.push(ch);
                }
                ')' => {
                    paren_depth -= 1;
                    current_expr.push(ch);
                    if paren_depth < 0 {
                        return Err("Unmatched closing parenthesis".to_string());
                    }
                }
                ',' if paren_depth == 0 => {
                    if !current_expr.trim().is_empty() {
                        expressions.push(current_expr.trim().to_string());
                        current_expr.clear();
                    }
                }
                _ => {
                    current_expr.push(ch);
                }
            }
        }

        if paren_depth != 0 {
            return Err("Unmatched opening parenthesis".to_string());
        }

        if !current_expr.trim().is_empty() {
            expressions.push(current_expr.trim().to_string());
        }

        Ok(expressions)
    }

    fn parse_in_expression(expr: &str, not_in: bool) -> Result<LabelExpression, String> {
        let operator = if not_in { " notin " } else { " in " };
        let parts: Vec<&str> = expr.splitn(2, operator).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid {} expression: {}", operator.trim(), expr));
        }

        let key = parts[0].trim().to_string();
        let values_str = parts[1].trim();

        // Parse (value1,value2,value3) format
        if !values_str.starts_with('(') || !values_str.ends_with(')') {
            return Err(format!("Values must be in parentheses: '{}' (full expression: '{}')", values_str, expr));
        }

        let values_inner = &values_str[1..values_str.len()-1];
        let values: Vec<String> = values_inner
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();

        if values.is_empty() {
            return Err(format!("Empty values list in expression: {}", expr));
        }

        Ok(if not_in {
            LabelExpression::NotIn(key, values)
        } else {
            LabelExpression::In(key, values)
        })
    }

    /// Evaluate the selector against a set of labels
    pub fn matches(&self, labels: &BTreeMap<String, String>) -> bool {
        self.expressions.iter().all(|expr| match expr {
            LabelExpression::Equals(key, value) => {
                labels.get(key).map_or(false, |v| v == value)
            }
            LabelExpression::NotEquals(key, value) => {
                labels.get(key).map_or(true, |v| v != value)
            }
            LabelExpression::In(key, values) => {
                labels.get(key).map_or(false, |v| values.contains(v))
            }
            LabelExpression::NotIn(key, values) => {
                labels.get(key).map_or(true, |v| !values.contains(v))
            }
            LabelExpression::Exists(key) => labels.contains_key(key),
            LabelExpression::NotExists(key) => !labels.contains_key(key),
        })
    }
}

/// Resource filtering utilities
pub struct ResourceFilter;

impl ResourceFilter {
    /// Filter services based on criteria
    pub fn filter_services(
        services: Vec<ServiceInfo>,
        criteria: &FilterCriteria,
    ) -> Vec<ServiceInfo> {
        services
            .into_iter()
            .filter(|service| Self::matches_criteria(service, criteria))
            .collect()
    }

    /// Filter deployments based on criteria
    pub fn filter_deployments(
        deployments: Vec<DeploymentInfo>,
        criteria: &FilterCriteria,
    ) -> Vec<DeploymentInfo> {
        deployments
            .into_iter()
            .filter(|deployment| Self::matches_deployment_criteria(deployment, criteria))
            .collect()
    }

    /// Filter pods based on criteria
    pub fn filter_pods(pods: Vec<PodInfo>, criteria: &FilterCriteria) -> Vec<PodInfo> {
        pods.into_iter()
            .filter(|pod| Self::matches_pod_criteria(pod, criteria))
            .collect()
    }

    /// Filter configmaps based on criteria
    pub fn filter_configmaps(
        configmaps: Vec<ConfigMapInfo>,
        criteria: &FilterCriteria,
    ) -> Vec<ConfigMapInfo> {
        configmaps
            .into_iter()
            .filter(|configmap| Self::matches_configmap_criteria(configmap, criteria))
            .collect()
    }

    /// Filter secrets based on criteria
    pub fn filter_secrets(secrets: Vec<SecretInfo>, criteria: &FilterCriteria) -> Vec<SecretInfo> {
        secrets
            .into_iter()
            .filter(|secret| Self::matches_secret_criteria(secret, criteria))
            .collect()
    }

    fn matches_criteria(service: &ServiceInfo, criteria: &FilterCriteria) -> bool {
        // Label selector check
        if let Some(selector_str) = &criteria.label_selector {
            if let Ok(selector) = LabelSelector::parse(selector_str) {
                if let Some(labels) = &service.selector {
                    if !selector.matches(labels) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        // TODO: Add age filtering when we implement proper timestamp parsing
        // TODO: Add status filtering for services

        true
    }

    fn matches_deployment_criteria(deployment: &DeploymentInfo, criteria: &FilterCriteria) -> bool {
        // Label selector check
        if let Some(selector_str) = &criteria.label_selector {
            if let Ok(selector) = LabelSelector::parse(selector_str) {
                if !selector.matches(&deployment.labels) {
                    return false;
                }
            }
        }

        // Status filter (based on replica readiness)
        if let Some(status) = &criteria.status_filter {
            let deployment_status = if deployment.ready_replicas == deployment.replicas {
                "Ready"
            } else if deployment.ready_replicas == 0 {
                "NotReady"
            } else {
                "PartiallyReady"
            };

            if deployment_status != status {
                return false;
            }
        }

        true
    }

    fn matches_pod_criteria(pod: &PodInfo, criteria: &FilterCriteria) -> bool {
        // Label selector check
        if let Some(selector_str) = &criteria.label_selector {
            if let Ok(selector) = LabelSelector::parse(selector_str) {
                if !selector.matches(&pod.labels) {
                    return false;
                }
            }
        }

        // Status filter
        if let Some(status) = &criteria.status_filter {
            if pod.phase != *status {
                return false;
            }
        }

        true
    }

    fn matches_configmap_criteria(configmap: &ConfigMapInfo, criteria: &FilterCriteria) -> bool {
        // Label selector check
        if let Some(selector_str) = &criteria.label_selector {
            if let Ok(selector) = LabelSelector::parse(selector_str) {
                if !selector.matches(&configmap.labels) {
                    return false;
                }
            }
        }

        // TODO: Add age filtering when we implement proper timestamp parsing

        true
    }

    fn matches_secret_criteria(secret: &SecretInfo, criteria: &FilterCriteria) -> bool {
        // Label selector check
        if let Some(selector_str) = &criteria.label_selector {
            if let Ok(selector) = LabelSelector::parse(selector_str) {
                if !selector.matches(&secret.labels) {
                    return false;
                }
            }
        }

        // TODO: Add age filtering when we implement proper timestamp parsing

        true
    }
}

/// Resource grouping utilities
pub struct ResourceGrouper;

impl ResourceGrouper {
    /// Group resources by the specified criteria
    pub fn group_resources(
        services: Vec<ServiceInfo>,
        pods: Vec<PodInfo>,
        deployments: Vec<DeploymentInfo>,
        statefulsets: Vec<StatefulSetInfo>,
        daemonsets: Vec<DaemonSetInfo>,
        group_by: &GroupBy,
    ) -> GroupedResources {
        let mut groups = BTreeMap::new();

        match group_by {
            GroupBy::App => {
                Self::group_by_label(&mut groups, services, pods, deployments, statefulsets, daemonsets, "app");
            }
            GroupBy::Tier => {
                Self::group_by_label(&mut groups, services, pods, deployments, statefulsets, daemonsets, "tier");
            }
            GroupBy::HelmRelease => {
                Self::group_by_helm_release(&mut groups, services, pods, deployments, statefulsets, daemonsets);
            }
            GroupBy::Namespace => {
                Self::group_by_namespace(&mut groups, services, pods, deployments, statefulsets, daemonsets);
            }
            GroupBy::CustomLabel(label_key) => {
                Self::group_by_label(&mut groups, services, pods, deployments, statefulsets, daemonsets, label_key);
            }
            GroupBy::None => {
                let mut group = ResourceGroup::new("All Resources".to_string(), "none".to_string());
                group.services = services;
                group.pods = pods;
                group.deployments = deployments;
                group.statefulsets = statefulsets;
                group.daemonsets = daemonsets;
                groups.insert("all".to_string(), group);
            }
        }

        GroupedResources { groups }
    }

    /// Group configmaps by the specified criteria
    pub fn group_configmaps(configmaps: Vec<ConfigMapInfo>, group_by: &GroupBy) -> GroupedResources {
        let mut groups = BTreeMap::new();

        match group_by {
            GroupBy::App => {
                Self::group_configmaps_by_label(&mut groups, configmaps, "app");
            }
            GroupBy::Tier => {
                Self::group_configmaps_by_label(&mut groups, configmaps, "tier");
            }
            GroupBy::HelmRelease => {
                Self::group_configmaps_by_label(&mut groups, configmaps, "app.kubernetes.io/instance");
            }
            GroupBy::Namespace => {
                Self::group_configmaps_by_namespace(&mut groups, configmaps);
            }
            GroupBy::CustomLabel(label_key) => {
                Self::group_configmaps_by_label(&mut groups, configmaps, label_key);
            }
            GroupBy::None => {
                let mut group = ResourceGroup::new("All ConfigMaps".to_string(), "none".to_string());
                // Note: We'd need to extend ResourceGroup to include configmaps field
                groups.insert("all".to_string(), group);
            }
        }

        GroupedResources { groups }
    }

    /// Group secrets by the specified criteria
    pub fn group_secrets(secrets: Vec<SecretInfo>, group_by: &GroupBy) -> GroupedResources {
        let mut groups = BTreeMap::new();

        match group_by {
            GroupBy::App => {
                Self::group_secrets_by_label(&mut groups, secrets, "app");
            }
            GroupBy::Tier => {
                Self::group_secrets_by_label(&mut groups, secrets, "tier");
            }
            GroupBy::HelmRelease => {
                Self::group_secrets_by_label(&mut groups, secrets, "app.kubernetes.io/instance");
            }
            GroupBy::Namespace => {
                Self::group_secrets_by_namespace(&mut groups, secrets);
            }
            GroupBy::CustomLabel(label_key) => {
                Self::group_secrets_by_label(&mut groups, secrets, label_key);
            }
            GroupBy::None => {
                let mut group = ResourceGroup::new("All Secrets".to_string(), "none".to_string());
                // Note: We'd need to extend ResourceGroup to include secrets field
                groups.insert("all".to_string(), group);
            }
        }

        GroupedResources { groups }
    }

    fn group_by_label(
        groups: &mut BTreeMap<String, ResourceGroup>,
        services: Vec<ServiceInfo>,
        pods: Vec<PodInfo>,
        deployments: Vec<DeploymentInfo>,
        statefulsets: Vec<StatefulSetInfo>,
        daemonsets: Vec<DaemonSetInfo>,
        label_key: &str,
    ) {
        // Group services
        for service in services {
            let group_name = service
                .selector
                .as_ref()
                .and_then(|labels| labels.get(label_key))
                .unwrap_or(&"unknown".to_string())
                .clone();
            
            let group = groups
                .entry(group_name.clone())
                .or_insert_with(|| ResourceGroup::new(group_name, label_key.to_string()));
            group.services.push(service);
        }

        // Group other resources similarly...
        for deployment in deployments {
            let group_name = deployment
                .labels
                .get(label_key)
                .unwrap_or(&"unknown".to_string())
                .clone();
            
            let group = groups
                .entry(group_name.clone())
                .or_insert_with(|| ResourceGroup::new(group_name, label_key.to_string()));
            group.deployments.push(deployment);
        }

        // Continue for other resource types...
        for pod in pods {
            let group_name = pod
                .labels
                .get(label_key)
                .unwrap_or(&"unknown".to_string())
                .clone();
            
            let group = groups
                .entry(group_name.clone())
                .or_insert_with(|| ResourceGroup::new(group_name, label_key.to_string()));
            group.pods.push(pod);
        }

        for statefulset in statefulsets {
            let group_name = statefulset
                .labels
                .get(label_key)
                .unwrap_or(&"unknown".to_string())
                .clone();
            
            let group = groups
                .entry(group_name.clone())
                .or_insert_with(|| ResourceGroup::new(group_name, label_key.to_string()));
            group.statefulsets.push(statefulset);
        }

        for daemonset in daemonsets {
            let group_name = daemonset
                .labels
                .get(label_key)
                .unwrap_or(&"unknown".to_string())
                .clone();
            
            let group = groups
                .entry(group_name.clone())
                .or_insert_with(|| ResourceGroup::new(group_name, label_key.to_string()));
            group.daemonsets.push(daemonset);
        }
    }

    fn group_by_helm_release(
        groups: &mut BTreeMap<String, ResourceGroup>,
        services: Vec<ServiceInfo>,
        pods: Vec<PodInfo>,
        deployments: Vec<DeploymentInfo>,
        statefulsets: Vec<StatefulSetInfo>,
        daemonsets: Vec<DaemonSetInfo>,
    ) {
        // Helm releases are identified by specific labels
        const HELM_RELEASE_LABEL: &str = "app.kubernetes.io/instance";
        const HELM_MANAGED_BY_LABEL: &str = "app.kubernetes.io/managed-by";

        Self::group_by_label(groups, services, pods, deployments, statefulsets, daemonsets, HELM_RELEASE_LABEL);

        // Add Helm metadata to groups
        for group in groups.values_mut() {
            group.metadata.insert("managed-by".to_string(), "Helm".to_string());
        }
    }

    fn group_by_namespace(
        groups: &mut BTreeMap<String, ResourceGroup>,
        services: Vec<ServiceInfo>,
        pods: Vec<PodInfo>,
        deployments: Vec<DeploymentInfo>,
        statefulsets: Vec<StatefulSetInfo>,
        daemonsets: Vec<DaemonSetInfo>,
    ) {
        // Group by namespace
        for service in services {
            let group = groups
                .entry(service.namespace.clone())
                .or_insert_with(|| ResourceGroup::new(service.namespace.clone(), "namespace".to_string()));
            group.services.push(service);
        }

        for deployment in deployments {
            let group = groups
                .entry(deployment.namespace.clone())
                .or_insert_with(|| ResourceGroup::new(deployment.namespace.clone(), "namespace".to_string()));
            group.deployments.push(deployment);
        }

        for pod in pods {
            let group = groups
                .entry(pod.namespace.clone())
                .or_insert_with(|| ResourceGroup::new(pod.namespace.clone(), "namespace".to_string()));
            group.pods.push(pod);
        }

        for statefulset in statefulsets {
            let group = groups
                .entry(statefulset.namespace.clone())
                .or_insert_with(|| ResourceGroup::new(statefulset.namespace.clone(), "namespace".to_string()));
            group.statefulsets.push(statefulset);
        }

        for daemonset in daemonsets {
            let group = groups
                .entry(daemonset.namespace.clone())
                .or_insert_with(|| ResourceGroup::new(daemonset.namespace.clone(), "namespace".to_string()));
            group.daemonsets.push(daemonset);
        }
    }

    fn group_configmaps_by_label(
        groups: &mut BTreeMap<String, ResourceGroup>,
        configmaps: Vec<ConfigMapInfo>,
        label_key: &str,
    ) {
        for configmap in configmaps {
            let group_name = configmap
                .labels
                .get(label_key)
                .unwrap_or(&"unknown".to_string())
                .clone();

            let group = groups
                .entry(group_name.clone())
                .or_insert_with(|| ResourceGroup::new(group_name, label_key.to_string()));
            group.configmaps.push(configmap);
        }
    }

    fn group_configmaps_by_namespace(
        groups: &mut BTreeMap<String, ResourceGroup>,
        configmaps: Vec<ConfigMapInfo>,
    ) {
        for configmap in configmaps {
            let group = groups
                .entry(configmap.namespace.clone())
                .or_insert_with(|| ResourceGroup::new(configmap.namespace.clone(), "namespace".to_string()));
            group.configmaps.push(configmap);
        }
    }

    fn group_secrets_by_label(
        groups: &mut BTreeMap<String, ResourceGroup>,
        secrets: Vec<SecretInfo>,
        label_key: &str,
    ) {
        for secret in secrets {
            let group_name = secret
                .labels
                .get(label_key)
                .unwrap_or(&"unknown".to_string())
                .clone();

            let group = groups
                .entry(group_name.clone())
                .or_insert_with(|| ResourceGroup::new(group_name, label_key.to_string()));
            group.secrets.push(secret);
        }
    }

    fn group_secrets_by_namespace(
        groups: &mut BTreeMap<String, ResourceGroup>,
        secrets: Vec<SecretInfo>,
    ) {
        for secret in secrets {
            let group = groups
                .entry(secret.namespace.clone())
                .or_insert_with(|| ResourceGroup::new(secret.namespace.clone(), "namespace".to_string()));
            group.secrets.push(secret);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_labels() -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "web".to_string());
        labels.insert("tier".to_string(), "frontend".to_string());
        labels.insert("version".to_string(), "v1.0.0".to_string());
        labels.insert("environment".to_string(), "production".to_string());
        labels
    }

    #[test]
    fn test_label_selector_equals() {
        let selector = LabelSelector::parse("app=web").unwrap();
        let labels = create_test_labels();

        assert!(selector.matches(&labels));

        let mut wrong_labels = labels.clone();
        wrong_labels.insert("app".to_string(), "api".to_string());
        assert!(!selector.matches(&wrong_labels));
    }

    #[test]
    fn test_label_selector_not_equals() {
        let selector = LabelSelector::parse("tier!=backend").unwrap();
        let labels = create_test_labels();

        assert!(selector.matches(&labels)); // tier=frontend, not backend

        let mut backend_labels = labels.clone();
        backend_labels.insert("tier".to_string(), "backend".to_string());
        assert!(!selector.matches(&backend_labels));
    }

    #[test]
    fn test_label_selector_in() {
        let selector = LabelSelector::parse("environment in (production,staging)").unwrap();
        let labels = create_test_labels();

        assert!(selector.matches(&labels)); // environment=production

        let mut staging_labels = labels.clone();
        staging_labels.insert("environment".to_string(), "staging".to_string());
        assert!(selector.matches(&staging_labels));

        let mut dev_labels = labels.clone();
        dev_labels.insert("environment".to_string(), "development".to_string());
        assert!(!selector.matches(&dev_labels));
    }

    #[test]
    fn test_label_selector_not_in() {
        let selector = LabelSelector::parse("tier notin (backend,database)").unwrap();
        let labels = create_test_labels();

        assert!(selector.matches(&labels)); // tier=frontend

        let mut backend_labels = labels.clone();
        backend_labels.insert("tier".to_string(), "backend".to_string());
        assert!(!selector.matches(&backend_labels));
    }

    #[test]
    fn test_label_selector_exists() {
        let selector = LabelSelector::parse("app").unwrap();
        let labels = create_test_labels();

        assert!(selector.matches(&labels));

        let mut no_app_labels = labels.clone();
        no_app_labels.remove("app");
        assert!(!selector.matches(&no_app_labels));
    }

    #[test]
    fn test_label_selector_not_exists() {
        let selector = LabelSelector::parse("!database").unwrap();
        let labels = create_test_labels();

        assert!(selector.matches(&labels)); // no database label

        let mut with_database = labels.clone();
        with_database.insert("database".to_string(), "mysql".to_string());
        assert!(!selector.matches(&with_database));
    }

    #[test]
    fn test_label_selector_multiple_expressions() {
        let selector = LabelSelector::parse("app=web,tier=frontend,environment in (production,staging)").unwrap();
        let labels = create_test_labels();

        assert!(selector.matches(&labels));

        let mut wrong_app = labels.clone();
        wrong_app.insert("app".to_string(), "api".to_string());
        assert!(!selector.matches(&wrong_app));
    }

    #[test]
    fn test_label_selector_parse_errors() {
        assert!(LabelSelector::parse("app=").is_err());
        assert!(LabelSelector::parse("=web").is_err());
        assert!(LabelSelector::parse("app in web").is_err()); // missing parentheses
        assert!(LabelSelector::parse("app in ()").is_err()); // empty values
    }

    #[test]
    fn test_filter_criteria_default() {
        let criteria = FilterCriteria::default();
        assert!(criteria.label_selector.is_none());
        assert!(criteria.status_filter.is_none());
        assert!(criteria.newer_than.is_none());
        assert!(criteria.older_than.is_none());
        assert!(criteria.include_types.is_empty());
        assert!(criteria.exclude_types.is_empty());
    }

    #[test]
    fn test_resource_group_creation() {
        let group = ResourceGroup::new("web-app".to_string(), "app".to_string());

        assert_eq!(group.name, "web-app");
        assert_eq!(group.group_type, "app");
        assert_eq!(group.total_resources(), 0);
        assert!(group.services.is_empty());
        assert!(group.pods.is_empty());
        assert!(group.deployments.is_empty());
    }

    #[test]
    fn test_resource_group_total_count() {
        let mut group = ResourceGroup::new("test".to_string(), "app".to_string());

        // Add some mock resources (we'll use empty vecs for simplicity)
        group.services = vec![]; // Would contain ServiceInfo in real usage
        group.pods = vec![]; // Would contain PodInfo in real usage

        assert_eq!(group.total_resources(), 0);
    }

    #[test]
    fn test_grouped_resources_serialization() {
        let mut groups = BTreeMap::new();
        let group = ResourceGroup::new("web".to_string(), "app".to_string());
        groups.insert("web".to_string(), group);

        let grouped = GroupedResources { groups };

        // Test JSON serialization
        let json = serde_json::to_string(&grouped).expect("Failed to serialize to JSON");
        assert!(json.contains("web"));
        assert!(json.contains("app"));

        // Test deserialization
        let deserialized: GroupedResources = serde_json::from_str(&json).expect("Failed to deserialize from JSON");
        assert!(deserialized.groups.contains_key("web"));
    }

    #[test]
    fn test_filter_configmaps() {
        use crate::discovery::{ConfigMapInfo, ResourceReference, ReferenceType};

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "web".to_string());
        labels.insert("tier".to_string(), "frontend".to_string());

        let configmap = ConfigMapInfo {
            name: "web-config".to_string(),
            namespace: "default".to_string(),
            data_keys: vec!["config.yaml".to_string()],
            age: "5d".to_string(),
            labels,
            used_by: vec![ResourceReference {
                kind: "Pod".to_string(),
                name: "web-pod".to_string(),
                namespace: "default".to_string(),
                reference_type: ReferenceType::VolumeMount,
            }],
            mount_paths: vec!["/etc/config".to_string()],
        };

        let configmaps = vec![configmap];

        // Test label selector filtering
        let criteria = FilterCriteria {
            label_selector: Some("app=web".to_string()),
            ..Default::default()
        };
        let filtered = ResourceFilter::filter_configmaps(configmaps.clone(), &criteria);
        assert_eq!(filtered.len(), 1);

        // Test label selector that doesn't match
        let criteria = FilterCriteria {
            label_selector: Some("app=api".to_string()),
            ..Default::default()
        };
        let filtered = ResourceFilter::filter_configmaps(configmaps, &criteria);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_secrets() {
        use crate::discovery::{SecretInfo, ResourceReference, ReferenceType};

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "database".to_string());

        let secret = SecretInfo {
            name: "db-secret".to_string(),
            namespace: "default".to_string(),
            secret_type: "Opaque".to_string(),
            data_keys: vec!["password".to_string(), "username".to_string()],
            age: "10d".to_string(),
            labels,
            used_by: vec![ResourceReference {
                kind: "Pod".to_string(),
                name: "db-pod".to_string(),
                namespace: "default".to_string(),
                reference_type: ReferenceType::Environment,
            }],
            mount_paths: vec![],
        };

        let secrets = vec![secret];

        // Test label selector filtering
        let criteria = FilterCriteria {
            label_selector: Some("app=database".to_string()),
            ..Default::default()
        };
        let filtered = ResourceFilter::filter_secrets(secrets.clone(), &criteria);
        assert_eq!(filtered.len(), 1);

        // Test label selector that doesn't match
        let criteria = FilterCriteria {
            label_selector: Some("app=web".to_string()),
            ..Default::default()
        };
        let filtered = ResourceFilter::filter_secrets(secrets, &criteria);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_group_configmaps_by_app() {
        use crate::discovery::ConfigMapInfo;

        let mut web_labels = BTreeMap::new();
        web_labels.insert("app".to_string(), "web".to_string());

        let mut api_labels = BTreeMap::new();
        api_labels.insert("app".to_string(), "api".to_string());

        let configmaps = vec![
            ConfigMapInfo {
                name: "web-config".to_string(),
                namespace: "default".to_string(),
                data_keys: vec!["config.yaml".to_string()],
                age: "5d".to_string(),
                labels: web_labels,
                used_by: vec![],
                mount_paths: vec![],
            },
            ConfigMapInfo {
                name: "api-config".to_string(),
                namespace: "default".to_string(),
                data_keys: vec!["api.conf".to_string()],
                age: "3d".to_string(),
                labels: api_labels,
                used_by: vec![],
                mount_paths: vec![],
            },
        ];

        let grouped = ResourceGrouper::group_configmaps(configmaps, &GroupBy::App);

        assert_eq!(grouped.groups.len(), 2);
        assert!(grouped.groups.contains_key("web"));
        assert!(grouped.groups.contains_key("api"));

        let web_group = &grouped.groups["web"];
        assert_eq!(web_group.configmaps.len(), 1);
        assert_eq!(web_group.configmaps[0].name, "web-config");
    }

    #[test]
    fn test_group_secrets_by_namespace() {
        use crate::discovery::SecretInfo;

        let secrets = vec![
            SecretInfo {
                name: "secret1".to_string(),
                namespace: "default".to_string(),
                secret_type: "Opaque".to_string(),
                data_keys: vec!["key1".to_string()],
                age: "5d".to_string(),
                labels: BTreeMap::new(),
                used_by: vec![],
                mount_paths: vec![],
            },
            SecretInfo {
                name: "secret2".to_string(),
                namespace: "production".to_string(),
                secret_type: "kubernetes.io/tls".to_string(),
                data_keys: vec!["tls.crt".to_string(), "tls.key".to_string()],
                age: "30d".to_string(),
                labels: BTreeMap::new(),
                used_by: vec![],
                mount_paths: vec![],
            },
        ];

        let grouped = ResourceGrouper::group_secrets(secrets, &GroupBy::Namespace);

        assert_eq!(grouped.groups.len(), 2);
        assert!(grouped.groups.contains_key("default"));
        assert!(grouped.groups.contains_key("production"));

        let prod_group = &grouped.groups["production"];
        assert_eq!(prod_group.secrets.len(), 1);
        assert_eq!(prod_group.secrets[0].secret_type, "kubernetes.io/tls");
    }

    #[test]
    fn test_resource_reference_types() {
        use crate::discovery::ReferenceType;

        // Test serialization of reference types
        let volume_mount = ReferenceType::VolumeMount;
        let environment = ReferenceType::Environment;
        let env_from = ReferenceType::EnvironmentFrom;
        let image_pull = ReferenceType::ImagePullSecret;

        let json_vm = serde_json::to_string(&volume_mount).expect("Failed to serialize VolumeMount");
        let json_env = serde_json::to_string(&environment).expect("Failed to serialize Environment");
        let json_envfrom = serde_json::to_string(&env_from).expect("Failed to serialize EnvironmentFrom");
        let json_imgpull = serde_json::to_string(&image_pull).expect("Failed to serialize ImagePullSecret");

        assert!(json_vm.contains("VolumeMount"));
        assert!(json_env.contains("Environment"));
        assert!(json_envfrom.contains("EnvironmentFrom"));
        assert!(json_imgpull.contains("ImagePullSecret"));
    }
}
