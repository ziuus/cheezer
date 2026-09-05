use crate::ingest::Alert;

pub fn match_rule(alert: &Alert) -> Option<String> {
    let signature = alert.labels.get("alertname").map(|s| s.as_str()).unwrap_or("");
    
    match signature {
        "CrashLoopBackOff" => Some("restart pod".to_string()),
        "OOMKilled" => Some("restart pod".to_string()),
        "ImagePullBackOff" => Some("log manual review needed".to_string()),
        "PodPending" => Some("log check node capacity".to_string()),
        "ProbeFailure" => Some("restart pod".to_string()),
        "NodeNotReady" => Some("cordon node".to_string()),
        _ => None,
    }
}

/// Local Fallback Mode execution for when LLM is unreachable or times out
pub fn execute_fallback(alert: &Alert) -> String {
    if let Some(action) = match_rule(alert) {
        return action;
    }

    // Default safe fallback action for unhandled novel alerts during LLM outage
    "restart pod".to_string()
}

