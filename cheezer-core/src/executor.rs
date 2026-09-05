use crate::action::Action;
use crate::ingest::Alert;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Namespace, Node, Pod};
use kube::api::{Api, DeleteParams, Patch, PatchParams};
use kube::Client;
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum ExecutionError {
    KubeError(kube::Error),
    ClientInitError(kube::Error),
    InvalidAction(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::KubeError(e) => write!(f, "Kubernetes API error: {}", e),
            ExecutionError::ClientInitError(e) => write!(f, "Kubernetes client init error: {}", e),
            ExecutionError::InvalidAction(msg) => write!(f, "Invalid action error: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecutionError::KubeError(e) | ExecutionError::ClientInitError(e) => Some(e),
            ExecutionError::InvalidAction(_) => None,
        }
    }
}

impl From<kube::Error> for ExecutionError {
    fn from(err: kube::Error) -> Self {
        ExecutionError::KubeError(err)
    }
}

pub async fn apply_action(action: &Action, _alert: &Alert) -> Result<(), ExecutionError> {
    if std::env::var("MOCK_EXECUTOR").unwrap_or_default() == "true" {
        log::info!("[MOCK EXECUTOR] Action '{:?}' simulated successfully", action);
        return Ok(());
    }

    log::info!("Initializing Kubernetes client for action execution: {}", action);
    let client = Client::try_default().await.map_err(ExecutionError::ClientInitError)?;

    match action {
        Action::RestartPod { pod, namespace } => {
            if pod.is_empty() {
                return Err(ExecutionError::InvalidAction("Target pod name is empty".to_string()));
            }
            let ns = if namespace.is_empty() { "default" } else { namespace.as_str() };
            log::info!("Executing RestartPod via kube-rs: deleting pod '{}' in namespace '{}'", pod, ns);
            let pods: Api<Pod> = Api::namespaced(client, ns);
            pods.delete(pod, &DeleteParams::default()).await?;
            log::info!("Pod '{}' in namespace '{}' deleted successfully via kube API", pod, ns);
        }
        Action::ScaleDeployment { deployment, target_replicas, namespace } => {
            if deployment.is_empty() {
                return Err(ExecutionError::InvalidAction("Target deployment name is empty".to_string()));
            }
            let ns = if namespace.is_empty() { "default" } else { namespace.as_str() };
            log::info!("Executing ScaleDeployment via kube-rs: scaling deployment '{}' in namespace '{}' to {} replicas", deployment, ns, target_replicas);
            let deployments: Api<Deployment> = Api::namespaced(client, ns);
            let patch = json!({
                "spec": {
                    "replicas": target_replicas
                }
            });
            let patch_params = PatchParams::default();
            deployments.patch(deployment, &patch_params, &Patch::Merge(&patch)).await?;
            log::info!("Deployment '{}' scaled to {} replicas successfully via kube API", deployment, target_replicas);
        }
        Action::CordonNode { node } => {
            if node.is_empty() {
                return Err(ExecutionError::InvalidAction("Target node name is empty".to_string()));
            }
            log::info!("Executing CordonNode via kube-rs: cordoning node '{}'", node);
            let nodes: Api<Node> = Api::all(client);
            let patch = json!({
                "spec": {
                    "unschedulable": true
                }
            });
            let patch_params = PatchParams::default();
            nodes.patch(node, &patch_params, &Patch::Merge(&patch)).await?;
            log::info!("Node '{}' cordoned successfully via kube API", node);
        }
        Action::DeleteNamespace { namespace } => {
            if namespace.is_empty() {
                return Err(ExecutionError::InvalidAction("Target namespace name is empty".to_string()));
            }
            log::info!("Executing DeleteNamespace via kube-rs: deleting namespace '{}'", namespace);
            let namespaces: Api<Namespace> = Api::all(client);
            namespaces.delete(namespace, &DeleteParams::default()).await?;
            log::info!("Namespace '{}' deleted successfully via kube API", namespace);
        }
        Action::ExecCommand { pod, command } => {
            log::warn!("ExecCommand target '{}' with command '{:?}' not implemented as direct API call", pod, command);
        }
        Action::ModifyRbac { resource } => {
            log::warn!("ModifyRbac target '{}' not implemented as direct API call", resource);
        }
        Action::LogReviewNeeded { reason } => {
            log::info!("[EXECUTOR NO-ACTION INTENT] Log review needed: {}", reason);
        }
        Action::LogCheckCapacity { reason } => {
            log::info!("[EXECUTOR NO-ACTION INTENT] Check node capacity: {}", reason);
        }
        Action::None => {
            log::info!("[EXECUTOR NO-ACTION INTENT] No operation required");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_executor_k8s_real_dry_run() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("MOCK_EXECUTOR");
        }

        let alert = Alert {
            status: "firing".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
        };

        let action = Action::RestartPod {
            pod: "non-existent-pod-xyz-123".to_string(),
            namespace: "non-existent-namespace-abc-456".to_string(),
        };

        let result = apply_action(&action, &alert).await;
        assert!(result.is_err(), "Expected error when performing k8s operation on non-existent cluster/pod");

        let err = result.unwrap_err();
        println!("Verified error handling in real k8s execution: {}", err);
        match err {
            ExecutionError::KubeError(_) | ExecutionError::ClientInitError(_) | ExecutionError::InvalidAction(_) => {
                // Cleanly caught without process crash
            }
        }
    }
}
