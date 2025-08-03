//! Command-line interface definitions

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[clap(name = "kdx", version = "0.1.0")]
#[clap(about = "Kubernetes cluster exploration and discovery tool")]
#[clap(long_about = "A command-line tool for exploring and discovering resources in Kubernetes clusters. Provides easy-to-use commands for listing services, pods, and understanding cluster topology and relationships.")]
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
}

#[derive(Clone, ValueEnum)]
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
