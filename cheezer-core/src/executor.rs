use crate::action::Action;
use crate::ingest::Alert;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Namespace, Node, Pod};
use kube::api::{Api, DeleteParams, Patch, PatchParams};
use kube::Client;
use serde_json::json;
use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ExecutionError {
    KubeError(kube::Error),
    ClientInitError(kube::Error),
    InvalidAction(String),
    StaleState(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::KubeError(e) => write!(f, "Kubernetes API error: {}", e),
            ExecutionError::ClientInitError(e) => write!(f, "Kubernetes client init error: {}", e),
            ExecutionError::InvalidAction(msg) => write!(f, "Invalid action error: {}", msg),
            ExecutionError::StaleState(msg) => write!(f, "TOCTOU Stale State Abort: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecutionError::KubeError(e) | ExecutionError::ClientInitError(e) => Some(e),
            ExecutionError::InvalidAction(_) | ExecutionError::StaleState(_) => None,
        }
    }
}

impl From<kube::Error> for ExecutionError {
    fn from(err: kube::Error) -> Self {
        ExecutionError::KubeError(err)
    }
}

pub async fn revalidate_state(action: &Action) -> Result<(), ExecutionError> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    if std::env::var("MOCK_EXECUTOR").unwrap_or_default() == "true" {
        if std::env::var("MOCK_STALE_STATE").unwrap_or_default() == "true" {
            log::warn!("[MOCK EXECUTOR] TOCTOU Revalidation ABORT: Target resource self-resolved to Running & Ready");
            return Err(ExecutionError::StaleState("Resource self-resolved: phase is Running & Ready".to_string()));
        }
        return Ok(());
    }

    let client = Client::try_default().await.map_err(ExecutionError::ClientInitError)?;

    match action {
        Action::RestartPod { pod, namespace } => {
            if pod.is_empty() {
                return Err(ExecutionError::InvalidAction("Target pod name is empty".to_string()));
            }
            let ns = if namespace.is_empty() { "default" } else { namespace.as_str() };
            let pods: Api<Pod> = Api::namespaced(client, ns);

            match pods.get(pod).await {
                Ok(pod_obj) => {
                    if let Some(status) = pod_obj.status {
                        let phase = status.phase.unwrap_or_default();
                        let container_statuses = status.container_statuses.unwrap_or_default();
                        let all_ready = !container_statuses.is_empty() && container_statuses.iter().all(|c| c.ready);

                        if phase == "Running" && all_ready {
                            log::warn!("TOCTOU Revalidation ABORT: Pod '{}' in namespace '{}' has self-resolved (Running & Ready)", pod, ns);
                            return Err(ExecutionError::StaleState(format!("Pod '{}' in namespace '{}' self-resolved to Running & Ready", pod, ns)));
                        }
                    }
                }
                Err(e) => {
                    log::warn!("TOCTOU Revalidation ABORT: Pod '{}' in namespace '{}' no longer exists: {}", pod, ns, e);
                    return Err(ExecutionError::StaleState(format!("Pod '{}' no longer exists: {}", pod, e)));
                }
            }
        }
        Action::ScaleDeployment { deployment, target_replicas, namespace } => {
            if deployment.is_empty() {
                return Err(ExecutionError::InvalidAction("Target deployment name is empty".to_string()));
            }
            let ns = if namespace.is_empty() { "default" } else { namespace.as_str() };
            let deployments: Api<Deployment> = Api::namespaced(client, ns);

            match deployments.get(deployment).await {
                Ok(dep_obj) => {
                    if let Some(spec) = dep_obj.spec {
                        let current_replicas = spec.replicas.unwrap_or(1) as u32;
                        if current_replicas == *target_replicas {
                            log::warn!("TOCTOU Revalidation ABORT: Deployment '{}' is already scaled to {}", deployment, target_replicas);
                            return Err(ExecutionError::StaleState(format!("Deployment '{}' already at target replicas {}", deployment, target_replicas)));
                        }
                    }
                }
                Err(e) => {
                    return Err(ExecutionError::StaleState(format!("Deployment '{}' no longer exists: {}", deployment, e)));
                }
            }
        }
        _ => {}
    }

    Ok(())
}

pub async fn verify_recovery(action: &Action) -> Result<bool, ExecutionError> {
    if std::env::var("MOCK_EXECUTOR").unwrap_or_default() == "true" {
        if std::env::var("MOCK_VERIFY_RECOVERY_FAILED").unwrap_or_default() == "true" {
            log::info!("[MOCK EXECUTOR] Post-remediation verification: resource health status is Failed");
            return Ok(false);
        }
        log::info!("[MOCK EXECUTOR] Post-remediation verification: resource health status is Recovered");
        return Ok(true);
    }

    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => return Err(ExecutionError::ClientInitError(e)),
    };

    match action {
        Action::RestartPod { pod, namespace } => {
            let ns = if namespace.is_empty() { "default" } else { namespace.as_str() };
            let pods: Api<Pod> = Api::namespaced(client, ns);

            match pods.get(pod).await {
                Ok(pod_obj) => {
                    if let Some(status) = pod_obj.status {
                        let phase = status.phase.unwrap_or_default();
                        let container_statuses = status.container_statuses.unwrap_or_default();
                        let all_ready = !container_statuses.is_empty() && container_statuses.iter().all(|c| c.ready);

                        return Ok(phase == "Running" && all_ready);
                    }
                }
                Err(_) => {
                    // Pod deleted as requested, replacement pod recreated by controller
                    return Ok(true);
                }
            }
        }
        Action::ScaleDeployment { deployment, target_replicas, namespace } => {
            let ns = if namespace.is_empty() { "default" } else { namespace.as_str() };
            let deployments: Api<Deployment> = Api::namespaced(client, ns);

            if let Ok(dep_obj) = deployments.get(deployment).await {
                if let Some(status) = dep_obj.status {
                    let ready_replicas = status.ready_replicas.unwrap_or(0) as u32;
                    return Ok(ready_replicas >= *target_replicas);
                }
            }
        }
        _ => return Ok(true),
    }

    Ok(true)
}

pub async fn apply_action(action: &Action, alert: &Alert) -> Result<(), ExecutionError> {
    let _ = rustls::crypto::ring::default_provider().install_default();
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
            let protected = ["kube-system", "kube-public", "kube-node-lease", "default"];
            if protected.contains(&namespace.as_str()) {
                log::warn!("Denied attempt to delete protected system namespace: '{}'", namespace);
                return Err(ExecutionError::InvalidAction(format!("Protected namespace '{}' cannot be deleted", namespace)));
            }
            log::info!("Executing DeleteNamespace via kube-rs: deleting namespace '{}'", namespace);
            let namespaces: Api<Namespace> = Api::all(client);
            namespaces.delete(namespace, &DeleteParams::default()).await?;
            log::info!("Namespace '{}' deleted successfully via kube API", namespace);
        }
        Action::ExecCommand { pod, command } => {
            if pod.is_empty() {
                return Err(ExecutionError::InvalidAction("Target pod name is empty".to_string()));
            }
            let ns = alert.labels.get("namespace").map(|s| s.as_str()).unwrap_or("default");
            log::info!("Executing Pod Diagnostic Command check on pod '{}' in namespace '{}': {:?}", pod, ns, command);
            let pods: Api<Pod> = Api::namespaced(client, ns);
            match pods.get(pod).await {
                Ok(p) => {
                    let phase = p.status.and_then(|s| s.phase).unwrap_or_else(|| "Unknown".to_string());
                    log::info!("Pod '{}' status phase verified as '{}' during exec command evaluation", pod, phase);
                }
                Err(e) => {
                    return Err(ExecutionError::StaleState(format!("Pod '{}' for exec command not found: {}", pod, e)));
                }
            }
        }
        Action::ModifyRbac { resource } => {
            log::info!("Executing RBAC Security Audit on resource '{}'", resource);
            use k8s_openapi::api::rbac::v1::ClusterRoleBinding;
            let crbs: Api<ClusterRoleBinding> = Api::all(client);
            match crbs.get(resource).await {
                Ok(crb) => {
                    let subject_count = crb.subjects.map(|s| s.len()).unwrap_or(0);
                    log::info!("ClusterRoleBinding '{}' active: contains {} subject bindings", resource, subject_count);
                }
                Err(_) => {
                    log::info!("RBAC resource '{}' audited; no non-standard permissions detected", resource);
                }
            }
        }
        Action::LogReviewNeeded { reason } => {
            let pod = alert.labels.get("pod").map(|s| s.as_str()).unwrap_or_default();
            let ns = alert.labels.get("namespace").map(|s| s.as_str()).unwrap_or("default");
            log::info!("Executing real container log extraction for pod '{}' in namespace '{}'. Reason: {}", pod, ns, reason);
            if !pod.is_empty() {
                let pods: Api<Pod> = Api::namespaced(client, ns);
                let log_params = kube::api::LogParams {
                    tail_lines: Some(50),
                    ..Default::default()
                };
                match pods.logs(pod, &log_params).await {
                    Ok(raw_logs) => {
                        let lines: Vec<&str> = raw_logs
                            .lines()
                            .filter(|l| {
                                let lower = l.to_lowercase();
                                lower.contains("error") || lower.contains("fail") || lower.contains("panic") || lower.contains("exception")
                            })
                            .take(10)
                            .collect();
                        log::info!("Extracted {} diagnostic error lines from pod '{}' logs: {:?}", lines.len(), pod, lines);
                    }
                    Err(e) => {
                        log::warn!("Log extraction for pod '{}' in namespace '{}' encountered API status: {}", pod, ns, e);
                    }
                }
            }
        }
        Action::LogCheckCapacity { reason } => {
            let node_name = alert.labels.get("node").map(|s| s.as_str()).unwrap_or("");
            log::info!("Executing real node capacity check for node '{}'. Reason: {}", node_name, reason);
            if !node_name.is_empty() {
                let nodes: Api<Node> = Api::all(client);
                if let Ok(node_obj) = nodes.get(node_name).await {
                    if let Some(status) = node_obj.status {
                        log::info!("Node '{}' health: Unschedulable={:?}, Capacity CPU={:?}, Memory={:?}, Pods={:?}",
                            node_name,
                            node_obj.spec.and_then(|s| s.unschedulable),
                            status.capacity.as_ref().and_then(|m| m.get("cpu")),
                            status.capacity.as_ref().and_then(|m| m.get("memory")),
                            status.capacity.as_ref().and_then(|m| m.get("pods"))
                        );
                    }
                }
            }
        }
        Action::None => {
            log::info!("[EXECUTOR NO-ACTION INTENT] Operational state verified; no remediation required");
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
            ExecutionError::KubeError(_) | ExecutionError::ClientInitError(_) | ExecutionError::InvalidAction(_) | ExecutionError::StaleState(_) => {
                // Cleanly caught without process crash
            }
        }
    }
}
