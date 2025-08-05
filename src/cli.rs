//! Command-line interface definitions

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[clap(name = "kdx", version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Kubernetes cluster exploration and discovery tool")]
#[clap(
    long_about = "A command-line tool for exploring and discovering resources in Kubernetes clusters. Provides easy-to-use commands for listing services, pods, and understanding cluster topology and relationships."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Kubernetes context to use
    #[clap(long, global = true)]
    pub context: Option<String>,

    /// Default namespace to use
    #[clap(long, short = 'n', global = true)]
    pub namespace: Option<String>,

    /// Output format
    #[clap(long, global = true, default_value = "table")]
    pub output: OutputFormat,

    /// Enable verbose logging
    #[clap(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Maximum number of resources to return
    #[clap(long, global = true)]
    pub limit: Option<usize>,

    /// Page size for API requests (default: 100)
    #[clap(long, global = true, default_value = "100")]
    pub page_size: usize,

    /// Show progress for long operations
    #[clap(long, global = true)]
    pub show_progress: bool,

    /// Maximum concurrent operations (default: 10)
    #[clap(long, global = true, default_value = "10")]
    pub concurrency: usize,

    /// Use streaming output for large datasets (JSON/YAML only)
    #[clap(long, global = true)]
    pub stream: bool,

    /// Enable memory optimization for large clusters
    #[clap(long, global = true)]
    pub memory_optimized: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List services in the cluster
    Services {
        /// Show services from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,

        /// Filter by label selector (e.g., app=web,tier!=cache)
        #[clap(long, short = 's')]
        selector: Option<String>,

        /// Group resources by criteria (app, tier, helm-release, namespace)
        #[clap(long, short = 'g')]
        group_by: Option<String>,
    },

    /// List pods in the cluster
    Pods {
        /// Show pods from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Filter by label selector (e.g., app=nginx,version=v1)
        #[clap(long, short = 's')]
        selector: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,

        /// Filter by status (Running, Pending, Failed, Succeeded)
        #[clap(long)]
        status: Option<String>,

        /// Group resources by criteria (app, tier, helm-release, namespace)
        #[clap(long, short = 'g')]
        group_by: Option<String>,
    },

    /// List deployments in the cluster
    Deployments {
        /// Show deployments from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,

        /// Filter by label selector (e.g., app=web,tier!=cache)
        #[clap(long, short = 's')]
        selector: Option<String>,

        /// Filter by status (Ready, NotReady, PartiallyReady)
        #[clap(long)]
        status: Option<String>,

        /// Group resources by criteria (app, tier, helm-release, namespace)
        #[clap(long, short = 'g')]
        group_by: Option<String>,
    },

    /// List statefulsets in the cluster
    Statefulsets {
        /// Show statefulsets from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,
    },

    /// List daemonsets in the cluster
    Daemonsets {
        /// Show daemonsets from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,
    },

    /// List configmaps in the cluster
    Configmaps {
        /// Show configmaps from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,

        /// Filter by label selector (e.g., app=web,tier!=cache)
        #[clap(long, short = 's')]
        selector: Option<String>,

        /// Group resources by criteria (app, tier, helm-release, namespace)
        #[clap(long, short = 'g')]
        group_by: Option<String>,

        /// Show unused configmaps (not referenced by any resource)
        #[clap(long)]
        unused: bool,
    },

    /// List secrets in the cluster
    Secrets {
        /// Show secrets from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,

        /// Filter by label selector (e.g., app=web,tier!=cache)
        #[clap(long, short = 's')]
        selector: Option<String>,

        /// Group resources by criteria (app, tier, helm-release, namespace)
        #[clap(long, short = 'g')]
        group_by: Option<String>,

        /// Show unused secrets (not referenced by any resource)
        #[clap(long)]
        unused: bool,

        /// Filter by secret type (Opaque, kubernetes.io/tls, etc.)
        #[clap(long)]
        secret_type: Option<String>,
    },

    /// List Custom Resource Definitions (CRDs) in the cluster
    Crds {
        /// Filter by label selector (e.g., app=web,tier!=cache)
        #[clap(long, short = 's')]
        selector: Option<String>,

        /// Group resources by criteria (app, tier, helm-release, namespace)
        #[clap(long, short = 'g')]
        group_by: Option<String>,

        /// Show only CRDs with instances
        #[clap(long)]
        with_instances: bool,

        /// Show detailed version information
        #[clap(long)]
        show_versions: bool,
    },

    /// List Custom Resource instances for a specific CRD
    CustomResources {
        /// Name of the CRD to list instances for
        #[clap(value_name = "CRD_NAME")]
        crd_name: String,

        /// Show custom resources from a specific namespace
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Show all namespaces
        #[clap(long, short = 'A')]
        all_namespaces: bool,

        /// Filter by label selector (e.g., app=web,tier!=cache)
        #[clap(long, short = 's')]
        selector: Option<String>,

        /// Group resources by criteria (app, tier, helm-release, namespace)
        #[clap(long, short = 'g')]
        group_by: Option<String>,
    },

    /// Describe a service and its relationships
    Describe {
        /// Service name to describe
        service: String,

        /// Namespace of the service
        #[clap(long, short = 'n')]
        namespace: Option<String>,
    },

    /// Show service topology and relationships
    Topology {
        /// Service name to analyze
        service: String,

        /// Namespace of the service
        #[clap(long, short = 'n')]
        namespace: Option<String>,
    },

    /// Generate a service dependency graph
    Graph {
        /// Namespace to analyze (default: all namespaces)
        #[clap(long, short = 'n')]
        namespace: Option<String>,

        /// Output format for the graph
        #[clap(long, default_value = "dot")]
        format: GraphFormat,

        /// Include pod relationships in the graph
        #[clap(long)]
        include_pods: bool,

        /// Highlight a specific service
        #[clap(long)]
        highlight: Option<String>,
    },

    /// Cache management operations
    Cache {
        #[clap(subcommand)]
        action: CacheAction,
    },

    /// Performance benchmarking and testing
    Benchmark {
        /// Number of iterations to run
        #[clap(long, default_value = "5")]
        iterations: usize,
        /// Resource types to benchmark
        #[clap(long)]
        resources: Vec<String>,
        /// Test memory optimization features
        #[clap(long)]
        test_memory: bool,
        /// Test concurrent discovery
        #[clap(long)]
        test_concurrent: bool,
    },
}

#[derive(Parser)]
pub enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear all cached data
    Clear,
    /// Warm cache by pre-loading common resources
    Warm {
        /// Namespaces to warm (default: all)
        #[clap(long)]
        namespaces: Vec<String>,
        /// Resource types to warm (default: all)
        #[clap(long)]
        resources: Vec<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum GraphFormat {
    /// DOT format (Graphviz)
    Dot,
    /// SVG format
    Svg,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::Table;
        assert!(matches!(format, OutputFormat::Table));
    }

    #[test]
    #[test]
    fn test_output_format_debug() {
        let format = OutputFormat::Table;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Table"));
    }
    #[test]
    fn test_graph_format_default() {
        let format = GraphFormat::Dot;
        assert!(matches!(format, GraphFormat::Dot));
    }

    #[test]
    #[test]
    fn test_graph_format_debug() {
        let format = GraphFormat::Dot;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Dot"));
    }
    #[test]
    fn test_cli_parsing_services() {
        let cli = Cli::try_parse_from(&["kdx", "services"]).unwrap();
        assert!(matches!(cli.command, Commands::Services { .. }));
    }

    #[test]
    fn test_cli_parsing_graph_with_options() {
        let cli = Cli::try_parse_from(&[
            "kdx",
            "graph",
            "--namespace",
            "test",
            "--format",
            "svg",
            "--include-pods",
            "--highlight",
            "nginx",
        ])
        .unwrap();

        if let Commands::Graph {
            namespace,
            format,
            include_pods,
            highlight,
        } = cli.command
        {
            assert_eq!(namespace, Some("test".to_string()));
            assert!(matches!(format, GraphFormat::Svg));
            assert!(include_pods);
            assert_eq!(highlight, Some("nginx".to_string()));
        } else {
            panic!("Expected Graph command");
        }
    }

    #[test]
    fn test_cli_global_options() {
        let cli = Cli::try_parse_from(&[
            "kdx",
            "--verbose",
            "--output",
            "json",
            "--context",
            "test-context",
            "services",
        ])
        .unwrap();

        assert!(cli.verbose);
        assert!(matches!(cli.output, OutputFormat::Json));
        assert_eq!(cli.context, Some("test-context".to_string()));
    }

    #[test]
    fn test_cli_phase2_flags() {
        let args = vec![
            "kdx",
            "services",
            "--limit",
            "50",
            "--page-size",
            "25",
            "--show-progress",
            "--concurrency",
            "5",
            "--stream",
            "--memory-optimized",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.limit, Some(50));
        assert_eq!(cli.page_size, 25);
        assert!(cli.show_progress);
        assert_eq!(cli.concurrency, 5);
        assert!(cli.stream);
        assert!(cli.memory_optimized);
    }

    #[test]
    fn test_cli_cache_commands() {
        // Test cache stats
        let args = vec!["kdx", "cache", "stats"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Cache { action } = cli.command {
            assert!(matches!(action, CacheAction::Stats));
        } else {
            panic!("Expected Cache command");
        }

        // Test cache clear
        let args = vec!["kdx", "cache", "clear"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Cache { action } = cli.command {
            assert!(matches!(action, CacheAction::Clear));
        } else {
            panic!("Expected Cache command");
        }

        // Test cache warm
        let args = vec!["kdx", "cache", "warm", "--namespaces", "default", "--namespaces", "kube-system"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Cache { action } = cli.command {
            if let CacheAction::Warm { namespaces, .. } = action {
                assert_eq!(namespaces, vec!["default", "kube-system"]);
            } else {
                panic!("Expected Warm action");
            }
        } else {
            panic!("Expected Cache command");
        }
    }

    #[test]
    fn test_cli_benchmark_command() {
        let args = vec![
            "kdx",
            "benchmark",
            "--iterations",
            "10",
            "--resources",
            "services",
            "--resources",
            "pods",
            "--test-memory",
            "--test-concurrent",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Benchmark { iterations, resources, test_memory, test_concurrent } = cli.command {
            assert_eq!(iterations, 10);
            assert_eq!(resources, vec!["services", "pods"]);
            assert!(test_memory);
            assert!(test_concurrent);
        } else {
            panic!("Expected Benchmark command");
        }
    }
}
