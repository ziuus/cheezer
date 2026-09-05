use crate::ingest::Alert;
use kube::{Client, api::{Api, DeleteParams}};
use k8s_openapi::api::core::v1::Pod;
use serde_json::json;

pub async fn apply_action(action: &str, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("MOCK_EXECUTOR").unwrap_or_default() == "true" {
        log::info!("[MOCK EXECUTOR] Action '{}' executed successfully", action);
        return Ok(());
    }

    let client = Client::try_default().await?;
    let namespace = alert.labels.get("namespace").map(|s| s.as_str()).unwrap_or("default");
    let pod_name = alert.labels.get("pod").map(|s| s.as_str()).unwrap_or("");

    if action == "restart pod" && !pod_name.is_empty() {
        let pods: Api<Pod> = Api::namespaced(client, namespace);
        pods.delete(pod_name, &DeleteParams::default()).await?;
    } else if action == "cordon node" {
        let node_name = alert.labels.get("node").map(|s| s.as_str()).unwrap_or("");
        if !node_name.is_empty() {
            let nodes: Api<k8s_openapi::api::core::v1::Node> = Api::all(client);
            let patch = json!({
                "spec": {
                    "unschedulable": true
                }
            });
            let patch_params = kube::api::PatchParams::default();
            nodes.patch(node_name, &patch_params, &kube::api::Patch::Merge(&patch)).await?;
        }
    }
    
    Ok(())
}

