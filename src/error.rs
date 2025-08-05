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

impl From<serde_json::Error> for ExplorerError {
    fn from(err: serde_json::Error) -> Self {
        ExplorerError::OutputFormat(err.to_string())
    }
}

impl From<serde_yaml::Error> for ExplorerError {
    fn from(err: serde_yaml::Error) -> Self {
        ExplorerError::OutputFormat(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_not_found_error_display() {
        let error = ExplorerError::ResourceNotFound {
            kind: "Service".to_string(),
            name: "nginx".to_string(),
            namespace: "default".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Resource not found: Service 'nginx' in namespace 'default'"
        );
    }

    #[test]
    fn test_output_format_error_display() {
        let error = ExplorerError::OutputFormat("Invalid JSON".to_string());
        assert_eq!(error.to_string(), "Output formatting error: Invalid JSON");
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ExplorerError>();
        assert_sync::<ExplorerError>();
    }

    #[test]
    fn test_error_debug_format() {
        let error = ExplorerError::OutputFormat("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("OutputFormat"));
        assert!(debug_str.contains("test"));
    }
}
