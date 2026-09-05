use crate::action::Action;
use crate::ingest::Alert;
use kube::{Client, api::{Api, DeleteParams}};
use k8s_openapi::api::core::v1::Pod;
use serde_json::json;

pub async fn apply_action(action: &Action, _alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("MOCK_EXECUTOR").unwrap_or_default() == "true" {
        log::info!("[MOCK EXECUTOR] Action '{:?}' executed successfully", action);
        return Ok(());
    }

    let client = Client::try_default().await?;

    match action {
        Action::RestartPod { pod, namespace } => {
            if !pod.is_empty() {
                let pods: Api<Pod> = Api::namespaced(client, namespace);
                pods.delete(pod, &DeleteParams::default()).await?;
            }
        }
        Action::CordonNode { node } => {
            if !node.is_empty() {
                let nodes: Api<k8s_openapi::api::core::v1::Node> = Api::all(client);
                let patch = json!({
                    "spec": {
                        "unschedulable": true
                    }
                });
                let patch_params = kube::api::PatchParams::default();
                nodes.patch(node, &patch_params, &kube::api::Patch::Merge(&patch)).await?;
            }
        }
        _ => {}
    }

    Ok(())
}


