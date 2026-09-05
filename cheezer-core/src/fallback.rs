use crate::action::Action;
use crate::ingest::Alert;

pub fn match_rule(alert: &Alert) -> Option<Action> {
    let signature = alert.labels.get("alertname").map(|s| s.as_str()).unwrap_or("");
    let pod = alert.labels.get("pod").cloned().unwrap_or_else(|| "default-pod".to_string());
    let namespace = alert.labels.get("namespace").cloned().unwrap_or_else(|| "default".to_string());
    let node = alert.labels.get("node").cloned().unwrap_or_else(|| "default-node".to_string());

    match signature {
        "CrashLoopBackOff" => Some(Action::RestartPod { pod, namespace }),
        "OOMKilled" => Some(Action::RestartPod { pod, namespace }),
        "ImagePullBackOff" => Some(Action::LogReviewNeeded { reason: "ImagePullBackOff credential/tag review required".to_string() }),
        "PodPending" => Some(Action::LogCheckCapacity { reason: "Pod stuck in Pending state".to_string() }),
        "ProbeFailure" => Some(Action::RestartPod { pod, namespace }),
        "NodeNotReady" => Some(Action::CordonNode { node }),
        _ => None,
    }
}

/// Local Fallback Mode execution for when LLM is unreachable or times out
pub fn execute_fallback(alert: &Alert) -> Action {
    if let Some(action) = match_rule(alert) {
        return action;
    }

    let pod = alert.labels.get("pod").cloned().unwrap_or_else(|| "fallback-pod".to_string());
    let namespace = alert.labels.get("namespace").cloned().unwrap_or_else(|| "default".to_string());

    Action::RestartPod { pod, namespace }
}


