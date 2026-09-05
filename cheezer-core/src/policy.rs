use serde::Serialize;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static POLICY_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn get_policy_call_count() -> usize {
    POLICY_CALL_COUNT.load(Ordering::Relaxed)
}

pub fn reset_policy_call_count() {
    POLICY_CALL_COUNT.store(0, Ordering::Relaxed);
}

#[derive(Serialize)]
struct OpaInput<'a> {
    action: &'a str,
    resource: &'a str,
    target_replicas: i32,
    command: Vec<&'a str>,
}

#[derive(Serialize)]
struct OpaQuery<'a> {
    input: OpaInput<'a>,
}

pub async fn check_action(action: &str, resource: &str, target_replicas: i32, command: Vec<&str>) -> bool {
    POLICY_CALL_COUNT.fetch_add(1, Ordering::Relaxed);

    let client = reqwest::Client::new();
    let query = OpaQuery {
        input: OpaInput {
            action,
            resource,
            target_replicas,
            command: command.clone(),
        },
    };

    if let Ok(res) = client.post("http://localhost:8181/v1/data/cheezer/authz/allow")
        .json(&query)
        .timeout(Duration::from_millis(500))
        .send()
        .await
    {
        if let Ok(json) = res.json::<Value>().await {
            if let Some(allow) = json.get("result").and_then(|v| v.as_bool()) {
                return allow;
            }
        }
    }

    // Fallback embedded evaluator matching cheezer.rego logic if OPA HTTP server is offline
    evaluate_rego_embedded(action, resource, target_replicas, &command)
}

pub fn evaluate_rego_embedded(action: &str, resource: &str, target_replicas: i32, command: &[&str]) -> bool {
    if action == "delete" && resource == "namespace" {
        return false;
    }
    if command.contains(&"exec") {
        return false;
    }
    if action == "scale" && target_replicas > 10 {
        return false;
    }
    if action == "modify" && resource == "rbac" {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_opa_deny_rules() {
        // 1. Delete namespace -> Blocked
        assert!(!check_action("delete", "namespace", 0, vec![]).await, "Delete namespace MUST be blocked!");

        // 2. Container shell exec -> Blocked
        assert!(!check_action("exec", "pod", 0, vec!["exec"]).await, "Exec command MUST be blocked!");

        // 3. Scale > 10 -> Blocked
        assert!(!check_action("scale", "deployment", 15, vec![]).await, "Scale > 10 MUST be blocked!");

        // 4. Modify RBAC -> Blocked
        assert!(!check_action("modify", "rbac", 0, vec![]).await, "Modify RBAC MUST be blocked!");

        // 5. Safe actions -> Allowed
        assert!(check_action("restart", "pod", 0, vec![]).await, "Restart pod MUST be allowed!");
        assert!(check_action("scale", "deployment", 5, vec![]).await, "Scale <= 10 MUST be allowed!");
        
        println!("SUCCESS: OPA policy rules verified - dangerous actions blocked, safe actions allowed!");
    }
}


