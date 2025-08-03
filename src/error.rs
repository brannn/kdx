//! Error types for k8s-explorer

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExplorerError {
    #[error("Kubernetes API error: {0}")]
    Kubernetes(#[from] kube::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Resource not found: {kind} '{name}' in namespace '{namespace}'")]
    ResourceNotFound {
        kind: String,
        name: String,
        namespace: String,
    },
    
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
    
    #[error("Output formatting error: {0}")]
    OutputFormat(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ExplorerError>;
