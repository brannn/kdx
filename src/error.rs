//! Error types for k8s-explorer

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExplorerError {
    #[error("Kubernetes API error: {0}")]
    Kubernetes(#[from] kube::Error),
    
    
    #[error("Resource not found: {kind} '{name}' in namespace '{namespace}'")]
    ResourceNotFound {
        kind: String,
        name: String,
        namespace: String,
    },
    
    
    #[error("Output formatting error: {0}")]
    OutputFormat(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ExplorerError>;
