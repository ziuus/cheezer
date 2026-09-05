use crate::action::Action;
use crate::store;

#[derive(Debug, PartialEq, Eq)]
pub enum GuardResult {
    Allow,
    Block(String),
}

pub struct RemediationGuard;

impl RemediationGuard {
    pub fn evaluate(incident_id: i64, resource: &str, _action: &Action) -> GuardResult {
        if resource.is_empty() || resource == "unknown" {
            return GuardResult::Allow;
        }

        // 1. Incident Budget: Max 5 total actions tied to a single incident_id
        if let Ok(incident_count) = store::get_incident_action_count(incident_id) {
            if incident_count >= 5 {
                return GuardResult::Block(format!(
                    "Incident budget exceeded: Max 5 actions allowed for incident_id {}",
                    incident_id
                ));
            }
        }

        // 2. Per-Resource Limit: Max 3 actions on the same resource within a 10-minute (600s) rolling window
        if let Ok(resource_count) = store::get_resource_action_count(resource, 600) {
            if resource_count >= 3 {
                return GuardResult::Block(format!(
                    "Per-resource limit exceeded: Max 3 actions per 10 minutes for resource '{}'",
                    resource
                ));
            }
        }

        // 3. Cooldown: A 60-second cooldown on the exact same resource after an action is taken
        let ignore_cooldown = std::env::var("IGNORE_COOLDOWN").unwrap_or_default() == "true";
        if !ignore_cooldown {
            if let Ok(Some(secs_since)) = store::get_seconds_since_last_resource_action(resource) {
                if secs_since < 60 {
                    return GuardResult::Block(format!(
                        "Cooldown active: Must wait 60s between actions on resource '{}' (last action {}s ago)",
                        resource, secs_since
                    ));
                }
            }
        }

        GuardResult::Allow
    }
}

pub fn send_outbound_notification(resource: &str, action: &str, reason: &str) {
    let payload = serde_json::json!({
        "event": "requires_human_intervention",
        "resource": resource,
        "action": action,
        "reason": reason,
        "circuit_breaker": "LOCKED",
        "channel": "slack/pagerduty/webhook"
    });
    log::info!("[OUTBOUND NOTIFICATION WEBHOOK] {}", payload);

    if let Ok(webhook_url) = std::env::var("NOTIFICATION_WEBHOOK_URL") {
        if !webhook_url.is_empty() {
            let client = reqwest::Client::new();
            let payload_clone = payload.clone();
            tokio::spawn(async move {
                match client.post(&webhook_url).json(&payload_clone).send().await {
                    Ok(res) => log::info!("Outbound webhook dispatched to {}: status {}", webhook_url, res.status()),
                    Err(e) => log::warn!("Failed to dispatch outbound notification webhook: {}", e),
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_guard_thresholds() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        store::init_db().unwrap();
        store::clear_db().unwrap();

        let resource = "test-pod-guard";
        let test_action = Action::RestartPod {
            pod: resource.to_string(),
            namespace: "default".to_string(),
        };

        // Initial check should allow
        assert_eq!(RemediationGuard::evaluate(1, resource, &test_action), GuardResult::Allow);

        // Log 3 remediations for resource
        store::log_remediation(1, resource, &test_action.to_action_string()).unwrap();
        store::log_remediation(1, resource, &test_action.to_action_string()).unwrap();
        store::log_remediation(1, resource, &test_action.to_action_string()).unwrap();

        // 4th evaluation should be blocked by Per-Resource Limit (Max 3)
        match RemediationGuard::evaluate(1, resource, &test_action) {
            GuardResult::Block(reason) => {
                assert!(reason.contains("Per-resource limit exceeded"));
            }
            _ => panic!("Expected GuardResult::Block for per-resource limit exceeded!"),
        }

        println!("SUCCESS: RemediationGuard per-resource limit threshold verified!");
    }
}

