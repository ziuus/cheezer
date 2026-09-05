use crate::action::Action;
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
    target_replicas: u32,
    command: Vec<&'a str>,
}

#[derive(Serialize)]
struct OpaQuery<'a> {
    input: OpaInput<'a>,
}

pub async fn check_action(action: &Action) -> bool {
    POLICY_CALL_COUNT.fetch_add(1, Ordering::Relaxed);

    let action_type = action.action_type();
    let resource = action.resource_type();
    let target_replicas = action.target_replicas();
    let command = action.commands();

    // Fast-path offline fallback for test suites
    if std::env::var("MOCK_OPA_ENABLED").unwrap_or_default() == "true" {
        return evaluate_rego_embedded(action_type, resource, target_replicas, &command);
    }

    let opa_url = std::env::var("OPA_URL")
        .unwrap_or_else(|_| "http://localhost:8181/v1/data/cheezer/authz/allow".to_string());

    let client = reqwest::Client::new();
    let query = OpaQuery {
        input: OpaInput {
            action: action_type,
            resource,
            target_replicas,
            command: command.clone(),
        },
    };

    // Real HTTP request to OPA daemon
    if let Ok(res) = client
        .post(&opa_url)
        .json(&query)
        .timeout(Duration::from_millis(500))
        .send()
        .await
    {
        let status = res.status();
        if status.is_success() {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(allow) = json.get("result").and_then(|v| v.as_bool()) {
                    return allow;
                }
            }
        }
        log::warn!("OPA HTTP daemon returned non-success response: {status}. Defaulting to FAIL-CLOSED (DENY).");
        return false;
    }

    // STRICT FAIL-CLOSED CONSTRAINT: Network errors, timeouts, or daemon offline default to DENY (false)
    log::warn!("OPA HTTP daemon at '{}' unreachable/timed out. Defaulting to FAIL-CLOSED (DENY).", opa_url);
    false
}

pub fn evaluate_rego_embedded(action: &str, resource: &str, target_replicas: u32, command: &[&str]) -> bool {
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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_opa_deny_rules_embedded() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MOCK_OPA_ENABLED", "true");
        }

        // 1. Delete namespace -> Blocked
        let delete_ns = Action::DeleteNamespace { namespace: "production".to_string() };
        assert!(!check_action(&delete_ns).await, "Delete namespace MUST be blocked!");

        // 2. Container shell exec -> Blocked
        let exec_cmd = Action::ExecCommand { pod: "app-pod".to_string(), command: vec!["exec".to_string(), "sh".to_string()] };
        assert!(!check_action(&exec_cmd).await, "Exec command MUST be blocked!");

        // 3. Scale > 10 -> Blocked
        let scale_high = Action::ScaleDeployment { deployment: "myapp".to_string(), target_replicas: 15, namespace: "default".to_string() };
        assert!(!check_action(&scale_high).await, "Scale > 10 MUST be blocked!");

        // 4. Modify RBAC -> Blocked
        let modify_rbac = Action::ModifyRbac { resource: "cluster-admin".to_string() };
        assert!(!check_action(&modify_rbac).await, "Modify RBAC MUST be blocked!");

        // 5. Safe actions -> Allowed
        let restart_pod = Action::RestartPod { pod: "app-pod".to_string(), namespace: "default".to_string() };
        assert!(check_action(&restart_pod).await, "Restart pod MUST be allowed!");

        let scale_ok = Action::ScaleDeployment { deployment: "myapp".to_string(), target_replicas: 5, namespace: "default".to_string() };
        assert!(check_action(&scale_ok).await, "Scale <= 10 MUST be allowed!");

        println!("SUCCESS: OPA policy rules verified - dangerous actions blocked, safe actions allowed!");
    }

    #[tokio::test]
    async fn test_policy_wiremock_real_http() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/data/cheezer/authz/allow"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true
            })))
            .mount(&mock_server)
            .await;

        unsafe {
            std::env::remove_var("MOCK_OPA_ENABLED");
            std::env::set_var("OPA_URL", format!("{}/v1/data/cheezer/authz/allow", mock_server.uri()));
        }

        let restart_action = Action::RestartPod {
            pod: "test-pod-wiremock".to_string(),
            namespace: "default".to_string(),
        };

        let is_allowed = check_action(&restart_action).await;
        assert!(is_allowed, "OPA wiremock HTTP response returning result:true MUST be allowed!");

        unsafe {
            std::env::remove_var("OPA_URL");
            std::env::set_var("MOCK_OPA_ENABLED", "true");
        }
        println!("SUCCESS: Live OPA HTTP POST request verified via wiremock!");
    }

    #[tokio::test]
    async fn test_opa_fail_closed_on_error() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let mock_server = MockServer::start().await;

        // Simulate 500 Internal Server Error from OPA daemon
        Mock::given(method("POST"))
            .and(path("/v1/data/cheezer/authz/allow"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        unsafe {
            std::env::remove_var("MOCK_OPA_ENABLED");
            std::env::set_var("OPA_URL", format!("{}/v1/data/cheezer/authz/allow", mock_server.uri()));
        }

        let restart_action = Action::RestartPod {
            pod: "test-pod-wiremock".to_string(),
            namespace: "default".to_string(),
        };

        let is_allowed = check_action(&restart_action).await;
        assert!(!is_allowed, "Non-200 OPA daemon error status MUST default to FAIL-CLOSED (false)!");

        unsafe {
            std::env::remove_var("OPA_URL");
            std::env::set_var("MOCK_OPA_ENABLED", "true");
        }
        println!("SUCCESS: OPA fail-closed check on HTTP 500 error verified!");
    }
}
