use crate::discovery::{DiscoveryEngine, IngressInfo, PodInfo, ServiceInfo};
use crate::error::Result;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::Graph;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct ServiceNode {
    pub name: String,
    pub namespace: String,
    pub node_type: NodeType,
    pub is_highlighted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Service,
    Pod,
    Ingress,
}

#[derive(Debug, Clone)]
pub struct ServiceEdge {
    pub relationship: EdgeType,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    ServiceToPod,
    IngressToService,
}

pub struct ServiceGraph {
    graph: UnGraph<ServiceNode, ServiceEdge>,
    node_map: HashMap<String, NodeIndex>,
}

impl ServiceGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new_undirected(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_service_node(&mut self, service: &ServiceInfo, is_highlighted: bool) -> NodeIndex {
        let node_id = format!("service:{}:{}", service.namespace, service.name);

        if let Some(&existing_idx) = self.node_map.get(&node_id) {
            return existing_idx;
        }

        let node = ServiceNode {
            name: service.name.clone(),
            namespace: service.namespace.clone(),
            node_type: NodeType::Service,
            is_highlighted,
        };

        let idx = self.graph.add_node(node);
        self.node_map.insert(node_id, idx);
        idx
    }

    pub fn add_pod_node(&mut self, pod: &PodInfo) -> NodeIndex {
        let node_id = format!("pod:{}:{}", pod.namespace, pod.name);

        if let Some(&existing_idx) = self.node_map.get(&node_id) {
            return existing_idx;
        }

        let node = ServiceNode {
            name: pod.name.clone(),
            namespace: pod.namespace.clone(),
            node_type: NodeType::Pod,
            is_highlighted: false,
        };

        let idx = self.graph.add_node(node);
        self.node_map.insert(node_id, idx);
        idx
    }

    pub fn add_ingress_node(&mut self, ingress: &IngressInfo) -> NodeIndex {
        let node_id = format!("ingress:{}:{}", ingress.namespace, ingress.name);

        if let Some(&existing_idx) = self.node_map.get(&node_id) {
            return existing_idx;
        }

        let node = ServiceNode {
            name: ingress.name.clone(),
            namespace: ingress.namespace.clone(),
            node_type: NodeType::Ingress,
            is_highlighted: false,
        };

        let idx = self.graph.add_node(node);
        self.node_map.insert(node_id, idx);
        idx
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, edge_type: EdgeType) {
        let edge = ServiceEdge {
            relationship: edge_type,
        };
        self.graph.add_edge(from, to, edge);
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        writeln!(dot, "graph ServiceDependencies {{").unwrap();
        writeln!(dot, "  rankdir=TB;").unwrap();
        writeln!(dot, "  node [shape=box, style=rounded];").unwrap();
        writeln!(dot).unwrap();

        // Add nodes
        for node_idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(node_idx) {
                let (shape, color, style) = match node.node_type {
                    NodeType::Service => (
                        "box",
                        if node.is_highlighted {
                            "red"
                        } else {
                            "lightblue"
                        },
                        "filled",
                    ),
                    NodeType::Pod => ("ellipse", "lightgreen", "filled"),
                    NodeType::Ingress => ("diamond", "orange", "filled"),
                };

                writeln!(
                    dot,
                    "  \"{}\" [label=\"{}\\n({})\", shape={}, fillcolor={}, style=\"{}\"];",
                    node_idx.index(),
                    node.name,
                    node.namespace,
                    shape,
                    color,
                    style
                )
                .unwrap();
            }
        }

        writeln!(dot).unwrap();

        // Add edges
        for edge_idx in self.graph.edge_indices() {
            if let Some((from, to)) = self.graph.edge_endpoints(edge_idx) {
                if let Some(edge) = self.graph.edge_weight(edge_idx) {
                    let (style, label) = match edge.relationship {
                        EdgeType::ServiceToPod => ("solid", "manages"),
                        EdgeType::IngressToService => ("bold", "exposes"),
                    };

                    writeln!(
                        dot,
                        "  \"{}\" -- \"{}\" [style={}, label=\"{}\"];",
                        from.index(),
                        to.index(),
                        style,
                        label
                    )
                    .unwrap();
                }
            }
        }

        writeln!(dot, "}}").unwrap();
        dot
    }

    pub fn to_svg(&self) -> Result<String> {
        // For now, we'll generate DOT and suggest using Graphviz to convert to SVG
        let dot = self.to_dot();
        Ok(format!(
            "<!-- SVG generation requires Graphviz. Use: echo '{}' | dot -Tsvg -->\n{}",
            dot.replace('\n', "\\n"),
            dot
        ))
    }
}

pub async fn generate_service_graph(
    discovery: &DiscoveryEngine,
    namespace: Option<&str>,
    include_pods: bool,
    highlight_service: Option<&str>,
) -> Result<ServiceGraph> {
    let mut graph = ServiceGraph::new();

    // Get services using the correct method name
    let services = discovery.list_services(namespace).await?;

    // Add service nodes
    let mut service_nodes = HashMap::new();
    for service in &services {
        let is_highlighted = highlight_service
            .map(|h| h == service.name)
            .unwrap_or(false);
        let node_idx = graph.add_service_node(service, is_highlighted);
        service_nodes.insert(format!("{}:{}", service.namespace, service.name), node_idx);
    }

    // Add pod relationships if requested
    if include_pods {
        for service in &services {
            // Get pods for this service (simplified - in reality we'd use selectors)
            if let Ok(pods) = discovery.list_pods(Some(&service.namespace), None).await {
                for pod in pods {
                    let pod_idx = graph.add_pod_node(&pod);
                    if let Some(&service_idx) =
                        service_nodes.get(&format!("{}:{}", service.namespace, service.name))
                    {
                        graph.add_edge(service_idx, pod_idx, EdgeType::ServiceToPod);
                    }
                }
            }
        }
    }

    // Add ingress relationships
    for service in &services {
        if let Ok(ingresses) = discovery
            .discover_ingress_for_service(&service.name, &service.namespace)
            .await
        {
            for ingress in ingresses {
                let ingress_idx = graph.add_ingress_node(&ingress);
                if let Some(&service_idx) =
                    service_nodes.get(&format!("{}:{}", service.namespace, service.name))
                {
                    graph.add_edge(ingress_idx, service_idx, EdgeType::IngressToService);
                }
            }
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_graph_creation() {
        let graph = ServiceGraph::new();
        assert_eq!(graph.graph.node_count(), 0);
        assert_eq!(graph.graph.edge_count(), 0);
        assert!(graph.node_map.is_empty());
    }

    #[test]
    fn test_node_type_debug() {
        let service_type = NodeType::Service;
        let debug_str = format!("{:?}", service_type);
        assert_eq!(debug_str, "Service");
    }

    #[test]
    fn test_edge_type_debug() {
        let edge_type = EdgeType::ServiceToPod;
        let debug_str = format!("{:?}", edge_type);
        assert_eq!(debug_str, "ServiceToPod");
    }

    #[test]
    fn test_service_node_creation() {
        let node = ServiceNode {
            name: "test".to_string(),
            namespace: "default".to_string(),
            node_type: NodeType::Service,
            is_highlighted: false,
        };
        assert_eq!(node.name, "test");
        assert_eq!(node.namespace, "default");
        assert!(!node.is_highlighted);
    }

    #[test]
    fn test_service_edge_creation() {
        let edge = ServiceEdge {
            relationship: EdgeType::ServiceToPod,
        };
        assert!(matches!(edge.relationship, EdgeType::ServiceToPod));
    }
}
